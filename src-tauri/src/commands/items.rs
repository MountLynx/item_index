use std::path::Path;
use sha2::{Sha256, Digest};
use rand::Rng;
use tauri::State;
use sqlx::SqlitePool;
use crate::models::{Item, ItemDetail, ItemType, Field, Group, Tag, FileNode};
use crate::state::AppState;

fn get_pool(window: &tauri::Window, state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    let label = window.label().to_string();
    state.repos.lock().unwrap()
        .get(&label)
        .map(|r| r.db.clone())
        .ok_or("No repository open".to_string())
}

fn get_repo_path(window: &tauri::Window, state: &State<'_, AppState>) -> Result<String, String> {
    let label = window.label().to_string();
    state.repos.lock().unwrap()
        .get(&label)
        .map(|r| r.path.clone())
        .ok_or("No repository open".to_string())
}

fn generate_id() -> String {
    let random_bytes: [u8; 16] = rand::thread_rng().gen();
    let hash = Sha256::digest(random_bytes);
    hex::encode(&hash[..6]) // 12 hex chars
}

fn read_dir_recursive(path: &Path) -> Result<Vec<FileNode>, String> {
    let mut children = vec![];
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let sub_children = if is_dir {
            read_dir_recursive(&entry.path())?
        } else {
            vec![]
        };
        children.push(FileNode { name, is_dir, children: sub_children });
    }
    children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(children)
}

async fn fetch_type(pool: &SqlitePool, type_id: i64) -> Result<ItemType, String> {
    let (id, name, icon, namespace): (i64, String, String, String) = sqlx::query_as(
        "SELECT id, name, icon, namespace FROM item_types WHERE id = ?"
    ).bind(type_id).fetch_one(pool).await.map_err(|e| e.to_string())?;

    let field_rows: Vec<(i64, i64, String, String, String, i32, String, String)> = sqlx::query_as(
        "SELECT id, type_id, name, field_type, icon, position, label, options FROM fields WHERE type_id = ? ORDER BY position"
    ).bind(id).fetch_all(pool).await.map_err(|e| e.to_string())?;

    let fields: Vec<Field> = field_rows.into_iter().map(|(fid, tid, n, ft, ficon, pos, label, opts)| Field {
        id: fid, type_id: tid, name: n, field_type: ft, icon: ficon, position: pos, label,
        options: serde_json::from_str(&opts).unwrap_or_default(),
    }).collect();

    Ok(ItemType { id, name, icon, namespace, fields })
}

async fn fetch_item_groups(pool: &SqlitePool, item_id: &str) -> Result<Vec<Group>, String> {
    let rows: Vec<(i64, Option<i64>, String, i32)> = sqlx::query_as(
        "SELECT g.id, g.parent_id, g.name, g.position FROM groups g
         INNER JOIN item_groups ig ON g.id = ig.group_id
         WHERE ig.item_id = ?"
    ).bind(item_id).fetch_all(pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, parent_id, name, position)| Group {
        id, parent_id, name, position, children: vec![],
    }).collect())
}

async fn fetch_item_tags(pool: &SqlitePool, item_id: &str) -> Result<Vec<Tag>, String> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT t.id, t.name, t.namespace FROM tags t
         INNER JOIN item_tags it ON t.id = it.tag_id
         WHERE it.item_id = ?"
    ).bind(item_id).fetch_all(pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name, namespace)| Tag { id, name, namespace }).collect())
}

fn map_item_rows(rows: Vec<(String, String, i64, String, String, String, String)>) -> Vec<Item> {
    rows.into_iter().map(|(id, name, type_id, props_str, namespace, created_at, updated_at)| {
        let properties: serde_json::Value = serde_json::from_str(&props_str).unwrap_or_default();
        Item { id, name, type_id, properties, namespace, created_at, updated_at }
    }).collect()
}

