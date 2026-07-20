use std::collections::HashMap;
use std::sync::LazyLock;
use sqlx::sqlite::SqlitePool;
use sqlx::QueryBuilder;
use tauri::State;
use crate::models::{FilterNode, Item, OrderBy, QueryResult};
use crate::state::AppState;

/// Tracks which special fields are referenced so we know which JOINs to emit.
struct FieldSet {
    has_group: bool,
    has_tag: bool,
    has_type: bool,
}

impl FieldSet {
    fn new() -> Self {
        Self {
            has_group: false,
            has_tag: false,
            has_type: false,
        }
    }
    fn needs_distinct(&self) -> bool {
        self.has_group || self.has_tag
    }
}

/// Validate a field name — only ASCII alphanumeric, underscore, and CJK characters.
static FIELD_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-zA-Z_\u{4e00}-\u{9fff}][a-zA-Z0-9_\u{4e00}-\u{9fff}]*$").unwrap()
});

fn validate_field(field: &str) -> Result<(), String> {
    if FIELD_RE.is_match(field) {
        Ok(())
    } else {
        Err(format!("Invalid field name: {}", field))
    }
}

/// Map a field name to its SQL column expression. Updates FieldSet for JOIN-triggering fields.
fn resolve_column(field: &str, fs: &mut FieldSet) -> Result<String, String> {
    validate_field(field)?;
    match field {
        "name" => Ok("i.name".to_string()),
        "type_id" => Ok("i.type_id".to_string()),
        "type" => {
            fs.has_type = true;
            Ok("t.name".to_string())
        }
        "created_at" => Ok("i.created_at".to_string()),
        "updated_at" => Ok("i.updated_at".to_string()),
        "group" => {
            fs.has_group = true;
            Ok("g.name".to_string())
        }
        "tag" => {
            fs.has_tag = true;
            Ok("tg.name".to_string())
        }
        // Custom property fields use json_extract
        other => Ok(format!("json_extract(i.properties, '$.{}')", other)),
    }
}

/// Push a typed serde_json::Value into QueryBuilder so SQLite gets int/float/string correctly.
/// Binding numbers as strings would break numeric comparisons with json_extract results.
fn push_bind_value(
    qb: &mut QueryBuilder<'_, sqlx::Sqlite>,
    v: &serde_json::Value,
) -> Result<(), String> {
    match v {
        serde_json::Value::String(s) => {
            qb.push_bind(s.clone());
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                qb.push_bind(i);
            } else if let Some(f) = n.as_f64() {
                qb.push_bind(f);
            } else {
                return Err(format!("Unrepresentable number: {}", n));
            }
        }
        serde_json::Value::Bool(b) => {
            qb.push_bind(if *b { 1i64 } else { 0i64 });
        }
        serde_json::Value::Null => {
            qb.push_bind("".to_string());
        }
        _ => return Err(format!("Unsupported bind value: {:?}", v)),
    }
    Ok(())
}

