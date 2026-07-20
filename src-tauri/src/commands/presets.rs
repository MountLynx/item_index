use std::path::Path;
use tauri::{AppHandle, Manager, State};
use crate::models::{PresetSummary, WorkspaceConfig, PluginManifest};
use crate::refs;
use crate::state::AppState;

fn get_pool(window: &tauri::Window, state: &State<'_, AppState>) -> Result<sqlx::SqlitePool, String> {
    let label = window.label().to_string();
    state.repos.lock().unwrap()
        .get(&label)
        .map(|r| r.db.clone())
        .ok_or("No repository open".to_string())
}

/// Validates that a plugin or preset name contains only safe characters
/// (alphanumeric, hyphens, underscores) and no path separators or `..`.
fn is_safe_plugin_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

fn get_repo_path(window: &tauri::Window, state: &State<'_, AppState>) -> Result<String, String> {
    let label = window.label().to_string();
    state.repos.lock().unwrap()
        .get(&label)
        .map(|r| r.path.clone())
        .ok_or("No repository open".to_string())
}

/// Returns the presets directory path WITHOUT creating it (read path).
fn presets_dir_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
        .map(|d| d.join("workspace-presets"))
}

/// Returns the presets directory path, creating it if necessary (write path).
fn presets_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = presets_dir_path(app)?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create presets directory '{}': {}", dir.display(), e))?;
    Ok(dir)
}

fn repo_plugins_dir(repo_path: &str) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(repo_path).join(".index").join("plugins");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create repo plugins directory '{}': {}", dir.display(), e))?;
    Ok(dir)
}

fn repo_workspaces_dir(repo_path: &str) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(repo_path).join(".index").join("workspaces");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create repo workspaces directory '{}': {}", dir.display(), e))?;
    Ok(dir)
}

/// Returns the global plugin store path WITHOUT creating it (read path).
fn global_plugin_store(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
        .map(|d| d.join("plugin-store"))
}

/// List manifests in the global plugin store (available plugins).
#[tauri::command]
pub async fn list_global_plugins(app: AppHandle) -> Result<Vec<PluginManifest>, String> {
    let dir = global_plugin_store(&app)?;
    let mut results = vec![];
    if !dir.exists() { return Ok(results); }
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() { continue; }
        let manifest_path = entry_path.join("manifest.json");
        if !manifest_path.exists() { continue; }
        match std::fs::read_to_string(&manifest_path)
            .map_err(|e| e.to_string())
            .and_then(|raw| serde_json::from_str::<PluginManifest>(&raw).map_err(|e| e.to_string()))
        {
            Ok(m) => {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if m.name == dir_name {
                    results.push(m);
                }
            }
            Err(_) => {}
        }
    }
    Ok(results)
}

/// Install (copy) a plugin from the global store into the current repo.
#[tauri::command]
pub async fn install_plugin(
    window: tauri::Window,
    app: AppHandle,
    state: State<'_, AppState>,
    plugin_name: String,
) -> Result<(), String> {
    if !is_safe_plugin_name(&plugin_name) {
        return Err(format!("Invalid plugin name: '{}'", plugin_name));
    }
    let src = global_plugin_store(&app)?.join(&plugin_name);
    if !src.exists() {
        return Err(format!("Plugin '{}' not found in global store", plugin_name));
    }
    let repo_path = get_repo_path(&window, &state)?;
    let dst = repo_plugins_dir(&repo_path)?.join(&plugin_name);
    if dst.exists() {
        return Err(format!("Plugin '{}' is already installed in this repo", plugin_name));
    }
    copy_dir(&src, &dst)?;
    refs::add_repo_ref(&app, &state, &plugin_name, &repo_path)?;
    Ok(())
}

