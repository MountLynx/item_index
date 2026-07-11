use std::path::Path;
use tauri::State;
use crate::models::FileNode;
use crate::state::AppState;

fn get_repo_path(state: &State<'_, AppState>) -> Result<String, String> {
    state.repo_path.lock().unwrap().clone().ok_or("No repository open".to_string())
}

fn read_dir_recursive(path: &Path) -> Result<Vec<FileNode>, String> {
    let mut children = vec![];
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let sub_children = if is_dir {
            read_dir_recursive(&entry.path())?
        } else {
            vec![]
        };
        children.push(FileNode { name, is_dir, children: sub_children });
    }
    children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(children)
}

fn resolve_path(repo_path: &str, item_id: &str, rel_path: &str) -> Result<std::path::PathBuf, String> {
    crate::safe_path::safe_path(Path::new(repo_path), item_id, rel_path)
}

#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
    item_id: String,
) -> Result<FileNode, String> {
    let repo_path = get_repo_path(&state)?;
    let item_dir = Path::new(&repo_path).join(&item_id);
    if !item_dir.exists() {
        return Ok(FileNode { name: item_id, is_dir: true, children: vec![] });
    }
    let children = read_dir_recursive(&item_dir)?;
    Ok(FileNode { name: item_id, is_dir: true, children })
}

#[tauri::command]
pub async fn create_folder(
    state: State<'_, AppState>,
    item_id: String,
    rel_path: String,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;
    let target = resolve_path(&repo_path, &item_id, &rel_path)?;
    std::fs::create_dir_all(&target).map_err(|e| format!("Cannot create folder: {}", e))
}

#[tauri::command]
pub async fn delete_file(
    state: State<'_, AppState>,
    item_id: String,
    rel_path: String,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;
    let target = resolve_path(&repo_path, &item_id, &rel_path)?;
    if target.is_dir() {
        std::fs::remove_dir_all(&target).map_err(|e| format!("Cannot delete: {}", e))
    } else {
        std::fs::remove_file(&target).map_err(|e| format!("Cannot delete: {}", e))
    }
}

#[tauri::command]
pub async fn rename_file(
    state: State<'_, AppState>,
    item_id: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;
    let old_path = resolve_path(&repo_path, &item_id, &old_name)?;
    let new_path = resolve_path(&repo_path, &item_id, &new_name)?;
    std::fs::rename(&old_path, &new_path).map_err(|e| format!("Cannot rename: {}", e))
}

#[tauri::command]
pub async fn add_attachment(
    state: State<'_, AppState>,
    item_id: String,
    source_path: String,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;
    let source = Path::new(&source_path);
    let file_name = source.file_name()
        .ok_or("Invalid source path")?
        .to_string_lossy()
        .to_string();

    let item_dir = Path::new(&repo_path).join(&item_id);
    let mut target = item_dir.join(&file_name);

    // Auto-rename on collision: "cover.jpg" → "cover (2).jpg"
    if target.exists() {
        let stem = source.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = source.extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut counter = 2;
        loop {
            let new_name = if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            };
            target = item_dir.join(&new_name);
            if !target.exists() { break; }
            counter += 1;
        }
    }

    std::fs::copy(source, &target).map_err(|e| format!("Cannot copy file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_file(
    state: State<'_, AppState>,
    item_id: String,
    rel_path: String,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;
    let target = resolve_path(&repo_path, &item_id, &rel_path)?;
    open::that(target.to_str().ok_or("Invalid path")?)
        .map_err(|e| format!("Cannot open file: {}", e))
}
