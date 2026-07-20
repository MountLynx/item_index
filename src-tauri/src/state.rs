use std::collections::HashMap;
use std::sync::Mutex;
use crate::db::DbPool;
use crate::models::PluginUsage;

pub struct RepoState {
    pub db: DbPool,
    pub path: String,
}

pub struct AppState {
    pub repos: Mutex<HashMap<String, RepoState>>,
    pub pending_sub_repos: Mutex<HashMap<String, String>>, // label -> path, for new sub-repo windows
    pub theme: Mutex<String>,
    pub plugin_refs: Mutex<HashMap<String, PluginUsage>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            repos: Mutex::new(HashMap::new()),
            pending_sub_repos: Mutex::new(HashMap::new()),
            theme: Mutex::new("light".to_string()),
            plugin_refs: Mutex::new(HashMap::new()),
        }
    }
}
