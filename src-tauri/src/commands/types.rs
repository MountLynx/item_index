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

#[tauri::command]
pub async fn list_item_types(state: State<'_, AppState>) -> Result<Vec<ItemType>, String> {
    let pool = get_pool(&state)?;

    let type_rows: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT id, name, icon FROM item_types ORDER BY id")
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?;

    let mut result = vec![];
    for (id, name, icon) in type_rows {
        let field_rows: Vec<(i64, i64, String, String, i32)> = sqlx::query_as(
            "SELECT f.id, f.type_id, f.name, f.field_type, f.position FROM fields f WHERE f.type_id = ? ORDER BY f.position",
        )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

        let fields = field_rows
            .into_iter()
            .map(|(fid, tid, fname, ftype, pos)| Field {
                id: fid,
                type_id: tid,
                name: fname,
                field_type: ftype,
                position: pos,
            })
            .collect();

        result.push(ItemType {
            id,
            name,
            icon,
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
        "INSERT INTO item_types (name, icon) VALUES (?, ?) RETURNING id",
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
) -> Result<Field, String> {
    let pool = get_pool(&state)?;

    let max_pos: Option<i32> =
        sqlx::query_scalar("SELECT MAX(position) FROM fields WHERE type_id = ?")
            .bind(type_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
    let position = max_pos.unwrap_or(-1) + 1;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO fields (type_id, name, field_type, position) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(type_id)
    .bind(&name)
    .bind(&field_type)
    .bind(position)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Field {
        id,
        type_id,
        name,
        field_type,
        position,
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
