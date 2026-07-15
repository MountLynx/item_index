use std::collections::HashMap;
use std::sync::Mutex;
use crate::db::DbPool;
use crate::models::PluginUsage;

pub struct AppState {
    pub db: Mutex<Option<DbPool>>,
    pub repo_path: Mutex<Option<String>>,
    pub theme: Mutex<String>,
    pub plugin_refs: Mutex<HashMap<String, PluginUsage>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: Mutex::new(None),
            repo_path: Mutex::new(None),
            theme: Mutex::new("light".to_string()),
            plugin_refs: Mutex::new(HashMap::new()),
        }
    }
}
