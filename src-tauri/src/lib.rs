pub mod commands;
pub mod db;
pub mod models;
pub mod safe_path;
pub mod refs;
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
    let app_data = app.path().app_data_dir()?;

    // Use CARGO_MANIFEST_DIR as base — resource_dir() points to target/debug in dev mode
    let resources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    let src_plugins = resources.join("plugins");
    let src_presets = resources.join("presets");

    let store_dir = app_data.join("plugin-store");
    let presets_dir = app_data.join("workspace-presets");

    // Copy plugins
    if src_plugins.exists() {
        std::fs::create_dir_all(&store_dir).ok();
        for entry in std::fs::read_dir(&src_plugins)?.flatten() {
            let name = entry.file_name();
            let dst = store_dir.join(&name);
            // Remove broken entries, then copy only if missing (don't overwrite user changes)
            if dst.exists() && !dst.is_dir() {
                std::fs::remove_file(&dst).ok();
            }
            if !dst.exists() {
                copy_dir_all(&entry.path(), &dst)?;
            }
        }
    }

    // Copy presets
    if src_presets.exists() {
        std::fs::create_dir_all(&presets_dir).ok();
        for entry in std::fs::read_dir(&src_presets)?.flatten() {
            let dst = presets_dir.join(entry.file_name());
            if !dst.exists() {
                std::fs::copy(&entry.path(), &dst).ok();
            }
        }
    }

    Ok(())
}

/// Simple recursive directory copy.
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(&entry.path(), &dst_path)?;
        }
    }
    Ok(())
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
            commands::repo::open_sub_repo_window,
            commands::repo::get_sub_repo_path,
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
            commands::items::open_item_folder,
            commands::items::create_sub_repo,
            commands::items::list_sub_repos,
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
            commands::presets::list_global_plugins,
            commands::presets::install_plugin,
            commands::presets::install_plugin_to_global,
            commands::presets::delete_plugin,
            commands::plugin::check_plugin_usage,
            commands::plugin::uninstall_plugin_from_repo,
            commands::marketplace::fetch_marketplace_index,
            commands::marketplace::download_marketplace_plugin,
            commands::query::query_items,
        ])
        .setup(|app| {
            // Deploy bundled plugins and presets to app-data on first run
            deploy_bundled_resources(app.handle())?;

            // Load plugin reference table
            {
                let app_handle = app.handle().clone();
                let refs = crate::refs::load_refs(&app_handle).unwrap_or_default();
                *app.state::<AppState>().plugin_refs.lock().unwrap() = refs;
            }

            // Register cleanup for the main window on destroy
            {
                let app_handle = app.handle().clone();
                let main_window = app.get_webview_window("main").unwrap();
                let label = main_window.label().to_string();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Destroyed = event {
                        let state = app_handle.state::<AppState>();
                        let mut repos = state.repos.lock().unwrap();
                        repos.remove(&label);
                        let mut pending = state.pending_sub_repos.lock().unwrap();
                        pending.remove(&label);
                    }
                });
            }

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