/// Translate a FilterNode into SQL WHERE clause fragments using QueryBuilder.
/// Returns any regex conditions that couldn't be translated to SQL (for post-filtering).
fn translate_node(
    node: &FilterNode,
    fs: &mut FieldSet,
    qb: &mut QueryBuilder<'_, sqlx::Sqlite>,
    regexes: &mut Vec<(String, regex::Regex)>,
) -> Result<(), String> {
    match node {
        FilterNode::Condition { field, op, value } => {
            // Check if this is a regex condition — defer to Rust
            if op == "regex" {
                let pattern = value.as_str().unwrap_or("");
                match regex::Regex::new(pattern) {
                    Ok(re) => {
                        regexes.push((field.clone(), re));
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(format!(
                            "Invalid regex pattern '{}': {}",
                            pattern, e
                        ));
                    }
                }
            }

            // Validate operator
            let valid_ops = [
                "=", "!=", ">", "<", ">=", "<=", "in", "contains", "is_null",
                "is_not_null",
            ];
            if !valid_ops.contains(&op.as_str()) {
                return Err(format!("Unknown operator: {}", op));
            }

            let col = resolve_column(field, fs)?;

            match op.as_str() {
                "is_null" => {
                    qb.push(format!(
                        "({} IS NULL OR {} = '' OR {} = 'null')",
                        col, col, col
                    ));
                }
                "is_not_null" => {
                    qb.push(format!(
                        "({} IS NOT NULL AND {} != '' AND {} != 'null')",
                        col, col, col
                    ));
                }
                "in" => {
                    let arr = value
                        .as_array()
                        .ok_or("'in' operator requires an array value")?;
                    if arr.is_empty() {
                        qb.push("1 = 0"); // empty IN clause = always false
                        return Ok(());
                    }
                    qb.push(format!("{} IN (", col));
                    for (i, v) in arr.iter().enumerate() {
                        if i > 0 {
                            qb.push(", ");
                        }
                        push_bind_value(qb, v)?;
                    }
                    qb.push(")");
                }
                "contains" => {
                    qb.push(format!("{} LIKE ", col));
                    qb.push("'%' || ");
                    push_bind_value(qb, value)?;
                    qb.push(" || '%'");
                }
                _ => {
                    // = != > < >= <=
                    qb.push(format!("{} {} ", col, op));
                    push_bind_value(qb, value)?;
                }
            }
            Ok(())
        }
        FilterNode::Logic { and, or } => {
            let kids = and
                .as_ref()
                .or(or.as_ref())
                .ok_or("Logic node must have 'and' or 'or'")?;
            let connector = if and.is_some() { " AND " } else { " OR " };

            let mut first = true;
            qb.push("(");
            for child in kids.iter() {
                // If this child is purely regex conditions, skip SQL generation
                // and just collect the regex patterns for post-filtering.
                if is_pure_regex(child) {
                    collect_regex_from_node(child, regexes);
                    continue;
                }

                if !first {
                    qb.push(connector);
                }
                first = false;

                let mut child_regexes: Vec<(String, regex::Regex)> = Vec::new();
                translate_node(child, fs, qb, &mut child_regexes)?;
                regexes.extend(child_regexes);
            }
            if first {
                qb.push("1 = 1");
            }
            qb.push(")");
            Ok(())
        }
    }
}

/// Check if a filter node is purely a regex condition (or a Logic node containing
/// only regex conditions). These nodes produce no SQL and should be deferred to
/// Rust-side post-filtering.
fn is_pure_regex(node: &FilterNode) -> bool {
    match node {
        FilterNode::Condition { op, .. } => op == "regex",
        FilterNode::Logic { and, or } => {
            let kids = and.as_ref().or(or.as_ref());
            kids.map(|k| k.iter().all(is_pure_regex)).unwrap_or(true)
        }
    }
}

/// Collect regex patterns from a node that is known to be pure regex (all children
/// are regex-only). Does not generate any SQL.
fn collect_regex_from_node(node: &FilterNode, regexes: &mut Vec<(String, regex::Regex)>) {
    match node {
        FilterNode::Condition { field, value, .. } => {
            if let Some(pattern) = value.as_str() {
                if let Ok(re) = regex::Regex::new(pattern) {
                    regexes.push((field.clone(), re));
                }
            }
        }
        FilterNode::Logic { and, or } => {
            let kids = and.as_ref().or(or.as_ref());
            if let Some(kids) = kids {
                for child in kids {
                    collect_regex_from_node(child, regexes);
                }
            }
        }
    }
}

/// Pre-scan filter tree to populate FieldSet (determines which JOINs are needed).
fn scan_fields(node: &FilterNode, fs: &mut FieldSet) {
    match node {
        FilterNode::Condition { field, .. } => match field.as_str() {
            "group" => fs.has_group = true,
            "tag" => fs.has_tag = true,
            "type" => fs.has_type = true,
            _ => {}
        },
        FilterNode::Logic { and, or } => {
            let kids = and.as_ref().or(or.as_ref());
            if let Some(kids) = kids {
                for child in kids {
                    scan_fields(child, fs);
                }
            }
        }
    }
}

/// Get a property value from an item's properties JSON. Returns Value::Null if missing.
fn get_property_value(
    properties: &serde_json::Value,
    field: &str,
) -> serde_json::Value {
    properties.get(field).cloned().unwrap_or(serde_json::Value::Null)
}

/// Get the string representation of a field value from an item (for regex matching).
fn get_field_value(item: &Item, field: &str) -> String {
    match field {
        "name" => item.name.clone(),
        "type_id" => item.type_id.to_string(),
        "created_at" => item.created_at.clone(),
        "updated_at" => item.updated_at.clone(),
        _ => match &item.properties {
            serde_json::Value::Object(map) => map
                .get(field)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => String::new(),
                })
                .unwrap_or_default(),
            _ => String::new(),
        },
    }
}