/// Import a plugin folder into the global store.
#[tauri::command]
pub async fn install_plugin_to_global(
    app: AppHandle,
    source_path: String,
) -> Result<(), String> {
    let src = Path::new(&source_path);
    if !src.is_dir() {
        return Err("Source is not a directory".to_string());
    }
    let manifest_path = src.join("manifest.json");
    if !manifest_path.exists() {
        return Err("No manifest.json found in plugin folder".to_string());
    }
    let raw = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Cannot read manifest: {}", e))?;
    let manifest: PluginManifest = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid manifest: {}", e))?;
    if !is_safe_plugin_name(&manifest.name) {
        return Err(format!("Invalid plugin name: '{}'", manifest.name));
    }
    // Enforce name == directory name
    let dir_name = src.file_name().unwrap_or_default().to_string_lossy();
    if manifest.name != dir_name {
        return Err(format!("Plugin name '{}' != directory name '{}'", manifest.name, dir_name));
    }
    let dst = {
        let dir = global_plugin_store_write(&app)?;
        let d = dir.join(&manifest.name);
        if d.exists() {
            return Err(format!("Plugin '{}' already exists in global store", manifest.name));
        }
        d
    };
    copy_dir(src, &dst)?;
    // Initialize an empty reference entry for the new plugin
    {
        let app_state = app.state::<AppState>();
        let mut refs = app_state.plugin_refs.lock().unwrap().clone();
        refs.entry(manifest.name.clone()).or_default();
        refs::save_refs(&app, &refs)?;
        *app_state.plugin_refs.lock().unwrap() = refs;
    }
    Ok(())
}

