use std::path::Path;
use tauri::State;
use sqlx::SqlitePool;
use crate::db;
use crate::models::RepoInfo;
use crate::state::AppState;

fn get_pool(state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    state
        .db
        .lock()
        .unwrap()
        .clone()
        .ok_or("No repository open".to_string())
}

fn get_repo_path(state: &State<'_, AppState>) -> Result<String, String> {
    state
        .repo_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No repository open".to_string())
}

#[tauri::command]
pub async fn create_repo(
    state: State<'_, AppState>,
    path: String,
) -> Result<RepoInfo, String> {
    let repo_path = Path::new(&path);
    if !repo_path.exists() {
        std::fs::create_dir_all(repo_path)
            .map_err(|e| format!("Cannot create directory: {}", e))?;
    }

    let index_dir = repo_path.join(".index");
    std::fs::create_dir_all(&index_dir)
        .map_err(|e| format!("Cannot create .index: {}", e))?;

    let db_path = index_dir.join("index.db");
    let pool = db::create_pool(&db_path)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    db::run_migrations(&pool)
        .await
        .map_err(|e| format!("Migration error: {}", e))?;

    // Write initial state.json
    let state_json = index_dir.join("state.json");
    std::fs::write(&state_json, r#"{"theme":"light"}"#)
        .map_err(|e| format!("Write error: {}", e))?;

    let item_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM items")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    *state.db.lock().unwrap() = Some(pool);
    *state.repo_path.lock().unwrap() = Some(path.clone());

    Ok(RepoInfo {
        path,
        item_count: item_count.0,
        db_version: 1,
    })
}

#[tauri::command]
pub async fn open_repo(
    state: State<'_, AppState>,
    path: String,
) -> Result<RepoInfo, String> {
    let repo_path = Path::new(&path);
    let index_dir = repo_path.join(".index");

    if !index_dir.exists() {
        return Err("Not a valid Index repository (no .index directory)".to_string());
    }

    let db_path = index_dir.join("index.db");
    let pool = db::create_pool(&db_path)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    db::run_migrations(&pool)
        .await
        .map_err(|e| format!("Migration error: {}", e))?;

    let item_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM items")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    *state.db.lock().unwrap() = Some(pool);
    *state.repo_path.lock().unwrap() = Some(path.clone());

    Ok(RepoInfo {
        path,
        item_count: item_count.0,
        db_version: 1,
    })
}

#[tauri::command]
pub async fn close_repo(state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.db.lock().unwrap().take();
    *state.repo_path.lock().unwrap() = None;
    if let Some(pool) = pool {
        pool.close().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_repo_info(state: State<'_, AppState>) -> Result<RepoInfo, String> {
    let path = get_repo_path(&state)?;
    let pool = get_pool(&state)?;

    let item_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM items")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(RepoInfo {
        path,
        item_count: item_count.0,
        db_version: 1,
    })
}