/// Build and execute the full query from a FilterNode.
/// This is the public function that the Tauri command and tests call.
pub async fn execute_query(
    pool: &SqlitePool,
    filter: &FilterNode,
    extract: Option<&Vec<String>>,
    order_by: Option<&OrderBy>,
    limit: Option<i64>,
) -> Result<QueryResult, String> {
    let mut fs = FieldSet::new();
    let mut regexes: Vec<(String, regex::Regex)> = Vec::new();

    // Pre-scan to determine which JOINs are needed
    scan_fields(filter, &mut fs);
    if let Some(ob) = order_by {
        match ob.field.as_str() {
            "group" => fs.has_group = true,
            "tag" => fs.has_tag = true,
            "type" => fs.has_type = true,
            _ => {}
        }
    }

    // Build query using QueryBuilder
    let mut qb = QueryBuilder::new("SELECT ");
    if fs.needs_distinct() {
        qb.push("DISTINCT ");
    }
    qb.push(
        "i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at FROM items i",
    );

    if fs.has_type {
        qb.push(" JOIN item_types t ON i.type_id = t.id");
    }
    if fs.has_group {
        qb.push(" JOIN item_groups ig ON i.id = ig.item_id JOIN groups g ON ig.group_id = g.id");
    }
    if fs.has_tag {
        qb.push(" JOIN item_tags it ON i.id = it.item_id JOIN tags tg ON it.tag_id = tg.id");
    }

    qb.push(" WHERE ");
    let before_where = qb.sql().len();
    translate_node(filter, &mut fs, &mut qb, &mut regexes)?;
    if qb.sql().len() == before_where {
        qb.push("1 = 1");
    }

    // ORDER BY
    if let Some(ob) = order_by {
        let col = resolve_column(&ob.field, &mut fs)?;
        qb.push(format!(
            " ORDER BY {} {}",
            col,
            if ob.desc.unwrap_or(false) {
                "DESC"
            } else {
                "ASC"
            }
        ));
    } else {
        qb.push(" ORDER BY i.updated_at DESC");
    }

    // LIMIT
    if let Some(lim) = limit {
        qb.push(" LIMIT ");
        qb.push_bind(lim);
    }

    // Execute via QueryBuilder
    let rows: Vec<(String, String, i64, String, String, String, String)> = qb
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut items: Vec<Item> = rows
        .into_iter()
        .map(
            |(id, name, type_id, props_str, namespace, created_at, updated_at)| {
                let properties: serde_json::Value =
                    serde_json::from_str(&props_str).unwrap_or_default();
                Item {
                    id,
                    name,
                    type_id,
                    properties,
                    namespace,
                    created_at,
                    updated_at,
                }
            },
        )
        .collect();

    // Apply deferred regex filters
    if !regexes.is_empty() {
        items.retain(|item| {
            regexes.iter().all(|(field, re)| {
                let val = get_field_value(item, field);
                re.is_match(&val)
            })
        });
    }

    let total = items.len() as i64;

    // Extract requested fields
    let extracted = extract.map(|fields| {
        let mut map = HashMap::new();
        for item in &items {
            let mut field_vals = serde_json::Map::new();
            for f in fields {
                let val = get_property_value(&item.properties, f);
                field_vals.insert(f.clone(), val);
            }
            map.insert(item.id.clone(), serde_json::Value::Object(field_vals));
        }
        map
    });

    Ok(QueryResult {
        items,
        total,
        extracted,
    })
}

