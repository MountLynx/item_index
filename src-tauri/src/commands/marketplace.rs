use std::io::Cursor;
use tauri::{AppHandle, Manager};
use crate::models::PluginIndex;

const MARKETPLACE_INDEX_URL: &str =
    "https://raw.githubusercontent.com/MountLynx/index-plugins/main/index.json";

/// Fetch the plugin marketplace index from the remote repository.
#[tauri::command]
pub async fn fetch_marketplace_index() -> Result<PluginIndex, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(MARKETPLACE_INDEX_URL)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;
    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    serde_json::from_str::<PluginIndex>(&body)
        .map_err(|e| format!("解析插件索引失败: {}", e))
}

/// Download a plugin zip from the marketplace and extract to the global plugin store.
#[tauri::command]
pub async fn download_marketplace_plugin(
    app: AppHandle,
    url: String,
    expected_sha256: String,
) -> Result<(), String> {
    // 1. Download
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await
        .map_err(|e| format!("下载失败: {}", e))?;
    let bytes = resp.bytes().await
        .map_err(|e| format!("读取下载内容失败: {}", e))?;

    // 2. Verify SHA256
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());
    if hash != expected_sha256.to_lowercase() {
        return Err(format!("校验失败: 期望 {} 实际 {}", expected_sha256, hash));
    }

    // 3. Extract
    let cursor = Cursor::new(&bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("解压失败: {}", e))?;

    // Collect entries: (name, is_dir, data)
    struct Entry {
        name: String,
        is_dir: bool,
        data: Vec<u8>,
    }
    let mut entries: Vec<Entry> = vec![];
    let mut top_dir = String::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("读取压缩包条目失败: {}", e))?;
        let full_name = file.name().to_string();
        let is_dir = file.is_dir();

        if top_dir.is_empty() {
            if let Some(slash) = full_name.find('/') {
                top_dir = full_name[..slash].to_string();
            }
        }

        let relative = if let Some(slash) = full_name.find('/') {
            let rel = &full_name[slash + 1..];
            if rel.is_empty() { continue; }
            rel.to_string()
        } else {
            full_name.clone()
        };

        let mut data = Vec::new();
        if !is_dir {
            use std::io::Read;
            file.read_to_end(&mut data)
                .map_err(|e| format!("读取文件失败: {}", e))?;
        }
        entries.push(Entry { name: relative, is_dir, data });
    }

    // Validate plugin name safety
    if top_dir.contains("..") || top_dir.contains('/') || top_dir.contains('\\') {
        return Err(format!("无效的插件名: '{}'", top_dir));
    }

    // Find and validate manifest
    let manifest_raw = entries.iter()
        .find(|e| e.name == "manifest.json")
        .map(|e| String::from_utf8_lossy(&e.data).to_string())
        .ok_or("压缩包中缺少 manifest.json")?;

    let manifest: crate::models::PluginManifest = serde_json::from_str(&manifest_raw)
        .map_err(|e| format!("manifest.json 解析失败: {}", e))?;

    if manifest.name != top_dir {
        return Err(format!(
            "插件名不匹配: 目录 '{}' != manifest '{}'",
            top_dir, manifest.name
        ));
    }

    // Extract files to plugin-store
    let store_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("plugin-store");
    std::fs::create_dir_all(&store_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let dest = store_dir.join(&top_dir);
    if dest.exists() {
        std::fs::remove_dir_all(&dest)
            .map_err(|e| format!("删除旧版本失败: {}", e))?;
    }

    for entry in &entries {
        let out = dest.join(&entry.name);
        if entry.is_dir {
            std::fs::create_dir_all(&out)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            std::fs::write(&out, &entry.data)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    Ok(())
}
