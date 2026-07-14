use std::path::Path;
use tauri::{AppHandle, Manager, State};
use crate::models::{PresetSummary, WorkspaceConfig};
use crate::state::AppState;

/// Validates that a plugin or preset name contains only safe characters
/// (alphanumeric, hyphens, underscores) and no path separators or `..`.
fn is_safe_plugin_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

fn get_repo_path(state: &State<'_, AppState>) -> Result<String, String> {
    state.repo_path.lock().unwrap().clone().ok_or("No repository open".to_string())
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

fn repo_plugins_dir(state: &State<'_, AppState>) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(&get_repo_path(state)?).join(".index").join("plugins");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create repo plugins directory '{}': {}", dir.display(), e))?;
    Ok(dir)
}

fn repo_workspaces_dir(state: &State<'_, AppState>) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(&get_repo_path(state)?).join(".index").join("workspaces");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create repo workspaces directory '{}': {}", dir.display(), e))?;
    Ok(dir)
}

/// Returns the global plugin store path WITHOUT creating it (read path).
fn global_plugin_store(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
        .map(|d| d.join("plugin-store"))
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

    // 2. Copy bundled plugins from global plugin-store to repo plugins
    let bundle = cfg.get("bundle");
    if let Some(b) = bundle {
        if let Some(plugins) = b.get("plugins").and_then(|v| v.as_array()) {
            let global_store = global_plugin_store(&app)?;
            let repo_plugins = repo_plugins_dir(&state)?;
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

    // 3. Write workspace config to repo
    let ws_path = repo_workspaces_dir(&state)?.join(format!("{}.json", preset_name));
    let json = serde_json::to_string_pretty(&workspace_cfg)
        .map_err(|e| format!("Failed to serialize workspace config: {}", e))?;
    std::fs::write(&ws_path, &json)
        .map_err(|e| format!("Failed to write workspace config to '{}': {}", ws_path.display(), e))?;

    Ok(workspace_cfg)
}

#[tauri::command]
pub async fn export_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    // H2: Validate name parameter to prevent path traversal
    if !is_safe_plugin_name(&name) {
        return Err(format!("Invalid preset name: '{}'", name));
    }

    // Read workspace config
    let ws_dir = repo_workspaces_dir(&state)?;
    let ws_path = ws_dir.join(format!("{}.json", name));
    let raw = std::fs::read_to_string(&ws_path)
        .map_err(|e| format!("Workspace not found at '{}': {}", ws_path.display(), e))?;

    // Write to presets dir
    let preset_path = presets_dir(&app)?.join(format!("{}.json", name));
    // Preserve existing bundle if re-exporting
    let mut cfg: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse workspace config '{}': {}", ws_path.display(), e))?;
    if !cfg.as_object().map_or(false, |o| o.contains_key("bundle")) {
        if let serde_json::Value::Object(ref mut map) = cfg {
            map.insert("bundle".to_string(), serde_json::json!({"plugins": [], "itemTypes": []}));
        }
    }

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
