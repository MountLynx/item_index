export interface ItemType {
  id: number
  name: string
  icon: string
  fields: Field[]
}

export interface Field {
  id: number
  type_id: number
  name: string
  field_type: 'text' | 'checkbox'
  position: number
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
