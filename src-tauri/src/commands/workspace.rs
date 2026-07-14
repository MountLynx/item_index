use std::path::Path;
use tauri::State;
use crate::models::{WorkspaceConfig, WorkspaceSummary};
use crate::state::AppState;

fn get_repo_path(state: &State<'_, AppState>) -> Result<String, String> {
    state.repo_path.lock().unwrap().clone().ok_or("No repository open".to_string())
}

fn workspaces_dir(state: &State<'_, AppState>) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(&get_repo_path(state)?).join(".index").join("workspaces");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn default_workspace() -> WorkspaceConfig {
    WorkspaceConfig {
        name: "默认工作区".to_string(),
        icon: "layout".to_string(),
        item_types: vec![],
        center_tabs: vec![crate::models::CenterTab {
            tab_type: "list".to_string(),
            label: "列表".to_string(),
            icon: Some("list".to_string()),
            plugin: None,
            config: None,
        }],
        default_tab: "list".to_string(),
        right_panel_addons: vec![],
        sidebar_addons: vec![],
    }
}

#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<WorkspaceSummary>, String> {
    let dir = workspaces_dir(&state)?;

    // Read active workspace from state.json
    let state_json_path = Path::new(&get_repo_path(&state)?).join(".index").join("state.json");
    let active: String = if state_json_path.exists() {
        let raw = std::fs::read_to_string(&state_json_path).map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        v.get("active_workspace").and_then(|v| v.as_str()).unwrap_or("default").to_string()
    } else {
        "default".to_string()
    };

    let mut results = vec![];
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(cfg) = serde_json::from_str::<WorkspaceConfig>(&raw) {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                results.push(WorkspaceSummary {
                    is_default: name == active,
                    name,
                    icon: cfg.icon,
                });
            }
        }
    }

    // Auto-create default workspace if empty
    if results.is_empty() {
        let cfg = default_workspace();
        let p = dir.join("default.json");
        let json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&p, &json).map_err(|e| e.to_string())?;
        results.push(WorkspaceSummary { name: "default".into(), icon: "layout".into(), is_default: true });
    }

    Ok(results)
}

#[tauri::command]
pub async fn read_workspace(state: State<'_, AppState>, name: String) -> Result<WorkspaceConfig, String> {
    let path = workspaces_dir(&state)?.join(format!("{}.json", name));
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("Workspace not found: {}", e))?;
    serde_json::from_str(&raw).map_err(|e| format!("Parse error: {}", e))
}

#[tauri::command]
pub async fn write_workspace(
    state: State<'_, AppState>,
    name: String,
    config: WorkspaceConfig,
) -> Result<(), String> {
    let path = workspaces_dir(&state)?.join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_workspace(state: State<'_, AppState>, name: String) -> Result<(), String> {
    // Prevent deleting the only workspace
    let dir = workspaces_dir(&state)?;
    let count = std::fs::read_dir(&dir).map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .count();
    if count <= 1 {
        return Err("Cannot delete the last workspace".to_string());
    }

    let path = dir.join(format!("{}.json", name));
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}
