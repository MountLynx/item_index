use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;
use crate::models::PluginUsage;

fn refs_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("plugin-store").join(".refs.json"))
}

/// Load the reference table from disk. Returns empty table if file doesn't exist.
pub fn load_refs(app: &tauri::AppHandle) -> Result<HashMap<String, PluginUsage>, String> {
    let path = refs_path(app)?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read .refs.json: {}", e))?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse .refs.json: {}", e))
}

/// Save the reference table to disk.
pub fn save_refs(app: &tauri::AppHandle, refs: &HashMap<String, PluginUsage>) -> Result<(), String> {
    let path = refs_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create plugin-store dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(refs)
        .map_err(|e| format!("Failed to serialize .refs.json: {}", e))?;
    std::fs::write(&path, &json)
        .map_err(|e| format!("Failed to write .refs.json: {}", e))
}

/// Add a repo path to a plugin's reference entry, creating the entry if needed.
pub fn add_repo_ref(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, crate::state::AppState>,
    plugin_name: &str,
    repo_path: &str,
) -> Result<(), String> {
    let mut refs = state.plugin_refs.lock().unwrap().clone();
    let entry = refs.entry(plugin_name.to_string()).or_default();
    if !entry.repos.iter().any(|r| r == repo_path) {
        entry.repos.push(repo_path.to_string());
    }
    save_refs(app, &refs)?;
    *state.plugin_refs.lock().unwrap() = refs;
    Ok(())
}

/// Remove a repo path from a plugin's reference entry.
pub fn remove_repo_ref(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, crate::state::AppState>,
    plugin_name: &str,
    repo_path: &str,
) -> Result<(), String> {
    let mut refs = state.plugin_refs.lock().unwrap().clone();
    if let Some(entry) = refs.get_mut(plugin_name) {
        entry.repos.retain(|r| r != repo_path);
    }
    save_refs(app, &refs)?;
    *state.plugin_refs.lock().unwrap() = refs;
    Ok(())
}

/// Add a preset name to a plugin's reference entry.
pub fn add_preset_ref(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, crate::state::AppState>,
    plugin_name: &str,
    preset_name: &str,
) -> Result<(), String> {
    let mut refs = state.plugin_refs.lock().unwrap().clone();
    let entry = refs.entry(plugin_name.to_string()).or_default();
    if !entry.presets.iter().any(|p| p == preset_name) {
        entry.presets.push(preset_name.to_string());
    }
    save_refs(app, &refs)?;
    *state.plugin_refs.lock().unwrap() = refs;
    Ok(())
}

/// Get the usage information for a plugin.
pub fn get_usage(
    state: &tauri::State<'_, crate::state::AppState>,
    plugin_name: &str,
) -> PluginUsage {
    state.plugin_refs.lock().unwrap()
        .get(plugin_name)
        .cloned()
        .unwrap_or_default()
}

/// Remove all references for a given repo path (called when repo is closed/deleted).
pub fn cleanup_repo(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, crate::state::AppState>,
    repo_path: &str,
) -> Result<(), String> {
    let mut refs = state.plugin_refs.lock().unwrap().clone();
    for entry in refs.values_mut() {
        entry.repos.retain(|r| r != repo_path);
    }
    save_refs(app, &refs)?;
    *state.plugin_refs.lock().unwrap() = refs;
    Ok(())
}
