use std::path::Path;
use tauri::State;
use crate::models::PluginManifest;
use crate::state::AppState;
use crate::refs;

fn get_repo_path(state: &State<'_, AppState>) -> Result<String, String> {
    state.repo_path.lock().unwrap().clone().ok_or("No repository open".to_string())
}

fn plugins_dir(state: &State<'_, AppState>) -> Result<std::path::PathBuf, String> {
    let path = Path::new(&get_repo_path(&state)?).join(".index").join("plugins");
    Ok(path)
}

#[tauri::command]
pub async fn list_installed_plugins(state: State<'_, AppState>) -> Result<Vec<PluginManifest>, String> {
    let dir = plugins_dir(&state)?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut manifests = vec![];
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let manifest_path = entry.path().join("manifest.json");
            if manifest_path.exists() {
                let raw = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
                match serde_json::from_str::<PluginManifest>(&raw) {
                    Ok(m) => {
                        // Enforce name === directory name
                        let dir_name = entry.file_name().to_string_lossy().to_string();
                        if m.name == dir_name {
                            manifests.push(m);
                        }
                    }
                    Err(_) => {} // Skip invalid manifests
                }
            }
        }
    }

    Ok(manifests)
}

#[tauri::command]
pub async fn read_plugin_file(
    state: State<'_, AppState>,
    plugin_name: String,
    filename: String,
) -> Result<String, String> {
    let dir = plugins_dir(&state)?;
    let file_path = dir.join(&plugin_name).join(&filename);

    // Security: ensure path is within plugins dir
    let canonical_dir = dir.canonicalize().map_err(|e| e.to_string())?;
    let canonical_file = file_path.canonicalize().map_err(|e| format!("File not found: {}", e))?;
    if !canonical_file.starts_with(&canonical_dir) {
        return Err("Path traversal detected".to_string());
    }

    std::fs::read_to_string(&canonical_file).map_err(|e| format!("Read error: {}", e))
}

#[tauri::command]
#[allow(unused_variables)]
pub async fn check_plugin_usage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    plugin_name: String,
) -> Result<crate::models::PluginUsage, String> {
    Ok(refs::get_usage(&state, &plugin_name))
}

/// Uninstall a plugin from the current repo.
/// Blocks if any workspace in this repo still references the plugin.
#[tauri::command]
pub async fn uninstall_plugin_from_repo(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    plugin_name: String,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;
    let workspaces_dir = std::path::Path::new(&repo_path)
        .join(".index").join("workspaces");

    // Check if any workspace references this plugin
    if workspaces_dir.exists() {
        for entry in std::fs::read_dir(&workspaces_dir).map_err(|e| e.to_string())?.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(raw) = std::fs::read_to_string(&path) {
                    let lower = raw.to_lowercase();
                    if lower.contains(&format!("\"plugin\":\"{}\"", plugin_name.to_lowercase()))
                        || lower.contains(&format!("\"plugin\": \"{}\"", plugin_name.to_lowercase()))
                    {
                        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                        return Err(format!(
                            "Plugin '{}' is still used by workspace '{}'. Remove it from the workspace first.",
                            plugin_name, stem
                        ));
                    }
                }
            }
        }
    }

    let dir = plugins_dir(&state)?.join(&plugin_name);
    if !dir.exists() {
        return Err(format!("Plugin '{}' is not installed in this repo", plugin_name));
    }
    std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;

    // Update refs
    refs::remove_repo_ref(&app, &state, &plugin_name, &repo_path)?;
    Ok(())
}
