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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedRepo {
    pub path: String,
    pub icon: Option<String>,
    pub name: Option<String>,
    pub last_opened_at: String,
    pub item_count: Option<i64>,
}

// ── Workspace & Plugin models ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CenterTab {
    #[serde(rename = "type")]
    pub tab_type: String,              // "list" or "plugin"
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,        // plugin name, only when type="plugin"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonRef {
    pub plugin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub item_types: Vec<String>,       // item type names, empty = all
    #[serde(rename = "centerTabs")]
    pub center_tabs: Vec<CenterTab>,
    #[serde(rename = "defaultTab")]
    pub default_tab: String,
    #[serde(default, rename = "rightPanelAddons")]
    pub right_panel_addons: Vec<AddonRef>,
    #[serde(default, rename = "sidebarAddons")]
    pub sidebar_addons: Vec<AddonRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub name: String,
    pub icon: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub title: String,
    pub icon: String,
    pub extends: String,
    #[serde(default)]
    pub requires_fields: Vec<String>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSummary {
    pub name: String,
    pub icon: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetBundle {
    pub plugins: Vec<String>,
    #[serde(rename = "itemTypes")]
    pub item_types: Vec<PresetTypeTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetTypeTemplate {
    pub name: String,
    pub icon: String,
    pub fields: Vec<PresetFieldTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetFieldTemplate {
    pub name: String,
    pub field_type: String,
    pub icon: String,
    pub label: String,
}
