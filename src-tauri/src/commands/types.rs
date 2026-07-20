use tauri::State;
use sqlx::SqlitePool;
use crate::models::{Field, ItemType};
use crate::state::AppState;

fn get_pool(state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    state
        .db
        .lock()
        .unwrap()
        .clone()
        .ok_or("No repository open".to_string())
}

fn parse_options(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

#[tauri::command]
pub async fn list_item_types(state: State<'_, AppState>) -> Result<Vec<ItemType>, String> {
    let pool = get_pool(&state)?;

    let type_rows: Vec<(i64, String, String, String)> =
        sqlx::query_as("SELECT id, name, icon, namespace FROM item_types ORDER BY id")
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?;

    let mut result = vec![];
    for (id, name, icon, namespace) in type_rows {
        let field_rows: Vec<(i64, i64, String, String, String, i32, String, String)> = sqlx::query_as(
            "SELECT f.id, f.type_id, f.name, f.field_type, f.icon, f.position, f.label, f.options FROM fields f WHERE f.type_id = ? ORDER BY f.position",
        )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

        let fields = field_rows
            .into_iter()
            .map(|(fid, tid, fname, ftype, ficon, pos, label, opts)| Field {
                id: fid,
                type_id: tid,
                name: fname,
                field_type: ftype,
                icon: ficon,
                position: pos,
                label,
                options: parse_options(&opts),
            })
            .collect();

        result.push(ItemType {
            id,
            name,
            icon,
            namespace,
            fields,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn create_item_type(
    state: State<'_, AppState>,
    name: String,
    icon: Option<String>,
) -> Result<ItemType, String> {
    let pool = get_pool(&state)?;
    let icon = icon.unwrap_or_else(|| "📄".to_string());

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO item_types (name, icon, namespace) VALUES (?, ?, 'default') RETURNING id",
    )
    .bind(&name)
    .bind(&icon)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ItemType {
        id,
        name,
        icon,
        namespace: "default".into(),
        fields: vec![],
    })
}

#[tauri::command]
pub async fn delete_item_type(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let pool = get_pool(&state)?;
    // Preset types (1=通用, 2=任务) are protected
    if id == 1 || id == 2 {
        return Err("Cannot delete preset types".to_string());
    }
    sqlx::query("DELETE FROM item_types WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_field(
    state: State<'_, AppState>,
    type_id: i64,
    name: String,
    field_type: String,
    icon: Option<String>,
    label: Option<String>,
    options: Option<Vec<String>>,
) -> Result<Field, String> {
    let pool = get_pool(&state)?;
    let icon = icon.unwrap_or_else(|| "circle".to_string());
    let label = label.unwrap_or_default();
    let opts = serde_json::to_string(&options.clone().unwrap_or_default()).map_err(|e| e.to_string())?;
    let options_list = options.unwrap_or_default();

    let max_pos: Option<i32> =
        sqlx::query_scalar("SELECT MAX(position) FROM fields WHERE type_id = ?")
            .bind(type_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
    let position = max_pos.unwrap_or(-1) + 1;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO fields (type_id, name, field_type, icon, position, label, options) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(type_id)
    .bind(&name)
    .bind(&field_type)
    .bind(&icon)
    .bind(position)
    .bind(&label)
    .bind(&opts)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Field {
        id,
        type_id,
        name,
        field_type,
        icon,
        position,
        label,
        options: options_list,
    })
}

#[tauri::command]
pub async fn remove_field(state: State<'_, AppState>, field_id: i64) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("DELETE FROM fields WHERE id = ?")
        .bind(field_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn reorder_fields(
    state: State<'_, AppState>,
    type_id: i64,
    field_ids: Vec<i64>,
) -> Result<(), String> {
    let pool = get_pool(&state)?;
    for (i, field_id) in field_ids.into_iter().enumerate() {
        sqlx::query("UPDATE fields SET position = ? WHERE id = ? AND type_id = ?")
            .bind(i as i32)
            .bind(field_id)
            .bind(type_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn update_item_type(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    icon: String,
) -> Result<ItemType, String> {
    let pool = get_pool(&state)?;

    sqlx::query("UPDATE item_types SET name = ?, icon = ? WHERE id = ?")
        .bind(&name)
        .bind(&icon)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // Return updated type with its fields
    let (_, _, _, namespace): (i64, String, String, String) = sqlx::query_as(
        "SELECT id, name, icon, namespace FROM item_types WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let field_rows: Vec<(i64, i64, String, String, String, i32, String, String)> = sqlx::query_as(
        "SELECT f.id, f.type_id, f.name, f.field_type, f.icon, f.position, f.label, f.options FROM fields f WHERE f.type_id = ? ORDER BY f.position",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let fields = field_rows
        .into_iter()
        .map(|(fid, tid, fname, ftype, ficon, pos, label, opts)| Field {
            id: fid,
            type_id: tid,
            name: fname,
            field_type: ftype,
            icon: ficon,
            position: pos,
            label,
            options: parse_options(&opts),
        })
        .collect();

    Ok(ItemType { id, name, icon, namespace, fields })
}

#[tauri::command]
pub async fn update_field(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    field_type: String,
    icon: String,
    label: String,
    options: Option<Vec<String>>,
) -> Result<Field, String> {
    let pool = get_pool(&state)?;
    let opts = serde_json::to_string(&options.clone().unwrap_or_default()).map_err(|e| e.to_string())?;
    let options_list = options.unwrap_or_default();

    // Get old field info before update (for cleanup)
    let old: (i64, String) = sqlx::query_as(
        "SELECT type_id, field_type FROM fields WHERE id = ?"
    ).bind(id).fetch_one(&pool).await.map_err(|e| e.to_string())?;

    sqlx::query("UPDATE fields SET name = ?, field_type = ?, icon = ?, label = ?, options = ? WHERE id = ?")
        .bind(&name)
        .bind(&field_type)
        .bind(&icon)
        .bind(&label)
        .bind(&opts)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // ── Cleanup: clear field values in all items of this type ──
    let opts_list: Vec<String> = options_list.clone();
    let needs_cleanup = old.1 != field_type || !opts_list.is_empty();

    if needs_cleanup {
        let items: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, properties FROM items WHERE type_id = ?"
        ).bind(old.0).fetch_all(&pool).await.map_err(|e| e.to_string())?;

        for (item_id, props_str) in items {
            let mut props: serde_json::Value = serde_json::from_str(&props_str).unwrap_or_default();
            let field_name = &name;

            if field_type == "dropdown" {
                // Keep value only if it's in the options list
                let keep = props.get(field_name)
                    .and_then(|v| v.as_str())
                    .map(|s| opts_list.contains(&s.to_string()))
                    .unwrap_or(false);
                if !keep {
                    if let serde_json::Value::Object(ref mut map) = props {
                        map.remove(field_name);
                    }
                }
            } else if field_type == "checkbox" {
                // Convert to boolean or clear
                if let serde_json::Value::Object(ref mut map) = props {
                    if let Some(v) = map.get(field_name) {
                        if let Some(s) = v.as_str() {
                            map.insert(field_name.to_string(), serde_json::Value::Bool(s == "true" || s == "1"));
                        } else if !v.is_boolean() {
                            map.remove(field_name);
                        }
                    }
                }
            } else {
                // For type change to text/number/date: clear old value
                if old.1 != field_type {
                    if let serde_json::Value::Object(ref mut map) = props {
                        map.remove(field_name);
                    }
                }
            }

            let new_props = serde_json::to_string(&props).map_err(|e| e.to_string())?;
            if new_props != props_str {
                sqlx::query("UPDATE items SET properties = ? WHERE id = ?")
                    .bind(&new_props)
                    .bind(&item_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    // Return updated field
    let (_, type_id, _, _, _, pos, _, opts_str): (i64, i64, String, String, String, i32, String, String) = sqlx::query_as(
        "SELECT id, type_id, name, field_type, icon, position, label, options FROM fields WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let opts_parsed = parse_options(&opts_str);

    Ok(Field {
        id,
        type_id,
        name,
        field_type,
        icon,
        position: pos,
        label,
        options: opts_parsed,
    })
}
