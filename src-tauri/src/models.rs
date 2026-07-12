use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemType {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub namespace: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: i64,
    pub type_id: i64,
    pub name: String,
    pub field_type: String,
    pub icon: String,
    pub position: i32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub type_id: i64,
    pub properties: serde_json::Value,
    pub namespace: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetail {
    pub item: Item,
    pub item_type: ItemType,
    pub groups: Vec<Group>,
    pub tags: Vec<Tag>,
    pub files: FileNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub position: i32,
    pub children: Vec<Group>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: String,
    pub item_count: i64,
    pub db_version: i64,
}
