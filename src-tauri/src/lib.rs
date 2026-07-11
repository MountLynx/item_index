pub mod commands;
pub mod db;
pub mod models;
pub mod safe_path;
pub mod state;

use state::AppState;
use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Vault.", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::repo::create_repo,
            commands::repo::open_repo,
            commands::repo::close_repo,
            commands::repo::get_repo_info,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
