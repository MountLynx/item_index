use std::path::Path;
use tauri::State;
use crate::models::{WorkspaceConfig, WorkspaceSummary};
use crate::state::AppState;

fn get_repo_path(window: &tauri::Window, state: &State<'_, AppState>) -> Result<String, String> {
    let label = window.label().to_string();
    state.repos.lock().unwrap()
        .get(&label)
        .map(|r| r.path.clone())
        .ok_or("No repository open".to_string())
}

fn workspaces_dir(repo_path: &str) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(repo_path).join(".index").join("workspaces");
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
pub async fn list_workspaces(window: tauri::Window, state: State<'_, AppState>) -> Result<Vec<WorkspaceSummary>, String> {
    let repo_path = get_repo_path(&window, &state)?;
    let dir = workspaces_dir(&repo_path)?;

    // Read active workspace from state.json
    let state_json_path = Path::new(&repo_path).join(".index").join("state.json");
    let mut active: String = if state_json_path.exists() {
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
                let file_key = path.file_stem().unwrap().to_string_lossy().to_string();
                results.push(WorkspaceSummary {
                    is_default: file_key == active,
                    name: cfg.name,
                    key: file_key,
                    icon: cfg.icon,
                });
            }
        }
    }

    // Auto-create default workspace if empty
    if results.is_empty() {
        let cfg = default_workspace();
        let filename = format!("{}.json", &cfg.name);
        let p = dir.join(&filename);
        let json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&p, &json).map_err(|e| e.to_string())?;
        results.push(WorkspaceSummary {
            name: cfg.name.clone(),
            key: cfg.name,
            icon: "layout".into(),
            is_default: true,
        });
        // Update active so it matches the auto-created file
        active = results[0].key.clone();
    }

    // Sync state.json: if it doesn't have active_workspace, set it to the active one.
    // Also: when we just auto-created default and state.json didn't have the key
    // (e.g. older repo or fresh create_repo), persist it so the next open restores correctly.
    if state_json_path.exists() {
        let raw = std::fs::read_to_string(&state_json_path).unwrap_or_default();
        let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));
        if !v.get("active_workspace").is_some() {
            if let Some(map) = v.as_object_mut() {
                map.insert("active_workspace".to_string(), serde_json::Value::String(active.clone()));
            }
            let updated = serde_json::to_string_pretty(&v).unwrap_or_default();
            let _ = std::fs::write(&state_json_path, updated);
        }
    } else {
        // state.json shouldn't be missing in normal flow, but create it just in case
        let _ = std::fs::write(
            &state_json_path,
            serde_json::json!({"theme": "light", "active_workspace": active}).to_string(),
        );
    }

    Ok(results)
}

#[tauri::command]
pub async fn read_workspace(window: tauri::Window, state: State<'_, AppState>, name: String) -> Result<WorkspaceConfig, String> {
    let repo_path = get_repo_path(&window, &state)?;
    let path = workspaces_dir(&repo_path)?.join(format!("{}.json", name));
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("Workspace not found: {}", e))?;
    serde_json::from_str(&raw).map_err(|e| format!("Parse error: {}", e))
}

#[tauri::command]
pub async fn write_workspace(
    window: tauri::Window,
    state: State<'_, AppState>,
    name: String,
    config: WorkspaceConfig,
) -> Result<(), String> {
    let repo_path = get_repo_path(&window, &state)?;
    let path = workspaces_dir(&repo_path)?.join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_workspace(window: tauri::Window, state: State<'_, AppState>, name: String) -> Result<(), String> {
    // Prevent deleting the only workspace
    let repo_path = get_repo_path(&window, &state)?;
    let dir = workspaces_dir(&repo_path)?;
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
