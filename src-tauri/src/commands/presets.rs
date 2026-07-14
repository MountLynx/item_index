use std::path::Path;
use tauri::{AppHandle, Manager, State};
use crate::models::{PresetSummary, WorkspaceConfig};
use crate::state::AppState;

fn get_repo_path(state: &State<'_, AppState>) -> Result<String, String> {
    state.repo_path.lock().unwrap().clone().ok_or("No repository open".to_string())
}

fn presets_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?
        .join("workspace-presets");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn repo_plugins_dir(state: &State<'_, AppState>) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(&get_repo_path(state)?).join(".index").join("plugins");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn repo_workspaces_dir(state: &State<'_, AppState>) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(&get_repo_path(state)?).join(".index").join("workspaces");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn global_plugin_store(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?
        .join("plugin-store");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
pub async fn list_workspace_presets(app: AppHandle) -> Result<Vec<PresetSummary>, String> {
    let dir = presets_dir(&app)?;
    let mut results = vec![];
    if !dir.exists() { return Ok(results); }

    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(cfg) = serde_json::from_str::<WorkspaceConfig>(&raw) {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
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
    // 1. Read preset config
    let preset_path = presets_dir(&app)?.join(format!("{}.json", preset_name));
    let raw = std::fs::read_to_string(&preset_path)
        .map_err(|e| format!("Preset not found: {}", e))?;
    let cfg: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let workspace_cfg: WorkspaceConfig = serde_json::from_value(cfg.clone()).map_err(|e| e.to_string())?;

    // 2. Copy bundled plugins from global plugin-store to repo plugins
    let bundle = cfg.get("bundle");
    if let Some(b) = bundle {
        if let Some(plugins) = b.get("plugins").and_then(|v| v.as_array()) {
            let global_store = global_plugin_store(&app)?;
            let repo_plugins = repo_plugins_dir(&state)?;
            for p in plugins {
                let plugin_name = p.as_str().unwrap_or("");
                if plugin_name.is_empty() { continue; }
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
    let json = serde_json::to_string_pretty(&workspace_cfg).map_err(|e| e.to_string())?;
    std::fs::write(&ws_path, &json).map_err(|e| e.to_string())?;

    Ok(workspace_cfg)
}

#[tauri::command]
pub async fn export_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    // Read workspace config
    let ws_dir = repo_workspaces_dir(&state)?;
    let ws_path = ws_dir.join(format!("{}.json", name));
    let raw = std::fs::read_to_string(&ws_path).map_err(|e| format!("Workspace not found: {}", e))?;

    // Write to presets dir
    let preset_path = presets_dir(&app)?.join(format!("{}.json", name));
    // Preserve existing bundle if re-exporting
    let mut cfg: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if !cfg.as_object().map_or(false, |o| o.contains_key("bundle")) {
        if let serde_json::Value::Object(ref mut map) = cfg {
            map.insert("bundle".to_string(), serde_json::json!({"plugins": [], "itemTypes": []}));
        }
    }

    let json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(&preset_path, &json).map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
