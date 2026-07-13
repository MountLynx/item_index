# Theme Redesign — "Quiet Strength" 样式美化与主题设置系统

**日期**: 2026-07-13
**状态**: 设计完成，待用户审阅

---

## 1. 概述

将应用整体视觉升级为 "Quiet Strength" 设计哲学，同时新增一个独立的设置弹窗，支持基础主题切换和自定义 CSS 皮肤预设管理。

### 设计原则

- UI 退后，内容主导。每个视觉元素必须证明其存在的必要性。
- 无装饰元素，无渐变表面，无粗重边框。
- 边框是耳语，不是墙壁。

---

## 2. 设计令牌改造

### 2.1 策略

现有 CSS 变量名**保持不变**（`--bg`、`--text`、`--accent` 等），仅调整值到 "Quiet Strength" 规范。新增少数几个变量（`--text-heading`、`--border-strong`、`--link`）。

所有组件中的 `var(--xxx)` 引用无需修改——名字不变，效果自动对齐。

### 2.2 颜色系统（浅色主题 `:root`）

| 令牌 | 当前值 | 新值 | 说明 |
|------|--------|------|------|
| `--bg` | `#FFFFFF` | `#FFFFFF` | 内容区白底 |
| `--surface` | `#FFFFFF` | `#FAFAFA` | 侧边栏/面板暖灰底 |
| `--surface-hover` | `#F5F5F5` | `#F0F0F0` | 悬停态 |
| `--surface-active` | `#F5F5F5` | `#E8E8E8` | 按压/活跃态 |
| `--text` | `#171717` | `#555555` | 正文灰度 |
| `--text-secondary` | `#737373` | `#999999` | 辅助文字 |
| `--text-muted` | `#D4D4D4` | `#C7C5C5` | 最淡（markdown 标记等） |
| **`--text-heading`** | — | `#333333` | **新增**：标题用 |
| `--border` | `#E8E8E8` | `#EEEEEE` | 淡化默认边框 |
| **`--border-strong`** | — | `#DDDDDD` | **新增**：结构分隔 |
| **`--border-light`** | `#F5F5F5` | `#F3F3F3` | 更浅分隔 |
| `--accent` | `#3B82F6`（蓝） | `#1A1C1E`（墨黑） | 主操作色 |
| `--accent-hover` | `#2563EB` | `#333333` | 按压态 |
| `--accent-subtle` | `#EFF6FF` | `rgba(26,28,30,0.10)` | 10% 墨黑淡化 |
| `--accent-fg` | `#FFFFFF` | `#FFFFFF` | 强调前景文字 |
| `--danger` | `#EF4444` | `#B42318` | 克制红色 |
| `--danger-hover` | `#DC2626` | `#8B1A10` | 红色悬停 |
| `--danger-subtle` | `#FEF2F2` | `rgba(180,35,24,0.08)` | 红色淡化 |
| **`--link`** | — | `#2F56C6` | **新增**：链接蓝色 |

### 2.3 暗色主题（`.dark`）

| 令牌 | 新值 | 说明 |
|------|------|------|
| `--bg` | `#1E1E1E` | 深炭色底 |
| `--surface` | `#252525` | 面板色 |
| `--surface-hover` | `#2E2E2E` | 悬停 |
| `--surface-active` | `#353535` | 活跃 |
| `--text` | `#E1E1E1` | 近白正文 |
| `--text-heading` | `#F4F4F5` | 近白标题 |
| `--text-secondary` | `#999999` | 辅助 |
| `--text-muted` | `#555555` | 最淡 |
| `--border` | `#333333` | 边框 |
| `--border-strong` | `#444444` | 结构边框 |
| `--accent` | `#F4F4F5` | 近白强调 |
| `--accent-hover` | `#CCCCCC` | 悬停 |
| `--accent-subtle` | `rgba(244,244,245,0.10)` | 淡化 |
| `--accent-fg` | `#1E1E1E` | 深色前景 |
| `--danger` | `#DA4A3F` | 暗色红 |
| `--danger-hover` | `#C0392B` | |
| `--danger-subtle` | `rgba(218,74,63,0.12)` | |
| `--link` | `#6B8FE8` | 暗色链接 |

