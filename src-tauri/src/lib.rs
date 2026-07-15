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

/// Copy bundled resources (plugins, presets) from resource dir to app_data_dir.
/// Skips files that already exist (first-run only).
fn deploy_bundled_resources(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let resource_dir = app.path().resource_dir()?;
    let app_data = app.path().app_data_dir()?;

    // Only deploy if plugin-store is missing or empty
    let store_dir = app_data.join("plugin-store");
    let presets_dir = app_data.join("workspace-presets");

    let src_plugins = resource_dir.join("plugins");
    let src_presets = resource_dir.join("presets");

    if src_plugins.exists() && !store_dir.join("calendar-view").exists() {
        copy_dir_if_new(&src_plugins, &store_dir);
    }
    if src_presets.exists() {
        std::fs::create_dir_all(&presets_dir).ok();
        copy_dir_if_new(&src_presets, &presets_dir);
    }
    Ok(())
}

fn copy_dir_if_new(src: &std::path::Path, dst: &std::path::Path) {
    if !dst.exists() {
        std::fs::create_dir_all(dst).ok();
    }
    for entry in std::fs::read_dir(src).into_iter().flatten().flatten() {
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            // Only copy dir if target doesn't exist (don't overwrite user's plugins)
            if !dst_path.exists() {
                copy_dir_if_new(&entry.path(), &dst_path);
            }
        } else {
            // Copy file only if target doesn't exist
            if !dst_path.exists() {
                std::fs::copy(entry.path(), &dst_path).ok();
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::repo::create_repo,
            commands::repo::open_repo,
            commands::repo::close_repo,
            commands::repo::get_repo_info,
            commands::repo::get_state,
            commands::repo::save_state,
            commands::dashboard::list_managed_repos,
            commands::dashboard::add_managed_repo,
            commands::dashboard::remove_managed_repo,
            commands::dashboard::update_repo_icon,
            commands::dashboard::open_dashboard_window,
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
            commands::workspace::list_workspaces,
            commands::workspace::read_workspace,
            commands::workspace::write_workspace,
            commands::workspace::delete_workspace,
            commands::plugin::list_installed_plugins,
            commands::plugin::read_plugin_file,
            commands::presets::list_workspace_presets,
            commands::presets::install_preset,
            commands::presets::export_preset,
        ])
        .setup(|app| {
            // Deploy bundled plugins and presets to app-data on first run
            deploy_bundled_resources(app.handle())?;

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
