use std::sync::Mutex;
use crate::db::DbPool;

pub struct AppState {
    pub db: Mutex<Option<DbPool>>,
    pub repo_path: Mutex<Option<String>>,
    pub theme: Mutex<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: Mutex::new(None),
            repo_path: Mutex::new(None),
            theme: Mutex::new("light".to_string()),
        }
    }
}
