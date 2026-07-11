use tauri::State;
use sqlx::SqlitePool;
use crate::models::Tag;
use crate::state::AppState;

fn get_pool(state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    state.db.lock().unwrap().clone().ok_or("No repository open".to_string())
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let pool = get_pool(&state)?;
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, name FROM tags ORDER BY name"
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name)| Tag { id, name }).collect())
}

#[tauri::command]
pub async fn create_tag(state: State<'_, AppState>, name: String) -> Result<Tag, String> {
    let pool = get_pool(&state)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO tags (name) VALUES (?) RETURNING id"
    ).bind(&name).fetch_one(&pool).await.map_err(|e| e.to_string())?;

    Ok(Tag { id, name })
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_tag_to_item(
    state: State<'_, AppState>,
    item_id: String,
    tag_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?, ?)")
        .bind(&item_id).bind(tag_id)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_item(
    state: State<'_, AppState>,
    item_id: String,
    tag_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("DELETE FROM item_tags WHERE item_id = ? AND tag_id = ?")
        .bind(&item_id).bind(tag_id)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}