/// Returns the global plugin store path, creating it if necessary.
fn global_plugin_store_write(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = global_plugin_store(app)?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Delete a plugin from the global store.
#[tauri::command]
pub async fn delete_plugin(
    app: AppHandle,
    state: State<'_, AppState>,
    plugin_name: String,
) -> Result<(), String> {
    if !is_safe_plugin_name(&plugin_name) {
        return Err(format!("Invalid plugin name: '{}'", plugin_name));
    }
    // Check usage before deleting
    let usage = refs::get_usage(&state, &plugin_name);
    if !usage.repos.is_empty() || !usage.presets.is_empty() {
        return Err(format!(
            "Plugin '{}' is still in use.\nRepos: {:?}\nPresets: {:?}",
            plugin_name, usage.repos, usage.presets
        ));
    }
    let dir = global_plugin_store(&app)?.join(&plugin_name);
    if !dir.exists() {
        return Err(format!("Plugin '{}' not found in global store", plugin_name));
    }
    std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    // Remove ref entry
    {
        let mut refs = state.plugin_refs.lock().unwrap().clone();
        refs.remove(&plugin_name);
        refs::save_refs(&app, &refs)?;
        *state.plugin_refs.lock().unwrap() = refs;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_workspace_presets(app: AppHandle) -> Result<Vec<PresetSummary>, String> {
    let dir = presets_dir_path(&app)?;
    let mut results = vec![];
    if !dir.exists() { return Ok(results); }

    let entries = std::fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read presets directory '{}': {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry in '{}': {}", dir.display(), e))?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let raw = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
            if let Ok(cfg) = serde_json::from_str::<WorkspaceConfig>(&raw) {
                let name = match path.file_stem() {
                    Some(stem) => stem.to_string_lossy().to_string(),
                    None => {
                        eprintln!("Warning: skipping file '{}' with no file stem", path.display());
                        continue;
                    }
                };
                results.push(PresetSummary {
                    name,
                    icon: cfg.icon,
                    description: cfg.name,
                });
            }
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn install_preset(
    window: tauri::Window,
    app: AppHandle,
    state: State<'_, AppState>,
    preset_name: String,
) -> Result<WorkspaceConfig, String> {
    // Validate preset name to prevent path traversal (H2-style check on user input)
    if !is_safe_plugin_name(&preset_name) {
        return Err(format!("Invalid preset name: '{}'", preset_name));
    }

    // 1. Read preset config
    let preset_path = presets_dir(&app)?.join(format!("{}.json", preset_name));
    let raw = std::fs::read_to_string(&preset_path)
        .map_err(|e| format!("Preset not found at '{}': {}", preset_path.display(), e))?;
    let cfg: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse preset '{}': {}", preset_path.display(), e))?;
    let workspace_cfg: WorkspaceConfig = serde_json::from_value(cfg.clone())
        .map_err(|e| format!("Failed to deserialize preset '{}': {}", preset_path.display(), e))?;

    let repo_path = get_repo_path(&window, &state)?;

    // 2. Copy bundled plugins from global plugin-store to repo plugins
    let bundle = cfg.get("bundle");
    if let Some(b) = bundle {
        if let Some(plugins) = b.get("plugins").and_then(|v| v.as_array()) {
            let global_store = global_plugin_store(&app)?;
            let repo_plugins = repo_plugins_dir(&repo_path)?;
            for p in plugins {
                let plugin_name = p.as_str().unwrap_or("");
                if plugin_name.is_empty() { continue; }

                // H1: Validate plugin name to prevent path traversal
                if !is_safe_plugin_name(plugin_name) {
                    return Err(format!("Invalid plugin name in bundle: '{}'", plugin_name));
                }

                let src = global_store.join(plugin_name);
                let dst = repo_plugins.join(plugin_name);
                if src.exists() && !dst.exists() {
                    copy_dir(&src, &dst)?;
                }
            }
        }
    }

    // 2.5 Create item types from bundle (insert or ignore if name exists)
    if let Some(b) = bundle {
        if let Some(item_types) = b.get("itemTypes").and_then(|v| v.as_array()) {
            let pool = get_pool(&window, &state)?;
            for t in item_types {
                let type_name = t["name"].as_str().unwrap_or("");
                let type_icon = t["icon"].as_str().unwrap_or("file");
                if type_name.is_empty() { continue; }

                // INSERT OR IGNORE — skip if type name already exists
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO item_types (name, icon, namespace) VALUES (?, ?, 'default')"
                ).bind(type_name).bind(type_icon).execute(&pool).await;

                // Get type id (newly inserted or already existing)
                let type_id: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM item_types WHERE name = ?"
                ).bind(type_name).fetch_optional(&pool).await.map_err(|e| e.to_string())?;

                if let Some(tid) = type_id {
                    if let Some(fields) = t.get("fields").and_then(|v| v.as_array()) {
                        for f in fields {
                            let fname = f["name"].as_str().unwrap_or("");
                            let ftype = f["field_type"].as_str().unwrap_or("text");
                            let ficon = f["icon"].as_str().unwrap_or("circle");
                            let flabel = f["label"].as_str().unwrap_or("");
                            if fname.is_empty() { continue; }

                            let max_pos: Option<i64> = sqlx::query_scalar(
                                "SELECT MAX(position) FROM fields WHERE type_id = ?"
                            ).bind(tid).fetch_one(&pool).await.map_err(|e| e.to_string())?;

                            let pos = max_pos.unwrap_or(-1) + 1;
                            let _ = sqlx::query(
                                "INSERT OR IGNORE INTO fields (type_id, name, field_type, icon, position, label) VALUES (?, ?, ?, ?, ?, ?)"
                            ).bind(tid).bind(fname).bind(ftype).bind(ficon).bind(pos).bind(flabel)
                                .execute(&pool).await;
                        }
                    }
                }
            }
        }
    }

    // 3. Write workspace config to repo (use config.name as filename, not preset_name)
    let ws_path = repo_workspaces_dir(&repo_path)?.join(format!("{}.json", workspace_cfg.name));
    let json = serde_json::to_string_pretty(&workspace_cfg)
        .map_err(|e| format!("Failed to serialize workspace config: {}", e))?;
    std::fs::write(&ws_path, &json)
        .map_err(|e| format!("Failed to write workspace config to '{}': {}", ws_path.display(), e))?;

    Ok(workspace_cfg)
}

#[tauri::command]
pub async fn export_preset(
    window: tauri::Window,
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    // H2: Validate name parameter to prevent path traversal
    if !is_safe_plugin_name(&name) {
        return Err(format!("Invalid preset name: '{}'", name));
    }

    let repo_path = get_repo_path(&window, &state)?;

    // Read workspace config
    let ws_dir = repo_workspaces_dir(&repo_path)?;
    let ws_path = ws_dir.join(format!("{}.json", name));
    let raw = std::fs::read_to_string(&ws_path)
        .map_err(|e| format!("Workspace not found at '{}': {}", ws_path.display(), e))?;
    let mut cfg: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse workspace config '{}': {}", ws_path.display(), e))?;

    // Scan installed plugins from repo plugins dir
    let plugin_names: Vec<String> = {
        let plugins_dir = repo_plugins_dir(&repo_path)?;
        if plugins_dir.exists() {
            let mut names = vec![];
            if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        if entry.path().join("manifest.json").exists() {
                            names.push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
            }
            names
        } else {
            vec![]
        }
    };

    // Update preset refs for each plugin
    for pname in &plugin_names {
        refs::add_preset_ref(&app, &state, pname, &name)?;
    }

    // Scan item types from DB
    let pool = get_pool(&window, &state)?;
    let item_types_raw: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, name, icon FROM item_types ORDER BY id"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut item_types_json: Vec<serde_json::Value> = vec![];
    for (tid, tname, ticon) in &item_types_raw {
        let fields: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT name, field_type, icon, label FROM fields WHERE type_id = ? ORDER BY position"
        )
        .bind(tid)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

        let fields_json: Vec<serde_json::Value> = fields
            .into_iter()
            .map(|(fname, ftype, ficon, flabel)| {
                serde_json::json!({
                    "name": fname,
                    "field_type": ftype,
                    "icon": ficon,
                    "label": flabel,
                })
            })
            .collect();

        item_types_json.push(serde_json::json!({
            "name": tname,
            "icon": ticon,
            "fields": fields_json,
        }));
    }

    // Build bundle with actual data (Bug #6)
    let bundle = serde_json::json!({
        "plugins": plugin_names,
        "itemTypes": item_types_json,
    });

    if let serde_json::Value::Object(ref mut map) = cfg {
        map.insert("bundle".to_string(), bundle);
    }

    // Write to presets dir
    let preset_path = presets_dir(&app)?.join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(&cfg)
        .map_err(|e| format!("Failed to serialize preset: {}", e))?;
    std::fs::write(&preset_path, &json)
        .map_err(|e| format!("Failed to write preset to '{}': {}", preset_path.display(), e))?;
    Ok(())
}

/// Recursively copies a directory.
///
/// Security:
/// - Canonicalizes the source path to detect symlink-based traversal in the source tree.
/// - Skips symlinks during iteration to prevent symlink-traversal attacks.
/// - Verifies each destination path stays within the target directory (defense-in-depth).
fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    // Canonicalize source to resolve any symlinks in the source path itself
    let canonical_src = std::fs::canonicalize(src)
        .map_err(|e| format!("Failed to resolve source path '{}': {}", src.display(), e))?;

    std::fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create destination directory '{}': {}", dst.display(), e))?;

    for entry in std::fs::read_dir(&canonical_src)
        .map_err(|e| format!("Failed to read directory '{}': {}", canonical_src.display(), e))?
    {
        let entry = entry
            .map_err(|e| format!("Failed to read entry in '{}': {}", canonical_src.display(), e))?;
        let file_type = entry.file_type()
            .map_err(|e| format!("Failed to get file type for '{}': {}", entry.path().display(), e))?;

        // Skip symlinks to prevent symlink-traversal attacks
        if file_type.is_symlink() {
            eprintln!("Warning: skipping symlink '{}'", entry.path().display());
            continue;
        }

        let dst_path = dst.join(entry.file_name());

        // Defense-in-depth: verify destination is within the target directory
        if !dst_path.starts_with(dst) {
            return Err(format!(
                "Path traversal blocked: '{}' is outside destination '{}'",
                dst_path.display(),
                dst.display()
            ));
        }

        if file_type.is_dir() {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(&entry.path(), &dst_path)
                .map_err(|e| format!("Failed to copy '{}' to '{}': {}", entry.path().display(), dst_path.display(), e))?;
        }
    }
    Ok(())
}
