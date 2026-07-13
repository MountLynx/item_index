# Repo Dashboard — 入口仓库看板

**日期**: 2026-07-13  
**状态**: 设计完成，待用户审阅

## 概述

将当前简陋的 `EmptyState.vue` 入口界面重新设计为仓库管理看板，支持管理多个本地仓库（创建、导入、选择、删除），统一使用现有设计令牌和主题系统。

## 架构

### 数据层（Rust Backend）

新增 4 个 Tauri 命令，读写 `app_data_dir/repos.json`：

| 命令 | 功能 |
|------|------|
| `list_managed_repos` | 读取 `repos.json`，返回 `Vec<ManagedRepo>` |
| `add_managed_repo` | 添加或更新一条仓库记录 |
| `remove_managed_repo` | 按路径移除记录（不删除实际文件） |
| `update_repo_icon` | 更新某个仓库的自定义图标 |

```rs
struct ManagedRepo {
    path: String,
    icon: Option<String>,       // emoji 或 Tabler 图标名
    name: Option<String>,       // 可选自定义名称，默认取路径最后一段
    last_opened_at: String,     // ISO 8601
    item_count: Option<i32>,    // 缓存，打开仓库时刷新
}
```

`repos.json` 格式：
```json
[
  {"path": "/Users/xxx/MyIndex", "icon": "📁", "name": null, "last_opened_at": "2026-07-13T10:30:00Z", "item_count": 12},
  {"path": "/Users/xxx/Books", "icon": "books", "name": null, "last_opened_at": "2026-07-12T08:00:00Z", "item_count": 45}
]
```

- 文件位于 Tauri app data 目录内
- 路径验证通过 `safe_path.rs` 确保不包含非法路径
- 重复路径自动合并（更新而非插入）

### 状态层（Pinia Store）

新增 `src/stores/dashboard.ts` — `useDashboardStore`：

| 状态 | 类型 | 说明 |
|------|------|------|
| `repos` | `ManagedRepo[]` | 仓库列表 |
| `loading` | `boolean` | 加载中 |

| 动作 | 说明 |
|------|------|
| `fetchAll()` | 调用 `list_managed_repos` |
| `addRepo(path, icon?)` | 调用 `add_managed_repo`，刷新列表 |
| `removeRepo(path)` | 调用 `remove_managed_repo`，刷新列表 |
| `updateIcon(path, icon)` | 调用 `update_repo_icon`，刷新列表 |

`repo.ts` 的 `openRepo()` / `createRepo()` 成功后自动调用 `dashboardStore.addRepo()` 记录。

### 组件树

```
RepoDashboard.vue          ← 替换 EmptyState.vue，全屏居中布局
├── RepoCard.vue           ← 网格卡片：大图标 + 名称 + 条目数 + 时间
├── RepoCreateTile.vue     ← 虚线边框、创建/导入入口
└── TemplateBanner.vue     ← 底部横幅："从模板快速开始 · 即将推出"
```

## 视觉设计

统一使用 `theme.css` 设计令牌（`--surface`, `--border`, `--text-secondary` 等），自动适配亮/暗/自定义主题。

### 整体布局

居中网格布局，卡片自动换行。最大宽度约 800px，垂直居中。

### RepoCard

- 尺寸：`180×140px`，`--r-lg` 圆角，`--surface` 背景，`--border` 边框
- Hover：边框 → `--border-strong`，`transform: translateY(-2px)`，`--slow` 过渡
- 内容（从上到下居中排列）：
  - 图标：48px（用户自定义 emoji/图标，默认 📁）
  - 名称：`--fw-semibold`，居中，单行截断
  - 元数据行：`📄 12 项` + `🕐 2小时前`，`--fs-xs`，`--text-secondary`
- 右下角 `...` 菜单：删除（确认弹窗提示"文件和数据仍保留在本地"）

### RepoCreateTile

- 同样 `180×140px`，**虚线边框** `2px dashed var(--border-strong)`
- 居中大号 "+"（`--text-muted`，hover → `--accent`）
- 文字："创建或导入仓库"
- 点击弹出选择菜单：
  - "新建仓库" → Tauri 原生文件夹选择 → 填写名称/图标 → `createRepo()`
  - "导入已有仓库" → 文件夹选择（选中含 `.index/index.db` 的目录）→ `addManagedRepo()`

### TemplateBanner

- 底部居中一行：`💡 从模板快速开始 · 即将推出`
- `--fs-xs`，`--text-muted`，`--surface` 背景标签样式，无交互

## 交互流程

### 多窗口逻辑

```
仪表盘窗口 ──点击仓库卡片──> 同窗口变为仓库工作界面
仓库窗口   ──点击"仓库"按钮──> 新开仪表盘窗口（原仓库窗口不变）
```

- 仪表盘窗口为默认窗口（`main` label）
- "仓库"按钮通过 `WebviewWindow` API 创建新仪表盘窗口
- 新窗口加载同一前端，由于未打开仓库，自然显示 `RepoDashboard`

### Titlebar 变更

移除右上角"类别管理"按钮（`category` 图标），替换为"仓库"按钮（`building-warehouse` 或 `database` 图标），点击新开仪表盘窗口。

### 操作矩阵

| 操作 | 触发 | 行为 |
|------|------|------|
| 打开仓库 | 点击卡片 | 同窗口 `openRepo()` |
| 创建仓库 | 创建卡片 → "新建仓库" | 原生文件夹选择 → `createRepo()` + 记录 |
| 导入仓库 | 创建卡片 → "导入已有仓库" | 文件夹选择 → 校验 `.index/index.db` → 记录 |
| 删除仓库 | 卡片菜单 → 删除 | 确认弹窗 → `removeManagedRepo()` |
| 新开仪表盘 | Titlebar"仓库"按钮 | 新窗口 → 显示看板 |

### 边界情况

| 情况 | 处理 |
|------|------|
| 首次使用（repos.json 不存在或为空） | 仅显示创建卡片 + 模板横幅 |
| 所选目录已是已管理仓库 | 跳过重复添加，直接打开 |
| 所选目录不是合法仓库（导入时） | 提示"所选目录不包含 Index 仓库"，不添加 |
| repos.json 中路径已被手动删除 | 卡片显示"路径不可用"提示，引导移除 |
| repos.json 损坏/读取失败 | 降级为空列表，不阻塞启动 |

## 向后兼容

- `EmptyState.vue` 的 `repoOpened` emit 签名保持不变
- `App.vue` 的 `v-if="!repoStore.isOpen"` 逻辑不变
- `repos.json` 是新增文件，不影响现有 `.index/state.json`
- "类别管理"功能通过 SettingsModal 仍可访问（在设置面板中）
