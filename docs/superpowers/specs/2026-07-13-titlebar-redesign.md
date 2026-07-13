# Custom Titlebar — Frameless Window Redesign

**日期**: 2026-07-13
**状态**: 设计完成

---

## 1. 概述

将原生 Windows 标题栏替换为自定义 HTML 标题栏，单行 40px，整合原 Topbar 的所有功能，所有颜色使用 CSS 令牌自动适配浅色/暗色主题。

## 2. 技术方案

### Tauri 配置

```json
// tauri.conf.json → app.windows[0]
{
  "decorations": false,
  "titleBarStyle": "Overlay"
}
```

- `decorations: false` — 移除原生标题栏
- `titleBarStyle: "Overlay"` — macOS 上用 overlay 风格（Windows 上等效 frameless）

### 窗口控制

使用 `@tauri-apps/api/window` 的 `getCurrentWindow()`：

```ts
import { getCurrentWindow } from '@tauri-apps/api/window'
const win = getCurrentWindow()
win.minimize()  // 最小化
win.toggleMaximize()  // 最大化/还原
win.close()  // 关闭
```

## 3. 标题栏布局

```
┌─ 单行标题栏 (40px, --surface 背景, --border 底边) ────────────────────────┐
│ ◆ 仓库名        [+ 新建条目]  ⚙  ☰  │  ─  □  ✕                        │
│ ← drag-region →                     │  ← 窗口控制 →                    │
└───────────────────────────────────────────────────────────────────────────┘
```

- 整行设置 `data-tauri-drag-region` 可拖拽移动
- 左侧：Logo 图标 + 仓库名（`--text`，`font-weight: 620`）
- 中间/右侧：操作按钮（`+ 新建条目` primary 按钮、齿轮设置、类别管理）
- 最右侧：窗口控制按钮，用竖线 `--border` 分隔

### 窗口控制按钮行为

| 按钮 | 图标 | hover 默认 | hover 关闭 |
|------|------|-----------|-----------|
| 最小化 | `─` | `--surface-hover` 背景 | — |
| 最大化/还原 | `□` / `❐` | `--surface-hover` 背景 | — |
| 关闭 | `✕` | — | `#C42B1C` 背景 + `#fff` 文字 |

## 4. 组件结构

**修改 `App.vue`**：移除 `<Topbar>`，内联标题栏 HTML 到 `.app` 顶部，或创建 `Titlebar.vue`。

推荐创建 `Titlebar.vue` 组件，保持 `App.vue` 简洁：

```
App.vue
├── Titlebar          ← 新：40px 自定义标题栏
├── .main (flex row)
│   ├── Sidebar
│   ├── CenterList
│   └── RightPanel
├── StatusBar
├── NewItemDialog
├── SettingsModal
└── Toast
```

## 5. CSS 令牌映射

| 标题栏元素 | CSS 变量 |
|-----------|----------|
| 背景 | `var(--surface)` |
| 底边框 | `var(--border)` |
| 仓库名文字 | `var(--text)` |
| Logo 图标 | `var(--accent)` |
| 按钮文字 | `var(--text)` |
| 按钮 hover | `var(--surface-hover)` |
| 关闭按钮 hover | `#C42B1C`（硬编码，危险红色） |
| 分隔线 | `var(--border-strong)` |

## 6. 文件变更

| 类型 | 文件 | 说明 |
|------|------|------|
| 新建 | `src/components/Titlebar.vue` | 自定义标题栏组件 |
| 修改 | `src/App.vue` | 移除 Topbar → Titlebar |
| 修改 | `src-tauri/tauri.conf.json` | `decorations: false` |
| 可删除 | `src/components/Topbar.vue` | 被 Titlebar 替代（保留也可） |

## 7. 向后兼容

- Topbar 移除不影响其他组件 — SettingsModal、NewItemDialog 等均独立
- 窗口尺寸不变，仅标题栏从 48px 原生 + 48px Topbar 变为 40px 自定义，净增 56px 内容空间
