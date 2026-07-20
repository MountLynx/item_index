use std::collections::HashMap;
use std::sync::LazyLock;
use sqlx::sqlite::SqlitePool;
use sqlx::QueryBuilder;
use tauri::State;
use crate::models::{FilterNode, Item, OrderBy, QueryResult};
use crate::state::AppState;

fn get_pool(window: &tauri::Window, state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    let label = window.label().to_string();
    state.repos.lock().unwrap()
        .get(&label)
        .map(|r| r.db.clone())
        .ok_or("No repository open".to_string())
}

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

/// Check whether every condition in the subtree is a regex condition.
/// These nodes produce no SQL and can be skipped entirely during SQL generation.
fn is_pure_regex(node: &FilterNode) -> bool {
    match node {
        FilterNode::Condition { op, .. } => op == "regex",
        FilterNode::Logic { and, or } => {
            let kids = and.as_ref().or(or.as_ref());
            kids.map(|k| k.iter().all(is_pure_regex)).unwrap_or(true)
        }
    }
}

/// Check whether any condition in the subtree is a regex condition.
/// Used to detect OR groups that must be deferred to evaluate_filter.
fn contains_regex(node: &FilterNode) -> bool {
    match node {
        FilterNode::Condition { op, .. } => op == "regex",
        FilterNode::Logic { and, or } => {
            let kids = and.as_ref().or(or.as_ref());
            kids.map(|k| k.iter().any(contains_regex)).unwrap_or(false)
        }
    }
}

/// Translate a FilterNode into SQL WHERE clause fragments using QueryBuilder.
fn translate_node(
    node: &FilterNode,
    fs: &mut FieldSet,
    qb: &mut QueryBuilder<'_, sqlx::Sqlite>,
) -> Result<(), String> {
    match node {
        FilterNode::Condition { field, op, value } => {
            // Regex conditions are deferred to Rust-side post-filtering (evaluate_filter)
            if op == "regex" {
                return Ok(());
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
            let is_or = or.is_some();
            let connector = if is_or { " OR " } else { " AND " };

            qb.push("(");

            // OR with any regex child: can't filter in SQL because the regex
            // could match items that the non-regex branches miss. Defer entirely
            // to evaluate_filter by emitting a no-op (1 = 1).
            if is_or && kids.iter().any(contains_regex) {
                qb.push("1 = 1");
            } else {
                let mut first = true;
                for child in kids.iter() {
                    // Skip children that produce no SQL (pure regex subtrees).
                    // evaluate_filter will apply their constraints in post-filter.
                    if is_pure_regex(child) {
                        continue;
                    }
                    if !first {
                        qb.push(connector);
                    }
                    first = false;
                    translate_node(child, fs, qb)?;
                }
                if first {
                    qb.push("1 = 1");
                }
            }
            qb.push(")");
            Ok(())
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

/// Get the typed field value from an item (for evaluate_filter).
fn get_field_value(item: &Item, field: &str) -> serde_json::Value {
    match field {
        "name" => serde_json::Value::String(item.name.clone()),
        "type_id" => serde_json::json!(item.type_id),
        "created_at" => serde_json::Value::String(item.created_at.clone()),
        "updated_at" => serde_json::Value::String(item.updated_at.clone()),
        _ => item.properties.get(field).cloned().unwrap_or(serde_json::Value::Null),
    }
}

/// Compare two serde_json::Values for ordering (used by evaluate_filter).
fn compare_values(a: &serde_json::Value, b: &serde_json::Value) -> std::cmp::Ordering {
    match (a, b) {
        (serde_json::Value::Number(an), serde_json::Value::Number(bn)) => {
            match (an.as_f64(), bn.as_f64()) {
                (Some(af), Some(bf)) => af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            }
        }
        (serde_json::Value::String(av), serde_json::Value::String(bv)) => av.cmp(bv),
        _ => std::cmp::Ordering::Equal,
    }
}

/// Post-filter items using the original filter tree. Evaluates ALL conditions (both
/// regex and non-regex) so that the OR/AND structure is correctly preserved. SQL acts
/// as a pre-filter (superset) for performance; this function is the final authority.
fn evaluate_filter(item: &Item, node: &FilterNode) -> bool {
    match node {
        FilterNode::Condition { field, op, value } => {
            let val = get_field_value(item, field);
            match op.as_str() {
                "=" => val == *value,
                "!=" => val != *value,
                ">" => compare_values(&val, value) == std::cmp::Ordering::Greater,
                "<" => compare_values(&val, value) == std::cmp::Ordering::Less,
                ">=" => matches!(
                    compare_values(&val, value),
                    std::cmp::Ordering::Greater | std::cmp::Ordering::Equal
                ),
                "<=" => matches!(
                    compare_values(&val, value),
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal
                ),
                "in" => value
                    .as_array()
                    .map(|arr| arr.contains(&val))
                    .unwrap_or(false),
                "contains" => {
                    let s = val.as_str().unwrap_or("").to_lowercase();
                    let pat = value.as_str().unwrap_or("").to_lowercase();
                    s.contains(&pat)
                }
                "is_null" => {
                    val.is_null()
                        || val.as_str().map(|s| s.is_empty() || s == "null").unwrap_or(false)
                }
                "is_not_null" => {
                    !val.is_null()
                        && val.as_str().map(|s| !s.is_empty() && s != "null").unwrap_or(true)
                }
                "regex" => {
                    let pattern = value.as_str().unwrap_or("");
                    match regex::Regex::new(pattern) {
                        Ok(re) => {
                            let s = val.as_str().unwrap_or("");
                            re.is_match(s)
                        }
                        Err(_) => false,
                    }
                }
                _ => true,
            }
        }
        FilterNode::Logic { and, or } => {
            let kids = and.as_ref().or(or.as_ref());
            match kids {
                None => true,
                Some(kids) => {
                    if and.is_some() {
                        kids.iter().all(|child| evaluate_filter(item, child))
                    } else {
                        kids.iter().any(|child| evaluate_filter(item, child))
                    }
                }
            }
        }
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
    translate_node(filter, &mut fs, &mut qb)?;
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

    // Apply deferred regex filters (preserving AND/OR structure)
    items.retain(|item| evaluate_filter(item, filter));

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
    window: tauri::Window,
    state: State<'_, AppState>,
    filter: FilterNode,
    extract: Option<Vec<String>>,
    order_by: Option<OrderBy>,
    limit: Option<i64>,
) -> Result<QueryResult, String> {
    let pool = get_pool(&window, &state)?;
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
        translate_node(&node, &mut fs, &mut qb).unwrap();
        let sql = qb.sql();
        assert!(
            sql.contains("i.name LIKE"),
            "SQL should contain LIKE: {}",
            sql
        );
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
        translate_node(&node, &mut fs, &mut qb).unwrap();
        let sql = qb.sql();
        assert!(
            sql.is_empty(),
            "Regex condition should produce no SQL, got: {}",
            sql
        );
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
        translate_node(&node, &mut fs, &mut qb).unwrap();
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
        translate_node(&node, &mut fs, &mut qb).unwrap();
        let sql = qb.sql();
        assert!(
            sql.contains("1 = 1"),
            "All-regex Logic should produce '1 = 1': {}",
            sql
        );
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
        translate_node(&node, &mut fs, &mut qb).unwrap();
        let sql = qb.sql();
        let and_count = sql.matches(" AND ").count();
        assert_eq!(and_count, 1, "Should have exactly one AND: {}", sql);
        assert!(
            !sql.contains("AND AND"),
            "Should not have double AND: {}",
            sql
        );
    }
}
