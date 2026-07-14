use std::path::Path;
use sha2::{Sha256, Digest};
use rand::Rng;
use tauri::State;
use sqlx::SqlitePool;
use crate::models::{Item, ItemDetail, ItemType, Field, Group, Tag, FileNode};
use crate::state::AppState;

fn get_pool(state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    state.db.lock().unwrap().clone().ok_or("No repository open".to_string())
}

fn get_repo_path(state: &State<'_, AppState>) -> Result<String, String> {
    state.repo_path.lock().unwrap().clone().ok_or("No repository open".to_string())
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

    let field_rows: Vec<(i64, i64, String, String, String, i32, String)> = sqlx::query_as(
        "SELECT id, type_id, name, field_type, icon, position, label FROM fields WHERE type_id = ? ORDER BY position"
    ).bind(id).fetch_all(pool).await.map_err(|e| e.to_string())?;

    let fields: Vec<Field> = field_rows.into_iter().map(|(fid, tid, n, ft, ficon, pos, label)| Field {
        id: fid, type_id: tid, name: n, field_type: ft, icon: ficon, position: pos, label,
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

#[tauri::command]
pub async fn create_item(
    state: State<'_, AppState>,
    type_id: i64,
    name: String,
) -> Result<Item, String> {
    let pool = get_pool(&state)?;
    let repo_path = get_repo_path(&state)?;
    let id = generate_id();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO items (id, name, type_id, properties, namespace, created_at, updated_at)
         VALUES (?, ?, ?, '{}', 'default', ?, ?)"
    ).bind(&id).bind(&name).bind(type_id).bind(&now).bind(&now)
        .execute(&pool).await.map_err(|e| e.to_string())?;

    // Create hash folder + auto-generate <name>.md
    let item_dir = Path::new(&repo_path).join(&id);
    std::fs::create_dir_all(&item_dir).map_err(|e| format!("Cannot create item folder: {}", e))?;
    let md_content = format!("# {}\n", name);
    std::fs::write(item_dir.join(format!("{}.md", name)), md_content)
        .map_err(|e| format!("Cannot create .md file: {}", e))?;

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
    state: State<'_, AppState>,
    id: String,
) -> Result<ItemDetail, String> {
    let pool = get_pool(&state)?;
    let repo_path = get_repo_path(&state)?;

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
    state: State<'_, AppState>,
    group_id: Option<i64>,
    tag_id: Option<i64>,
    type_ids: Option<Vec<i64>>,
) -> Result<Vec<Item>, String> {
    let pool = get_pool(&state)?;

    let rows: Vec<(String, String, i64, String, String, String, String)> = {
        let tid_filter = type_ids.clone();

        if let Some(gid) = group_id {
            if let Some(tid) = tag_id {
                if let Some(ref tids) = tid_filter {
                    if !tids.is_empty() {
                        // group + tag + type
                        let placeholders: Vec<String> = tids.iter().enumerate().map(|(i, _)| format!("?{}", i + 3)).collect();
                        let query = format!(
                            "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                             FROM items i
                             INNER JOIN item_groups ig ON i.id = ig.item_id
                             INNER JOIN item_tags it ON i.id = it.item_id
                             WHERE ig.group_id = ?1 AND it.tag_id = ?2 AND i.type_id IN ({})
                             ORDER BY i.updated_at DESC",
                            placeholders.join(",")
                        );
                        let mut q = sqlx::query_as(&query).bind(gid).bind(tid);
                        for tid_val in tids { q = q.bind(tid_val); }
                        q.fetch_all(&pool).await
                    } else {
                        sqlx::query_as(
                            "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                             FROM items i INNER JOIN item_groups ig ON i.id = ig.item_id INNER JOIN item_tags it ON i.id = it.item_id
                             WHERE ig.group_id = ? AND it.tag_id = ? ORDER BY i.updated_at DESC"
                        ).bind(gid).bind(tid).fetch_all(&pool).await
                    }
                } else {
                    sqlx::query_as(
                        "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                         FROM items i INNER JOIN item_groups ig ON i.id = ig.item_id INNER JOIN item_tags it ON i.id = it.item_id
                         WHERE ig.group_id = ? AND it.tag_id = ? ORDER BY i.updated_at DESC"
                    ).bind(gid).bind(tid).fetch_all(&pool).await
                }
            } else {
                // group only, optionally + type
                if let Some(ref tids) = tid_filter {
                    if !tids.is_empty() {
                        let placeholders: Vec<String> = tids.iter().enumerate().map(|(i, _)| format!("?{}", i + 2)).collect();
                        let query = format!(
                            "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                             FROM items i INNER JOIN item_groups ig ON i.id = ig.item_id
                             WHERE ig.group_id = ?1 AND i.type_id IN ({}) ORDER BY i.updated_at DESC",
                            placeholders.join(",")
                        );
                        let mut q = sqlx::query_as(&query).bind(gid);
                        for tid_val in tids { q = q.bind(tid_val); }
                        q.fetch_all(&pool).await
                    } else {
                        sqlx::query_as(
                            "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                             FROM items i INNER JOIN item_groups ig ON i.id = ig.item_id
                             WHERE ig.group_id = ? ORDER BY i.updated_at DESC"
                        ).bind(gid).fetch_all(&pool).await
                    }
                } else {
                    sqlx::query_as(
                        "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                         FROM items i INNER JOIN item_groups ig ON i.id = ig.item_id
                         WHERE ig.group_id = ? ORDER BY i.updated_at DESC"
                    ).bind(gid).fetch_all(&pool).await
                }
            }
        } else if let Some(tid) = tag_id {
            // tag only, optionally + type
            if let Some(ref tids) = tid_filter {
                if !tids.is_empty() {
                    let placeholders: Vec<String> = tids.iter().enumerate().map(|(i, _)| format!("?{}", i + 2)).collect();
                    let query = format!(
                        "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                         FROM items i INNER JOIN item_tags it ON i.id = it.item_id
                         WHERE it.tag_id = ?1 AND i.type_id IN ({}) ORDER BY i.updated_at DESC",
                        placeholders.join(",")
                    );
                    let mut q = sqlx::query_as(&query).bind(tid);
                    for tid_val in tids { q = q.bind(tid_val); }
                    q.fetch_all(&pool).await
                } else {
                    sqlx::query_as(
                        "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                         FROM items i INNER JOIN item_tags it ON i.id = it.item_id
                         WHERE it.tag_id = ? ORDER BY i.updated_at DESC"
                    ).bind(tid).fetch_all(&pool).await
                }
            } else {
                sqlx::query_as(
                    "SELECT DISTINCT i.id, i.name, i.type_id, i.properties, i.namespace, i.created_at, i.updated_at
                     FROM items i INNER JOIN item_tags it ON i.id = it.item_id
                     WHERE it.tag_id = ? ORDER BY i.updated_at DESC"
                ).bind(tid).fetch_all(&pool).await
            }
        } else {
            // type only or everything
            if let Some(ref tids) = tid_filter {
                if !tids.is_empty() {
                    let placeholders: Vec<String> = tids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
                    let query = format!(
                        "SELECT id, name, type_id, properties, namespace, created_at, updated_at
                         FROM items WHERE type_id IN ({}) ORDER BY updated_at DESC",
                        placeholders.join(",")
                    );
                    let mut q = sqlx::query_as(&query);
                    for tid_val in tids { q = q.bind(tid_val); }
                    q.fetch_all(&pool).await
                } else {
                    sqlx::query_as(
                        "SELECT id, name, type_id, properties, namespace, created_at, updated_at
                         FROM items ORDER BY updated_at DESC"
                    ).fetch_all(&pool).await
                }
            } else {
                sqlx::query_as(
                    "SELECT id, name, type_id, properties, namespace, created_at, updated_at
                     FROM items ORDER BY updated_at DESC"
                ).fetch_all(&pool).await
            }
        }
    }.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name, type_id, props_str, namespace, created_at, updated_at)| {
        let properties: serde_json::Value = serde_json::from_str(&props_str).unwrap_or_default();
        Item { id, name, type_id, properties, namespace, created_at, updated_at }
    }).collect())
}