### 2.4 字体权重

```css
--fw-normal:   400;   /* 正文 */
--fw-tertiary: 520;   /* 新增：三级标签 */
--fw-semibold: 560;   /* 曾 600 → 二级标签/控件 */
--fw-medium:   620;   /* 曾 500 → UI 控件主标签 */
--fw-emphasis: 650;   /* 新增：强调 */
--fw-bold:     760;   /* 曾 700 → 标题 */
```

### 2.5 字体栈

```css
--font: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
/* 移除 Inter 和 Microsoft YaHei，使用系统原生栈 */
/* font-synthesis: none; antialiased */
```

### 2.6 UI 字号

```css
--fs-xs:   0.6875rem;  /* 11px，保留但仅标记/徽章用 */
--fs-sm:   0.8125rem;  /* 13px */
--fs-base: 0.875rem;   /* 14px，UI 默认 */
--fs-lg:   1rem;       /* 16px */
```

### 2.7 间距与按钮

按钮高度对齐 spec：`h-8`（32px）保持默认，`h-9`（36px）作为 `.md` 变体。间距使用 2px 增量。

### 2.8 阴影（仅浮层元素）

```css
--shadow-sm: 0 1px 3px rgba(0,0,0,0.06);
--shadow-md: 0 18px 52px rgba(0,0,0,0.12);   /* 弹窗/菜单 */
--shadow-lg: 0 20px 64px rgba(0,0,0,0.16);   /* 展开态 */
/* 暗色模式额外：1px rgba(255,255,255,0.035) 边缘辉光 */
```

静态元素（卡片、面板、按钮）不使用阴影。

### 2.9 圆角

```css
--r-sm:  4px;
--r-md:  6px;   /* 小控件 */
--r-lg:  8px;   /* 卡片/面板/弹窗 */
--r-xl:  12px;
--r-full: 9999px;  /* 滚动条滑块 */
```

### 2.10 动效

```css
--fast:  120ms;  /* 保留 */
--normal: 180ms;
--slow:  280ms;
--ease:  cubic-bezier(0.16, 1, 0.3, 1);

/* Transition 默认使用 ease-out 150ms */
--transition-fast: 150ms ease-out;
```

### 2.11 滚动条

```css
::-webkit-scrollbar { width: 10px; }  /* 曾 5px */
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  background: transparent;
  border: 2px solid transparent;
  background-clip: content-box;
  border-radius: 999px;
  /* 颜色 = 44% of --text-secondary */
}
::-webkit-scrollbar-thumb { background-color: rgba(153,153,153,0.44); }
::-webkit-scrollbar-thumb:hover { background-color: rgba(120,120,120,0.50); }
::-webkit-scrollbar-thumb:active { background-color: var(--accent); }
```

---

## 3. 设置弹窗 (SettingsModal)

### 3.1 入口

Topbar 右上角月亮图标替换为齿轮图标（`IconSettings`），点击打开 SettingsModal。

### 3.2 布局

```
┌──── SettingsModal (720×520px, 居中弹窗, shadow-lg) ────┐
│  ╔══ 设置 ═══════════════════════════════ ✕ 关闭       │
│  ┌──────────┬──────────────────────────────────────┐   │
│  │ ⚙ 通用   │  右侧内容区，对应左侧选中页签           │   │
│  │ 🎨 主题   │                                      │   │
│  │          │                                      │   │
│  │          │                                      │   │
│  │ (空位)    │                                      │   │
│  └──────────┴──────────────────────────────────────┘   │
│                        [取消]  [保存]                    │
└────────────────────────────────────────────────────────┘
```

