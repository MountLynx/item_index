pub mod commands;
pub mod db;
pub mod models;
pub mod safe_path;
pub mod state;

use state::AppState;
use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Index.", name)
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
            commands::repo::get_state,
            commands::repo::save_state,
            commands::types::list_item_types,
            commands::types::create_item_type,
            commands::types::delete_item_type,
            commands::types::add_field,
            commands::types::remove_field,
            commands::types::reorder_fields,
            commands::types::update_item_type,
            commands::types::update_field,
            commands::items::create_item,
            commands::items::get_item,
            commands::items::list_items,
            commands::items::update_item,
            commands::items::delete_item,
            commands::groups::list_groups,
            commands::groups::create_group,
            commands::groups::update_group,
            commands::groups::delete_group,
            commands::groups::move_group,
            commands::groups::add_item_to_group,
            commands::groups::remove_item_from_group,
            commands::tags::list_tags,
            commands::tags::create_tag,
            commands::tags::delete_tag,
            commands::tags::add_tag_to_item,
            commands::tags::remove_tag_from_item,
            commands::files::list_files,
            commands::files::create_folder,
            commands::files::delete_file,
            commands::files::rename_file,
            commands::files::add_attachment,
            commands::files::open_file,
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
