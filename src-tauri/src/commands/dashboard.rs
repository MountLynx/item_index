use tauri::Manager;
use crate::models::ManagedRepo;

fn repos_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("repos.json"))
}

fn read(app: &tauri::AppHandle) -> Result<Vec<ManagedRepo>, String> {
    let p = repos_path(app)?;
    if !p.exists() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn write(app: &tauri::AppHandle, repos: &[ManagedRepo]) -> Result<(), String> {
    let p = repos_path(app)?;
    let json = serde_json::to_string_pretty(repos).map_err(|e| e.to_string())?;
    std::fs::write(&p, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_managed_repos(app: tauri::AppHandle) -> Result<Vec<ManagedRepo>, String> {
    read(&app)
}

#[tauri::command]
pub async fn add_managed_repo(
    app: tauri::AppHandle,
    path: String,
    icon: Option<String>,
    name: Option<String>,
    item_count: Option<i64>,
) -> Result<Vec<ManagedRepo>, String> {
    let mut repos = read(&app)?;
    let now = chrono::Utc::now().to_rfc3339();

    // Upsert: remove existing entry with same path
    repos.retain(|r| r.path != path);

    repos.push(ManagedRepo {
        path,
        icon,
        name,
        last_opened_at: now,
        item_count,
    });

    write(&app, &repos)?;
    Ok(repos)
}

#[tauri::command]
pub async fn remove_managed_repo(
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<ManagedRepo>, String> {
    let mut repos = read(&app)?;
    repos.retain(|r| r.path != path);
    write(&app, &repos)?;
    Ok(repos)
}

#[tauri::command]
pub async fn update_repo_icon(
    app: tauri::AppHandle,
    path: String,
    icon: String,
) -> Result<Vec<ManagedRepo>, String> {
    let mut repos = read(&app)?;
    if let Some(repo) = repos.iter_mut().find(|r| r.path == path) {
        repo.icon = Some(icon);
    }
    write(&app, &repos)?;
    Ok(repos)
}

#[tauri::command]
pub async fn open_dashboard_window(app: tauri::AppHandle) -> Result<(), String> {
    let label = format!("dashboard-{}", chrono::Utc::now().timestamp_millis());
    tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::App("/".into()),
    )
    .title("Index — 仓库管理")
    .inner_size(900.0, 640.0)
    .resizable(true)
    .decorations(true)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}
