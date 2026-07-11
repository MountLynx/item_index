use std::path::{Path, PathBuf};

/// Resolve a path inside repo/<item_id>/<rel_path>, preventing traversal attacks.
pub fn safe_path(repo_root: &Path, item_id: &str, rel_path: &str) -> Result<PathBuf, String> {
    let repo_root = repo_root
        .canonicalize()
        .map_err(|e| format!("Cannot resolve repo root: {}", e))?;
    let candidate = repo_root.join(item_id).join(rel_path);

    let resolved = if candidate.exists() {
        candidate
            .canonicalize()
            .map_err(|e| format!("Cannot resolve path: {}", e))?
    } else {
        let mut existing = candidate.clone();
        let mut tail = vec![];
        while !existing.exists() {
            tail.push(
                existing
                    .file_name()
                    .ok_or("Invalid path component")?
                    .to_os_string(),
            );
            existing = existing
                .parent()
                .ok_or("No valid parent found")?
                .to_path_buf();
        }
        let mut base = existing
            .canonicalize()
            .map_err(|e| format!("Cannot resolve path: {}", e))?;
        for component in tail.into_iter().rev() {
            base = base.join(component);
        }
        base
    };

    if !resolved.starts_with(&repo_root) {
        return Err("Path traversal detected".to_string());
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_valid_path() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();

        // Create item directory and a file inside it
        let item_dir = repo_root.join("item001");
        fs::create_dir(&item_dir).unwrap();
        let file_path = item_dir.join("file.txt");
        fs::write(&file_path, "hello").unwrap();

        let result = safe_path(repo_root, "item001", "file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), file_path.canonicalize().unwrap());
    }

    #[test]
    fn test_blocks_traversal() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();

        let result = safe_path(repo_root, "item001", "../../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("traversal"));
    }

    #[test]
    fn test_nonexistent_valid_path() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();

        // Create item directory but no file inside it
        let item_dir = repo_root.join("item002");
        fs::create_dir(&item_dir).unwrap();

        // A path that doesn't exist yet but is within the allowed tree
        let result = safe_path(repo_root, "item002", "nonexistent/subfolder");
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert!(resolved.starts_with(repo_root.canonicalize().unwrap()));
    }
}