#[tauri::command]
pub async fn query_items(
    state: State<'_, AppState>,
    filter: FilterNode,
    extract: Option<Vec<String>>,
    order_by: Option<OrderBy>,
    limit: Option<i64>,
) -> Result<QueryResult, String> {
    let pool = state
        .db
        .lock()
        .unwrap()
        .clone()
        .ok_or("No repository open".to_string())?;
    execute_query(
        &pool,
        &filter,
        extract.as_ref(),
        order_by.as_ref(),
        limit,
    )
    .await
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FilterNode;
    use sqlx::QueryBuilder;

    #[test]
    fn test_translate_simple_condition() {
        let node = FilterNode::Condition {
            field: "name".to_string(),
            op: "contains".to_string(),
            value: serde_json::Value::String("Hello".to_string()),
        };
        let mut fs = FieldSet::new();
        let mut qb = QueryBuilder::new("");
        let mut regexes = Vec::new();
        translate_node(&node, &mut fs, &mut qb, &mut regexes).unwrap();
        let sql = qb.sql();
        assert!(
            sql.contains("i.name LIKE"),
            "SQL should contain LIKE: {}",
            sql
        );
        assert!(regexes.is_empty(), "Should have no regexes");
    }

    #[test]
    fn test_translate_regex_condition() {
        let node = FilterNode::Condition {
            field: "name".to_string(),
            op: "regex".to_string(),
            value: serde_json::Value::String("^H".to_string()),
        };
        let mut fs = FieldSet::new();
        let mut qb = QueryBuilder::new("");
        let mut regexes = Vec::new();
        translate_node(&node, &mut fs, &mut qb, &mut regexes).unwrap();
        let sql = qb.sql();
        assert!(
            sql.is_empty(),
            "Regex condition should produce no SQL, got: {}",
            sql
        );
        assert_eq!(regexes.len(), 1, "Should have one regex");
    }

    #[test]
    fn test_translate_mixed_and_group() {
        // {and: [{field:"name",op:"contains",value:"Hello"}, {field:"name",op:"regex",value:"^H"}]}
        // The regex part should be deferred; the SQL should have one LIKE and no dangling/double AND.
        let node = FilterNode::Logic {
            and: Some(vec![
                FilterNode::Condition {
                    field: "name".to_string(),
                    op: "contains".to_string(),
                    value: serde_json::Value::String("Hello".to_string()),
                },
                FilterNode::Condition {
                    field: "name".to_string(),
                    op: "regex".to_string(),
                    value: serde_json::Value::String("^H".to_string()),
                },
            ]),
            or: None,
        };
        let mut fs = FieldSet::new();
        let mut qb = QueryBuilder::new("");
        let mut regexes = Vec::new();
        translate_node(&node, &mut fs, &mut qb, &mut regexes).unwrap();
        let sql = qb.sql();
        assert!(
            sql.contains("i.name LIKE"),
            "SQL should contain LIKE: {}",
            sql
        );
        assert!(
            !sql.contains("AND AND"),
            "Should not have double AND: {}",
            sql
        );
        assert!(
            !sql.trim_end().ends_with("AND"),
            "Should not have dangling AND: {}",
            sql
        );
        assert_eq!(regexes.len(), 1, "Should have one regex");
    }

    #[test]
    fn test_translate_regex_only_and_group() {
        // {and: [{field:"name",op:"regex",value:"^A"}, {field:"name",op:"regex",value:"^B"}]}
        // All children are regex-only, so SQL should fallback to "1 = 1".
        let node = FilterNode::Logic {
            and: Some(vec![
                FilterNode::Condition {
                    field: "name".to_string(),
                    op: "regex".to_string(),
                    value: serde_json::Value::String("^A".to_string()),
                },
                FilterNode::Condition {
                    field: "name".to_string(),
                    op: "regex".to_string(),
                    value: serde_json::Value::String("^B".to_string()),
                },
            ]),
            or: None,
        };
        let mut fs = FieldSet::new();
        let mut qb = QueryBuilder::new("");
        let mut regexes = Vec::new();
        translate_node(&node, &mut fs, &mut qb, &mut regexes).unwrap();
        let sql = qb.sql();
        assert!(
            sql.contains("1 = 1"),
            "All-regex Logic should produce '1 = 1': {}",
            sql
        );
        assert_eq!(regexes.len(), 2, "Should have two regexes");
    }

    #[test]
    fn test_translate_two_sql_conditions_with_and() {
        // Two SQL-producing conditions with AND — no regex involved.
        // Should produce: (condition1 AND condition2) with no double connectors.
        let node = FilterNode::Logic {
            and: Some(vec![
                FilterNode::Condition {
                    field: "name".to_string(),
                    op: "contains".to_string(),
                    value: serde_json::Value::String("Hello".to_string()),
                },
                FilterNode::Condition {
                    field: "name".to_string(),
                    op: "contains".to_string(),
                    value: serde_json::Value::String("World".to_string()),
                },
            ]),
            or: None,
        };
        let mut fs = FieldSet::new();
        let mut qb = QueryBuilder::new("");
        let mut regexes = Vec::new();
        translate_node(&node, &mut fs, &mut qb, &mut regexes).unwrap();
        let sql = qb.sql();
        let and_count = sql.matches(" AND ").count();
        assert_eq!(and_count, 1, "Should have exactly one AND: {}", sql);
        assert!(
            !sql.contains("AND AND"),
            "Should not have double AND: {}",
            sql
        );
        assert!(regexes.is_empty(), "Should have no regexes");
    }
}