#[tauri::command]
pub async fn create_item(
    window: tauri::Window,
    state: State<'_, AppState>,
    type_id: i64,
    name: String,
) -> Result<Item, String> {
    let pool = get_pool(&window, &state)?;
    let id = generate_id();
    let now = chrono::Utc::now().to_rfc3339();

    // Item folder + .md are created on-demand instead of eagerly,
    // keeping lightweight item types (e.g. expense tracking) filesystem-clean.
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at)
         VALUES (?, ?, ?, '{}', 'default', ?, ?)"
    ).bind(&id).bind(&name).bind(type_id).bind(&now).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Commit — item folder + .md are created on-demand instead of eagerly,
    // keeping lightweight item types (e.g. expense tracking) filesystem-clean.
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(Item {
        id, name, type_id,
        properties: serde_json::Value::Object(serde_json::Map::new()),
        namespace: "default".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn get_item(
    window: tauri::Window,
    state: State<'_, AppState>,
    id: String,
) -> Result<ItemDetail, String> {
    let pool = get_pool(&window, &state)?;
    let repo_path = get_repo_path(&window, &state)?;

    let (name, type_id, properties_str, namespace, created_at, updated_at): (
        String, i64, String, String, String, String,
    ) = sqlx::query_as(
        "SELECT name, type_id, properties, namespace, created_at, updated_at FROM items WHERE id = ?"
    ).bind(&id).fetch_one(&pool).await.map_err(|e| format!("Item not found: {}", e))?;

    let properties: serde_json::Value = serde_json::from_str(&properties_str).unwrap_or_default();
    let item = Item { id: id.clone(), name, type_id, properties, namespace, created_at, updated_at };
    let item_type = fetch_type(&pool, type_id).await?;
    let groups = fetch_item_groups(&pool, &id).await?;
    let tags = fetch_item_tags(&pool, &id).await?;

    let item_dir = Path::new(&repo_path).join(&id);
    let files = FileNode {
        name: id.clone(),
        is_dir: true,
        children: if item_dir.exists() {
            read_dir_recursive(&item_dir)?
        } else {
            vec![]
        }
    };

    Ok(ItemDetail { item, item_type, groups, tags, files })
}

#[tauri::command]
pub async fn list_items(
    window: tauri::Window,
    state: State<'_, AppState>,
    group_id: Option<i64>,
    tag_id: Option<i64>,
    type_ids: Option<Vec<i64>>,
) -> Result<Vec<Item>, String> {
    let pool = get_pool(&window, &state)?;

    let has_group = group_id.is_some();
    let has_tag = tag_id.is_some();
    let has_types = type_ids.as_ref().map_or(false, |v| !v.is_empty());

    // Fast path: no filters at all
    if !has_group && !has_tag && !has_types {
        let rows: Vec<(String, String, i64, String, String, String, String)> = sqlx::query_as(
            "SELECT id, name, type_id, properties, namespace, created_at, updated_at \
             FROM items ORDER BY updated_at DESC"
        ).fetch_all(&pool).await.map_err(|e| e.to_string())?;
        return Ok(map_item_rows(rows));
    }

    // Build query dynamically from independent filter clauses
    let mut joins = Vec::new();
    let mut conditions = Vec::new();
    let mut param_idx = 1usize; // sqlx SQLite uses 1-based ?N numbered params

    if has_group {
        joins.push("INNER JOIN item_groups ig ON i.id = ig.item_id");
        conditions.push(format!("ig.group_id = ?{}", param_idx));
        param_idx += 1;
    }
    if has_tag {
        joins.push("INNER JOIN item_tags it ON i.id = it.item_id");
        conditions.push(format!("it.tag_id = ?{}", param_idx));
        param_idx += 1;
    }
    if let Some(ref tids) = type_ids {
        let placeholders: Vec<String> = (0..tids.len())
            .map(|i| format!("?{}", param_idx + i))
            .collect();
        conditions.push(format!("i.type_id IN ({})", placeholders.join(",")));
    }

    let joins_str = joins.join(" ");
    let where_str = conditions.join(" AND ");
    // DISTINCT needed when joining to avoid cartesian-product duplicates
    let distinct = if !joins.is_empty() { "DISTINCT " } else { "" };

    let query = format!(
        "SELECT {}i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at \
         FROM items i {} WHERE {} ORDER BY i.updated_at DESC",
        distinct, joins_str, where_str
    );

    let mut q = sqlx::query_as::<_, (String, String, i64, String, String, String, String)>(&query);
    if let Some(gid) = group_id { q = q.bind(gid); }
    if let Some(tid) = tag_id { q = q.bind(tid); }
    if let Some(tids) = &type_ids { for tid in tids { q = q.bind(tid); } }

    let rows = q.fetch_all(&pool).await.map_err(|e| e.to_string())?;
    Ok(map_item_rows(rows))
}

#[tauri::command]
pub async fn update_item(
    window: tauri::Window,
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    properties: Option<serde_json::Value>,
) -> Result<Item, String> {
    let pool = get_pool(&window, &state)?;
    let now = chrono::Utc::now().to_rfc3339();

    if name.is_none() && properties.is_none() {
        return Err("Nothing to update".to_string());
    }

    // Build single dynamic UPDATE — one atomic statement for all changed columns
    let mut set_parts: Vec<String> = Vec::new();
    if name.is_some()       { set_parts.push("name = ?".to_string()); }
    if properties.is_some() { set_parts.push("properties = ?".to_string()); }
    set_parts.push("updated_at = ?".to_string());

    let query = format!("UPDATE items SET {} WHERE id = ?", set_parts.join(", "));

    let mut q = sqlx::query(&query);
    if let Some(ref n) = name { q = q.bind(n); }

    // Serialized properties must outlive the query execution
    let props_json: Option<String> = if let Some(ref p) = properties {
        Some(serde_json::to_string(p).map_err(|e| e.to_string())?)
    } else {
        None
    };
    if let Some(ref pj) = props_json { q = q.bind(pj); }

    q = q.bind(&now).bind(&id);
    q.execute(&pool).await.map_err(|e| e.to_string())?;

    // Fetch updated row
    let (name, type_id, props_str, namespace, created_at, updated_at): (
        String, i64, String, String, String, String,
    ) = sqlx::query_as(
        "SELECT name, type_id, properties, namespace, created_at, updated_at FROM items WHERE id = ?"
    ).bind(&id).fetch_one(&pool).await.map_err(|e| e.to_string())?;

    let properties: serde_json::Value = serde_json::from_str(&props_str).unwrap_or_default();
    Ok(Item { id, name, type_id, properties, namespace, created_at, updated_at })
}

#[tauri::command]
pub async fn delete_item(
    window: tauri::Window,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let pool = get_pool(&window, &state)?;
    let repo_path = get_repo_path(&window, &state)?;

    // Delete from DB first (in transaction) — CASCADE handles junction tables.
    // If this succeeds, the item is gone from the app regardless of filesystem outcome.
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;

    // Clean up sub_repos.json if this item was a sub-repo
    let sub_repos_path = Path::new(&repo_path).join(".index").join("sub_repos.json");
    if sub_repos_path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&sub_repos_path) {
            if let Ok(mut map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&raw) {
                if map.remove(&id).is_some() {
                    let _ = std::fs::write(&sub_repos_path, serde_json::to_string_pretty(&map).unwrap_or_default());
                }
            }
        }
    }

    // Then delete hash folder (best-effort — ignore errors, orphaned folder
    // is a harmless disk leak, not data corruption)
    let item_dir = Path::new(&repo_path).join(&id);
    if item_dir.exists() {
        let _ = std::fs::remove_dir_all(&item_dir);
    }

    Ok(())
}

