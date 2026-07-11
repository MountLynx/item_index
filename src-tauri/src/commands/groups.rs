use tauri::State;
use sqlx::SqlitePool;
use crate::models::Group;
use crate::state::AppState;

fn get_pool(state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    state.db.lock().unwrap().clone().ok_or("No repository open".to_string())
}

fn build_tree(all: &[(i64, Option<i64>, String, i32)], parent_id: Option<i64>) -> Vec<Group> {
    all.iter()
        .filter(|(_, pid, _, _)| *pid == parent_id)
        .map(|(id, _, name, position)| Group {
            id: *id,
            parent_id: parent_id,
            name: name.clone(),
            position: *position,
            children: build_tree(all, Some(*id)),
        })
        .collect()
}

#[tauri::command]
pub async fn list_groups(state: State<'_, AppState>) -> Result<Vec<Group>, String> {
    let pool = get_pool(&state)?;
    let rows: Vec<(i64, Option<i64>, String, i32)> = sqlx::query_as(
        "SELECT id, parent_id, name, position FROM groups ORDER BY position"
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    Ok(build_tree(&rows, None))
}

#[tauri::command]
pub async fn create_group(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<i64>,
) -> Result<Group, String> {
    let pool = get_pool(&state)?;

    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM groups WHERE parent_id IS ?"
    ).bind(parent_id).fetch_one(&pool).await.map_err(|e| e.to_string())?;
    let position = max_pos.unwrap_or(-1) + 1;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO groups (name, parent_id, position) VALUES (?, ?, ?) RETURNING id"
    ).bind(&name).bind(parent_id).bind(position)
        .fetch_one(&pool).await.map_err(|e| e.to_string())?;

    Ok(Group { id, parent_id, name, position, children: vec![] })
}

#[tauri::command]
pub async fn update_group(
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
) -> Result<Group, String> {
    let pool = get_pool(&state)?;

    if let Some(n) = &name {
        sqlx::query("UPDATE groups SET name = ? WHERE id = ?")
            .bind(n).bind(id).execute(&pool).await.map_err(|e| e.to_string())?;
    }

    let (name, parent_id, position): (String, Option<i64>, i32) = sqlx::query_as(
        "SELECT name, parent_id, position FROM groups WHERE id = ?"
    ).bind(id).fetch_one(&pool).await.map_err(|e| e.to_string())?;

    Ok(Group { id, parent_id, name, position, children: vec![] })
}

#[tauri::command]
pub async fn delete_group(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("DELETE FROM groups WHERE id = ?")
        .bind(id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn move_group(
    state: State<'_, AppState>,
    id: i64,
    parent_id: Option<i64>,
    position: i32,
) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("UPDATE groups SET parent_id = ?, position = ? WHERE id = ?")
        .bind(parent_id).bind(position).bind(id)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_item_to_group(
    state: State<'_, AppState>,
    item_id: String,
    group_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("INSERT OR IGNORE INTO item_groups (item_id, group_id) VALUES (?, ?)")
        .bind(&item_id).bind(group_id)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_item_from_group(
    state: State<'_, AppState>,
    item_id: String,
    group_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&state)?;
    sqlx::query("DELETE FROM item_groups WHERE item_id = ? AND group_id = ?")
        .bind(&item_id).bind(group_id)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}
