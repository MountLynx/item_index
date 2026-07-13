# Theme Fixes — 删除按钮样式、深色模式、颜色设置扩展、表单优先级

**日期**: 2026-07-13
**状态**: 设计完成

---

## 1. 删除按钮 `.danger` 样式修复

**问题**：`button.danger` 红色背景 + `-webkit-text-stroke` 图标渲染导致图标不可见。

**修复**：danger 按钮改为幽灵样式——透明背景，红色前景，hover 时淡红背景。

```css
button.danger {
  background: transparent;
  color: var(--danger);
  border-color: transparent;
}
button.danger:hover {
  background: var(--danger-subtle);
  color: var(--danger-hover);
}
```

影响范围：`theme.css` 全局 button.danger 规则；TypeManager 删除按钮、CenterList 右键菜单"删除条目"等自动适配。

---

## 2. 深色模式强调色确认

**问题**：暗色模式强调色 `#F4F4F5`（近白）应正常显示，但初始化流程曾缺失 `applyTheme()`。

**修复**：已在上一轮修复中加入 `onMounted` 的 `settingsStore.applyTheme()`。验证暗色模式下 Tokens 值的完整性——`.dark` 中 `--accent: #F4F4F5`、`--accent-fg: #1E1E1E` 均已在 theme.css 中定义，无代码变更。

---

## 3. 基础颜色设置扩展

SettingsModal 主题页签"区域 1"从 3 行扩展为 5 行：

```
  模式        ○ 浅色  ● 深色
  强调色      [🎨 #1A1C1E]                       ← 手动
  背景色      [🎨 #FFFFFF]                       ← 新增，手动
  字体色      [🎨 #333333] [自动 ✨]              ← 新增，自动+可手动
  字号        small  [medium]  large
```

### 数据模型

SettingsStore 新增 state：

```ts
bgColor: string         // hex, 默认 "#FFFFFF"
textColor: string       // hex, 默认 "#333333"
textColorAuto: boolean  // 默认 true
```

GlobalSettings 对应扩展：

```json
{
  "theme": {
    "mode": "light",
    "accentColor": "#1A1C1E",
    "bgColor": "#FFFFFF",
    "textColor": "#333333",
    "textColorAuto": true,
    "fontSize": "medium",
    "presets": []
  }
}
```

### 自动计算逻辑

**强调前景色**（纯自动，不暴露在 UI）：
```ts
function computeAccentFg(accentHex: string): string {
  const { r, g, b } = hexToRgb(accentHex)
  const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b  // 0-255
  return lum > 128 ? '#1E1E1E' : '#FFFFFF'
}
```

**字体色自动值**（由背景色计算）：
```ts
function computeTextColor(bgHex: string): string {
  const { r, g, b } = hexToRgb(bgHex)
  const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
  return lum > 128 ? '#333333' : '#F4F4F5'
}
```

**背景色联动**：修改 bgColor 时，同步更新 `--surface`（比 bgColor 略暗——亮度 × 0.97）：

```ts
function computeSurface(bgHex: string): string {
  const { r, g, b } = hexToRgb(bgHex)
  const factor = 0.97
  return rgbToHex(Math.round(r * factor), Math.round(g * factor), Math.round(b * factor))
}
```

### applyTheme() 更新

`theme-override` style 标签内容扩展：

```css
:root {
  --accent: ${accentColor};
  --accent-fg: ${computeAccentFg(accentColor)};
  --bg: ${bgColor};
  --surface: ${computeSurface(bgColor)};
  --text: ${textColorAuto ? computeTextColor(bgColor) : textColor};
  --text-heading: ${textColorAuto ? computeTextColor(bgColor) : textColor};
  --fs-base: ${FONT_SIZE_MAP[fontSize]};
  --fs-sm: ${fontSize === 'small' ? '0.75rem' : '0.8125rem'};
}
```

---

## 4. 表单 > CSS 优先级 + 同步按钮

### 优先级链

```
theme.css
  < style#theme-override      ← 表单基础设置（高优先级）
  < style#theme-preset        ← CSS 自定义预设（低优先级）
```

表单值修改时，确保 `#theme-override` 在 DOM 中位于 `#theme-preset` 之后（后加载胜出）。通过在 watch 中移除再 append 实现。

### CSS 解析（回填表单）

提取 CSS 字符串中的颜色变量：

```ts
function parseCSSVariables(css: string): Partial<ThemeOverrides> {
  const result: Partial<ThemeOverrides> = {}
  const extract = (name: string): string | null => {
    const m = css.match(new RegExp(`${name}\\s*:\\s*([^;]+)`))
    return m ? m[1].trim() : null
  }
  const v = extract('--accent')
  if (v) result.accentColor = v
  const b = extract('--bg')
  if (b) result.bgColor = b
  const t = extract('--text')
  if (t) result.textColor = t
  // ... 等
  return result
}
```

### "与自定义主题保持一致"按钮

按钮位于 CSS 编辑区下方：

```
  CSS 变量覆盖
  ┌──────────────────────┐
  │ :root { ... }        │
  └──────────────────────┘
  [应用] [保存为预设] [与自定义主题保持一致]
```

点击行为：
1. 解析 `localCSS` 中的变量值
2. 用解析出的值更新 `localAccentColor`、`localBgColor`、`localTextColor` 等表单控件
3. 同时将 `localTextColorAuto` 设为 `false`（用户显式选择了覆盖，关闭自动）
4. 表单 watch 触发 → `theme-override` 重新注入 → 表单值生效

### "应用"按钮行为更新

原：仅注入 CSS → `theme-preset`。  
新：注入 CSS → `theme-preset`，同时**解析 CSS 值回填表单**（同同步按钮逻辑），然后确保 `theme-override` 在 `theme-preset` 之后。

---

## 5. 文件变更

| 文件 | 改动 |
|------|------|
| `src/assets/theme.css` | button.danger 改为幽灵样式 |
| `src/stores/settings.ts` | 新增 bgColor/textColor/textColorAuto state；applyTheme 扩展变量注入；新增 computeAccentFg/computeTextColor/computeSurface/parseCSSVariables 工具函数 |
| `src/components/SettingsModal.vue` | 新增背景色/字体色行（含自动标记）；新增"与自定义主题保持一致"按钮；"应用"按钮增加回填逻辑；表单值修改时确保 style 标签优先级 |

---

## 6. 向后兼容

- localStorage `"index-settings"` 旧数据无 `bgColor`/`textColor`/`textColorAuto` → load() 中用默认值补全
- `accent-fg` 由自动计算覆盖，CSS 预设中的 `--accent-fg` 设置将被忽略
- `button.danger` 样式变更影响全局所有 danger 按钮，均为预期行为