/// Create the item folder + <name>.md file (for a single item).
/// Safe to call even if the folder already exists.
fn ensure_item_folder(repo_path: &str, item_id: &str, name: &str) -> Result<(), String> {
    let item_dir = Path::new(repo_path).join(item_id);
    if !item_dir.exists() {
        std::fs::create_dir_all(&item_dir)
            .map_err(|e| format!("Cannot create item folder: {}", e))?;
        let md_content = format!("# {}\n", name);
        std::fs::write(item_dir.join(format!("{}.md", name)), md_content)
            .map_err(|e| {
                let _ = std::fs::remove_dir_all(&item_dir);
                format!("Cannot create .md file: {}", e)
            })?;
    }
    Ok(())
}

#[tauri::command]
pub async fn create_item_folder(
    window: tauri::Window,
    state: State<'_, AppState>,
    item_id: String,
) -> Result<(), String> {
    let pool = get_pool(&window, &state)?;
    let repo_path = get_repo_path(&window, &state)?;

    let name: String = sqlx::query_scalar("SELECT name FROM items WHERE id = ?")
        .bind(&item_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Item not found: {}", e))?;

    ensure_item_folder(&repo_path, &item_id, &name)
}

#[tauri::command]
pub async fn item_has_folder(
    window: tauri::Window,
    state: State<'_, AppState>,
    item_id: String,
) -> Result<bool, String> {
    let repo_path = get_repo_path(&window, &state)?;
    let item_dir = Path::new(&repo_path).join(&item_id);
    Ok(item_dir.exists())
}

#[tauri::command]
pub async fn open_item_folder(
    window: tauri::Window,
    state: State<'_, AppState>,
    item_id: String,
) -> Result<(), String> {
    let pool = get_pool(&window, &state)?;
    let repo_path = get_repo_path(&window, &state)?;

    // Auto-create folder if it doesn't exist yet
    let item_dir = Path::new(&repo_path).join(&item_id);
    if !item_dir.exists() {
        let name: String = sqlx::query_scalar("SELECT name FROM items WHERE id = ?")
            .bind(&item_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Item not found: {}", e))?;
        ensure_item_folder(&repo_path, &item_id, &name)?;
    }

    open::that(item_dir.to_str().ok_or("Invalid path")?)
        .map_err(|e| format!("Cannot open folder: {}", e))
}

#[tauri::command]
pub async fn create_sub_repo(
    window: tauri::Window,
    state: State<'_, AppState>,
    item_id: String,
) -> Result<(), String> {
    let pool = get_pool(&window, &state)?;
    let repo_path = get_repo_path(&window, &state)?;
    let item_dir = Path::new(&repo_path).join(&item_id);
    let index_dir = item_dir.join(".index");

    if index_dir.exists() {
        return Err("Sub-repo already exists".to_string());
    }

    // Ensure item folder exists first (lazy-creation model)
    if !item_dir.exists() {
        let name: String = sqlx::query_scalar("SELECT name FROM items WHERE id = ?")
            .bind(&item_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Item not found: {}", e))?;
        ensure_item_folder(&repo_path, &item_id, &name)?;
    }

    std::fs::create_dir_all(&index_dir)
        .map_err(|e| format!("Cannot create .index: {}", e))?;

    let db_path = index_dir.join("index.db");
    let pool = crate::db::create_pool(&db_path)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    crate::db::run_migrations(&pool)
        .await
        .map_err(|e| format!("Migration error: {}", e))?;
    pool.close().await;

    let state_json = index_dir.join("state.json");
    std::fs::write(&state_json, r#"{"theme":"light"}"#)
        .map_err(|e| format!("Write error: {}", e))?;

    let workspaces_dir = index_dir.join("workspaces");
    std::fs::create_dir_all(&workspaces_dir)
        .map_err(|e| format!("Cannot create workspaces dir: {}", e))?;

    // Record in sub_repos.json
    let sub_repos_path = Path::new(&repo_path).join(".index").join("sub_repos.json");
    let mut map: serde_json::Map<String, serde_json::Value> = if sub_repos_path.exists() {
        let raw = std::fs::read_to_string(&sub_repos_path).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        serde_json::Map::new()
    };
    map.insert(item_id, serde_json::Value::String("active".to_string()));
    std::fs::write(&sub_repos_path, serde_json::to_string_pretty(&map).unwrap_or_default())
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn list_sub_repos(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let repo_path = get_repo_path(&window, &state)?;
    let sub_repos_path = Path::new(&repo_path).join(".index").join("sub_repos.json");

    if !sub_repos_path.exists() {
        return Ok(serde_json::Map::new());
    }

    let raw = std::fs::read_to_string(&sub_repos_path)
        .map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Parse error: {}", e))
}
