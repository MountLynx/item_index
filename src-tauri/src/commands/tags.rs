use tauri::State;
use sqlx::SqlitePool;
use crate::models::Tag;
use crate::state::AppState;

fn get_pool(window: &tauri::Window, state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    let label = window.label().to_string();
    state.repos.lock().unwrap()
        .get(&label)
        .map(|r| r.db.clone())
        .ok_or("No repository open".to_string())
}

#[tauri::command]
pub async fn list_tags(window: tauri::Window, state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let pool = get_pool(&window, &state)?;
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, name, namespace FROM tags ORDER BY name"
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name, namespace)| Tag { id, name, namespace }).collect())
}

#[tauri::command]
pub async fn create_tag(window: tauri::Window, state: State<'_, AppState>, name: String) -> Result<Tag, String> {
    let pool = get_pool(&window, &state)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO tags (name, namespace) VALUES (?, 'default') RETURNING id"
    ).bind(&name).fetch_one(&pool).await.map_err(|e| e.to_string())?;

    Ok(Tag { id, name, namespace: "default".into() })
}

#[tauri::command]
pub async fn delete_tag(window: tauri::Window, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let pool = get_pool(&window, &state)?;
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_tag_to_item(
    window: tauri::Window,
    state: State<'_, AppState>,
    item_id: String,
    tag_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&window, &state)?;
    sqlx::query("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?, ?)")
        .bind(&item_id).bind(tag_id)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_item(
    window: tauri::Window,
    state: State<'_, AppState>,
    item_id: String,
    tag_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&window, &state)?;
    sqlx::query("DELETE FROM item_tags WHERE item_id = ? AND tag_id = ?")
        .bind(&item_id).bind(tag_id)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}
