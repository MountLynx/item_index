export interface ItemType {
  id: number
  name: string
  icon: string
  namespace: string
  fields: Field[]
}

export interface Field {
  id: number
  type_id: number
  name: string
  field_type: 'text' | 'checkbox' | 'date' | 'number'
  icon: string
  position: number
  label: string
}

export interface Item {
  id: string
  name: string
  type_id: number
  properties: Record<string, unknown>
  namespace: string
  created_at: string
  updated_at: string
}

export interface ItemDetail {
  item: Item
  item_type: ItemType
  groups: Group[]
  tags: Tag[]
  files: FileNode
}

export interface Group {
  id: number
  parent_id: number | null
  name: string
  position: number
  children: Group[]
}

export interface Tag {
  id: number
  name: string
  namespace: string
}

export interface FileNode {
  name: string
  is_dir: boolean
  children: FileNode[]
}

export interface RepoInfo {
  path: string
  item_count: number
  db_version: number
}

export interface ManagedRepo {
  path: string
  icon: string | null
  name: string | null
  last_opened_at: string
  item_count: number | null
}

// ── Workspace & Plugin types ──

export interface CenterTab {
  type: 'list' | 'plugin'
  label: string
  icon?: string
  plugin?: string        // plugin name, only when type='plugin'
  config?: Record<string, unknown>
}

export interface WorkspaceConfig {
  name: string
  icon: string
  itemTypes: string[]    // item type names, empty = all
  centerTabs: CenterTab[]
  defaultTab: string
  rightPanelAddons: { plugin: string; config?: Record<string, unknown> }[]
  sidebarAddons: { plugin: string; config?: Record<string, unknown> }[]
}

export interface WorkspaceSummary {
  name: string
  key: string
  icon: string
  is_default: boolean
}

export interface PluginManifest {
  name: string
  version: string
  title: string
  icon: string
  extends: 'center-panel' | 'right-panel' | 'sidebar'
  requiresFields: string[]
  config?: Record<string, unknown>
  author: string
  description: string
  homepage?: string
  iconFile?: string
}

export interface PresetSummary {
  name: string
  icon: string
  description: string
}

export interface PluginIndex {
  version: number
  plugins: PluginIndexEntry[]
}

export interface PluginIndexEntry {
  name: string
  version: string
  title: string
  author: string
  description: string
  icon: string
  extends: string
  requiresFields: string[]
  downloadUrl: string
  sha256: string
}

export interface PluginUsage {
  repos: string[]
  presets: string[]
}
