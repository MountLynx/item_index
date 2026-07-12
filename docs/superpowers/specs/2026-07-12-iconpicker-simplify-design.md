# IconPicker 简化重设计

**日期**: 2026-07-12
**状态**: 已批准
**关联**: `src/components/IconPicker.vue`、`src/assets/icon-names.ts`、`src/components/TablerIcon.vue`

## 问题背景

当前 IconPicker 有 187 个图标分组，通过横向 tabs 导航，配合虚拟滚动。实际使用中：

- **图标不加载**: `loadIcons()` 动态 import 失败时静默吞错，`loadErr` 变量从未展示给用户
- **横向滚动不生效**: 187 个 tab 按钮的 `.pick-tabs` 容器缺少 `flex-wrap: nowrap`，被父容器压缩
- **分组 tabs 是过度设计**: 用户基本不会翻 187 个 tab，直接搜索才是定位图标的有效方式

## 设计目标

1. 去掉分组 tabs，简化交互为「常用图标 + 搜索」
2. 保持搜索+虚拟滚动的性能优势
3. 修复图标加载失败时的错误提示
4. 外部 API（props/emits）不变，`TypeManager.vue` 零改动

## 整体架构

```
IconPicker 面板打开时：

┌─────────────────────────────┐
│  🔍 搜索图标…               │  ← 搜索框（自动聚焦）
├─────────────────────────────┤
│  （无搜索词：常用图标）      │
│  circle  file  folder  ...  │  ← 全量渲染 ~52 个，无虚拟滚动
│                             │
│  （有搜索词：搜索结果）      │
│  匹配结果（上限 200 个）     │  ← 虚拟滚动 grid（6 列）
│                             │
│  无匹配 → "无结果，可输入    │
│             emoji 直接使用"  │
│  加载失败 → 错误提示 + 重试  │
│  加载中   → "加载中…"       │
└─────────────────────────────┘
```

**核心变化：**
- 删除 `pick-tabs`（分组标签栏）
- 删除 `activeGroup`、`currentIcons`、`groups` 等分组相关状态
- 新增「常用图标」静态列表，不依赖异步加载，面板打开即可显示

## 数据流

### Props / Emits（不变）

```ts
props: { modelValue: string }           // 当前选中图标名
emit:  { 'update:modelValue': [v: string] }
```

### 数据加载

```ts
// 扁平化：ICON_GROUPS[].icons → 单个 string[]
const iconNames = ref<string[]>([])

async function loadIcons() {
  if (iconNames.value.length) return
  try {
    const mod = await import('@/assets/icon-names')
    const seen = new Set<string>()
    iconNames.value = []
    for (const g of mod.ICON_GROUPS) {
      for (const n of g.icons) {
        if (!seen.has(n)) {
          seen.add(n)
          iconNames.value.push(n)
        }
      }
    }
  } catch (e) {
    loadErr.value = String(e)
    console.error('Failed to load icon names:', e)
  }
}
```

### 常用图标（静态常量）

```ts
const COMMON: string[] = [
  'circle', 'file', 'folder', 'star', 'heart', 'check', 'x',
  'plus', 'minus', 'search', 'settings', 'user', 'users', 'calendar',
  'clock', 'bell', 'mail', 'message', 'photo', 'camera', 'music',
  'video', 'map', 'lock', 'key', 'trash', 'edit', 'pencil',
  'book', 'bookmark', 'tag', 'flag', 'link', 'globe', 'home',
  'phone', 'download', 'upload', 'share', 'filter', 'copy',
  'database', 'code', 'chart-bar', 'list', 'lightbulb', 'info-circle',
  'alert-circle', 'help-circle', 'rocket', 'package'
]
```

不依赖动态 import，组件初始化即可渲染。

### 搜索逻辑

```ts
const search = ref('')
const MAX_RESULTS = 200

const results = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return [] as string[]
  const matched: string[] = []
  for (const name of iconNames.value) {
    if (name.includes(q)) matched.push(name)
    if (matched.length >= MAX_RESULTS) break
  }
  return matched
})
```

## 渲染模式

### 两种模式，由搜索词决定（互斥）

| 模式 | 触发 | 渲染 |
|------|------|------|
| 常用图标 | `search` 为空 | 全量渲染 ~52 个，无虚拟滚动 |
| 搜索结果 | `search` 非空 | 虚拟滚动，上限 200 个 |

### 虚拟滚动参数（不变）

```
COLS = 6, ROW_H = 32, VISIBLE_ROWS = 9, BUFFER = 3
```

- 常用图标：直接 `slice` 排成 rows 渲染
- 搜索结果：`totalHeight`、`offsetY`、`visibleRows` 同之前逻辑，数据源改为 `results`（平铺 `string[]`）

## 状态变量精简

| 之前 | 之后 |
|------|------|
| `open`, `search`, `rootEl`, `searchEl`, `gridEl` | 不变 |
| `dropStyle` | 不变 |
| `groups`, `activeGroup`, `scrollTop`, `loadErr` | `iconNames`, `scrollTop`, `loadErr` |

删除：`activeGroup`、`groups`、`currentIcons`（分组相关）。
新增：`iconNames`（扁平 `string[]`）。

## 错误处理和边界情况

### 加载失败

- `loadErr` 被赋值后，模板展示错误消息 + **重试按钮**
- 重试：重置 `iconNames` 和 `loadErr`，重新调用 `loadIcons()`
- 常用图标在加载失败时仍然可用（不依赖异步数据）

### 空状态

| 状态 | 展示 |
|------|------|
| 搜索无匹配 | "无匹配，可输入 emoji 直接使用"（已有） |
| 数据加载中 | "加载中…"（已有） |
| 数据加载失败 | "⚠ 图标加载失败" + 重试按钮（新增） |

### 键盘操作

- `Enter`：不适用（点击选择）
- `Escape`：关闭面板，不改变选中值（已有）
- 搜索框自动聚焦：面板打开时（已有）

### 定位（不变）

- Teleport 到 `body`
- `position()` 基于按钮 `getBoundingClientRect()` 计算 top/left
- `Math.min` 防溢出窗口边界

## 影响范围

| 文件 | 变化 |
|------|------|
| `src/components/IconPicker.vue` | 重写（模板 + 逻辑精简 ~40%） |
| `src/components/TypeManager.vue` | **无改动**（API 不变） |
| `src/assets/icon-names.ts` | **无改动**（数据格式不变，扁平化在使用侧完成） |
| `src/components/TablerIcon.vue` | **无改动** |

## 不做的事情

- 不更改图标字体方案（继续使用 `@tabler/icons-webfont`）
- 不添加"收藏图标"功能
- 不添加图标颜色/大小自定义
- 不改变 Teleport 定位策略