- **左侧页签栏**：160px 宽，图标 + 文字标签竖向排列。当前两个页签，为后续留空位。
- **右侧内容区**：560px 宽，`v-if` 切换内容。
- **Footer**：取消（丢弃未保存更改）+ 保存（持久化并应用）。

### 3.3 通用页签

当前仅占位，留空或显示 "更多设置即将推出"。

### 3.4 主题页签

不分二级页签，用 `<hr>` 分割线隔开三个区域：

#### 区域 1：基础设置

```
  模式      ○ 浅色  ● 深色
  强调色    [颜色选择器 #1A1C1E]
  字号      small  [medium]  large    (三选一按钮组)
```

- 模式：两个 radio 按钮
- 强调色：原生 `<input type="color">`，初始值 `#1A1C1E`
- 字号：三选一按钮组，影响 `--fs-base`（small=12px, medium=14px, large=15px）

#### 区域 2：皮肤预设

```
  自定义主题
  预设选择  [下拉: 默认 ▾]  [应用] [另存为...] [删除]
```

- 下拉列出所有 localStorage 中的预设
- "默认" 是内置预设（空 CSS，即纯令牌默认值），不可删除
- 选择预设后点"应用"立即生效
- "另存为"：弹出输入框输入名称，将当前 CSS 编辑区内容保存为新预设
- "删除"：删除当前选中预设（"默认"除外）

#### 区域 3：CSS 变量覆盖编辑器

```
  CSS 变量覆盖
  ┌──────────────────────────────────────────────┐
  │ :root {                                       │
  │   --accent: #1A1C1E;                         │
  │   --text: #555;                              │
  │ }                                             │
  └──────────────────────────────────────────────┘
                        [保存为预设]
```

- `<textarea>` 或最小化的 `<code>` 编辑器，monospace 字体
- 用户在编辑区写 CSS 变量覆盖
- "保存为预设"按钮等同于区域 2 的"另存为"

### 3.5 交互行为

- 打开弹窗时加载当前设置和活跃预设到表单中
- 修改基础设置（模式/强调色/字号）**即时生效**（实时预览）
- CSS 编辑器修改后点"应用"或"保存为预设"才生效
- 点击"取消"：丢弃所有未保存修改，恢复到打开弹窗前的状态
- 点击"保存"：持久化并关闭

### 3.6 即时预览实现

基础设置（模式/强调色/字号）通过 Pinia store 的 `watch` 实时写入 `document.documentElement.style.setProperty()`，修改即看到效果。取消时从 localStorage 重新加载覆盖。

CSS 编辑区的内容在用户点击"应用"后注入为 `<style id="theme-preset">` 标签，优先级高于 `theme.css`。

---

## 4. 数据模型与持久化

### 4.1 localStorage（全局）

Key: `"index-settings"`

```typescript
interface GlobalSettings {
  theme: ThemeSettings
  // 未来扩展: ai, general, ...
}

interface ThemeSettings {
  mode: 'light' | 'dark'
  accentColor: string            // hex, e.g. "#1A1C1E"
  fontSize: 'small' | 'medium' | 'large'
  presets: ThemePreset[]
}

interface ThemePreset {
  id: string                     // nanoid
  name: string
  css: string                    // CSS variable override snippet
  createdAt: string              // ISO 8601
  updatedAt: string
}
```

**默认值**（首次使用）:
```json
{
  "theme": {
    "mode": "light",
    "accentColor": "#1A1C1E",
    "fontSize": "medium",
    "presets": []
  }
}
```

### 4.2 仓库级（`.index/state.json`）

在现有 `state.json` 中新增一个字段：

```json
{
  "theme": "light",
  "activePresetId": null
}
```

- `theme`：已有字段，存亮/暗模式
- `activePresetId`：**新增**，引用 localStorage 中某个 `ThemePreset.id`。`null` 表示不使用预设（纯令牌默认值）。

### 4.3 SettingsStore（Pinia）