#[tauri::command]
pub async fn update_item(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    properties: Option<serde_json::Value>,
) -> Result<Item, String> {
    let pool = get_pool(&state)?;
    let now = chrono::Utc::now().to_rfc3339();

    // Build dynamic UPDATE
    if let Some(ref n) = name {
        sqlx::query("UPDATE items SET name = ?, updated_at = ? WHERE id = ?")
            .bind(n).bind(&now).bind(&id)
            .execute(&pool).await.map_err(|e| e.to_string())?;
    }
    if let Some(ref p) = properties {
        let props_str = serde_json::to_string(p).map_err(|e| e.to_string())?;
        sqlx::query("UPDATE items SET properties = ?, updated_at = ? WHERE id = ?")
            .bind(&props_str).bind(&now).bind(&id)
            .execute(&pool).await.map_err(|e| e.to_string())?;
    }
    // Always update updated_at if anything changed
    if name.is_some() || properties.is_some() {
        // already done above
    } else {
        return Err("Nothing to update".to_string());
    }

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
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let pool = get_pool(&state)?;
    let repo_path = get_repo_path(&state)?;

    // Delete hash folder
    let item_dir = Path::new(&repo_path).join(&id);
    if item_dir.exists() {
        std::fs::remove_dir_all(&item_dir)
            .map_err(|e| format!("Cannot delete item folder: {}", e))?;
    }

    // CASCADE handles junction tables
    sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
