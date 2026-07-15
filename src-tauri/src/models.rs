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
    #[serde(default, rename = "itemTypes")]
    pub item_types: Vec<String>,       // item type names, empty = all
    #[serde(default, rename = "centerTabs")]
    pub center_tabs: Vec<CenterTab>,
    #[serde(default, rename = "defaultTab")]
    pub default_tab: String,
    #[serde(default, rename = "rightPanelAddons")]
    pub right_panel_addons: Vec<AddonRef>,
    #[serde(default, rename = "sidebarAddons")]
    pub sidebar_addons: Vec<AddonRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub name: String,        // display name from config (e.g. "日程管理")
    pub key: String,         // unique identifier / filename stem (e.g. "schedule")
    pub icon: String,
    pub is_default: bool,    // matches state.json active_workspace
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub title: String,
    pub icon: String,
    pub extends: String,
    #[serde(default, rename = "requiresFields")]
    pub requires_fields: Vec<String>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub icon_file: Option<String>,
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

// ── Plugin marketplace & reference tracking ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginIndex {
    pub version: u32,
    pub plugins: Vec<PluginIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginIndexEntry {
    pub name: String,
    pub version: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub icon: String,
    pub extends: String,
    #[serde(default, rename = "requiresFields")]
    pub requires_fields: Vec<String>,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginUsage {
    #[serde(default)]
    pub repos: Vec<String>,
    #[serde(default)]
    pub presets: Vec<String>,
}

pub type RefTable = std::collections::HashMap<String, PluginUsage>;