```
src/stores/settings.ts

state:
  themeMode: 'light' | 'dark'
  accentColor: string
  fontSize: 'small' | 'medium' | 'large'
  presets: ThemePreset[]
  activePresetId: string | null

actions:
  load()              — 从 localStorage 读取，无则用默认值
  save()              — 写回 localStorage
  applyTheme()        — 注入/更新 <style> 标签 + CSS 变量
  createPreset(name)  — 用当前编辑器内容新建预设
  updatePreset(id)    — 更新预设的 CSS 内容
  deletePreset(id)    — 删除预设
  setActivePreset(id) — 切换预设并写入 state.json
```

### 4.4 主题应用流程

```
应用启动
  → App.vue onMounted
    → settingsStore.load()          // localStorage → Pinia state
    → themeStore.init()             // 读 settingsStore.themeMode → 设 .dark class
    → settingsStore.applyTheme()    // 注入 CSS 变量（模式/强调色/字号）
    → 如 activePresetId != null → 从 localStorage 读取对应 preset 的 css 字符串，注入 <style id="theme-preset">
```

CSS 注入优先级（后加载覆盖前加载）：

```
1. theme.css 基础令牌           ← <link> 引入
2. <style id="theme-override">  ← 基础设置（mode/accentColor/fontSize）覆盖
3. <style id="theme-preset">    ← 活跃预设 CSS 覆盖（最高优先级）
```

---

## 5. CenterList 行高调整

`.row` 的 padding 从 `8px 12px 8px 8px` 收紧为 `5px 12px 5px 8px`，使行高比字体略高（约 24-26px 有效内容高度 vs 14px 字体），视觉更紧凑。

```css
/* src/components/CenterList.vue, .row */
padding: 4px 12px 4px 8px;
```

---

## 6. 文件变更清单

### 新建

| 文件 | 职责 |
|------|------|
| `src/stores/settings.ts` | 全局设置 Store，localStorage 读写，预设 CRUD |
| `src/components/SettingsModal.vue` | 设置弹窗外壳：Header + Tab 导航 + 内容区 + Footer |

### 修改

| 文件 | 改动 |
|------|------|
| `src/assets/theme.css` | 令牌值对齐 "Quiet Strength"；新增 `--text-heading`、`--border-strong`、`--link`；字体栈/权重更新；滚动条加宽；阴影值调整 |
| `src/components/Topbar.vue` | 月亮图标 → 齿轮图标（`IconSettings`）；点击打开 SettingsModal |
| `src/components/CenterList.vue` | `.row` 行 padding 收紧 |
| `src/App.vue` | 挂载 `<SettingsModal>`；onMounted 初始化主题 |
| `src/stores/theme.ts` | 从 settingsStore 读取初始 mode；toggle 时同步写回 |
| `src-tauri/src/commands/repo.rs` | `get_state` / `save_state` 支持 `activePresetId` 字段 |

### 不涉及

- 所有其他组件的 CSS 变量引用**名字不变**
- TypeManager 保持独立，不整合进设置
- 数据库 schema 无变更

---

## 7. 向后兼容

- `theme.css` 变量名不变，现有组件无需修改
- `.index/state.json` 新增 `activePresetId` 是可选字段，旧仓库无此字段时视为 `null`
- localStorage `"index-settings"` 无数据时使用默认值
- `--fw-*` 权重值变化可能细微影响文本粗细，属于预期内的视觉升级

---

## 8. 未来扩展点

- **通用设置页签**：语言、启动行为、快捷键
- **AI 设置页签**：provider、API key、模型选择
- **插件设置页签**：由插件动态注册
- **预设导入/导出**：JSON 文件导入导出预设
- **可视化表单编辑器**：在 CSS 编辑器之外，提供每个令牌分类的可视化控件（字体下拉、间距滑块等），作为混合模式的可视化部分。本期仅提供 CSS 代码编辑器作为 MVP
