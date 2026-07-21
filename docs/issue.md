# Known Issues

> 记录已识别但尚未修复的隐患。按风险等级排列。

---

## 🔴 高风险

### #3 无分页/虚拟滚动 — 大数据量性能瓶颈

**位置**：`src/components/CenterList.vue` + `src/stores/items.ts`

**现象**：
- `CenterList.vue` 用 `v-for="item in items"` 全量渲染所有 item
- `items.ts::fetchList()` 一次性加载全部数据，无 `LIMIT/OFFSET`
- 后端 `list_items` 也无分页支持

**后果**：仓库累积上千条 item 后，DOM 节点爆炸 + 内存飙升，界面卡死。

**建议方案**：
- 后端 `list_items` 加 `page`/`pageSize` 参数
- 前端用 `vue-virtual-scroller` 或实现无限滚动（Intersection Observer）

**相关**: `src-tauri/src/commands/items.rs` `list_items`

---

## 🟡 中风险

### #5 `saveTimer` 是模块级全局变量，非 store 实例级

**位置**：`src/stores/items.ts:6`

```ts
let saveTimer: ReturnType<typeof setTimeout> | null = null  // 模块顶层！
```

**后果**：如果将来出现多个 Pinia store 实例（如多窗口），它们会共享同一个 debounce timer，导致属性保存相互覆盖。

**建议方案**：将 `saveTimer` 移入 `defineStore` 闭包内。

---

### #6 `fetchList` 每次调用都动态 import workspace/types store

**位置**：`src/stores/items.ts:19-31`

```ts
if (typeIds === undefined) {
    const { useWorkspaceStore } = await import('@/stores/workspace')
    const { useTypeStore } = await import('@/stores/types')
    // ...
}
```

**后果**：每次点击 sidebar 的 group/tag 筛选都会触发动态 `import()`。虽然 Vite 会缓存模块，但仍有不必要的异步开销。每次还要遍历 `typeStore.types` 做 name→id 映射。

**建议方案**：在 store 初始化时一次性解析 workspace type IDs 并缓存，后续 `fetchList` 直接复用。

---

### #7 `PropertiesForm` 直接 mutate detail，无乐观更新回滚

**位置**：`src/components/PropertiesForm.vue:58-66`

```ts
function setValue(name, value) {
    const p = props.detail!.item.properties as Record<string, unknown>
    p[name] = value  // 直接 mutate，立即反映到 UI
}
```

**后果**：如果后续 `saveProperties` 请求失败（网络错误、后端拒绝等），UI 上已经显示新值，用户以为保存成功实际未保存。

**建议方案**：
- 保存前 snapshot 旧值，请求失败后 restore
- 或在 `saveProperties` 中传入 (id, key, value) 替代全量 properties，由 store 做乐观更新管理

---

## 🟢 低风险 / 可改进

### ID 48bit 碰撞概率

**位置**：`src-tauri/src/commands/items.rs:17-21`

12 位 hex（48 bit 熵）。对个人桌面应用可接受，但不如 UUID v4（122 bit）。如果未来支持多用户协作、跨仓库合并，建议升级。

---

### `read_dir_recursive` 无深度/数量保护

**位置**：`src-tauri/src/commands/items.rs:23-38`

深层嵌套目录或符号链接循环可能导致栈溢出或无限递归。建议加最大深度限制（如 20 层）。

---

### `deleteItem` 使用原生 `confirm()`

**位置**：`src/components/CenterList.vue:65`

`window.confirm()` 是阻塞式原生弹窗，体验差且无法自定义样式/i18n。建议用自定义 Confirm Dialog 组件。

---

### 选中 Item 每次拉全量文件树

**位置**：`src/stores/items.ts:40-48` → `get_item` → `read_dir_recursive`

`select(id)` 每次完整拉取 `ItemDetail`（含 groups + tags + files 树）。当 item 附件目录很大时，延迟明显。建议 files 懒加载或分页。

---

### 修改 group/tag 后重新 fetch 全量 detail

**位置**：`src/components/RightPanel.vue:210,217,223,229`

添加/删除 group 或 tag 后调用 `itemStore.select(id)` 重新拉取全量 `ItemDetail`，而非仅局部更新 groups/tags 数组。

---

### `properties` 完全无类型安全

**位置**：`src/types/bindings.ts:25` — `properties: Record<string, unknown>`

字段值和声明的 `field_type` 之间没有编译期检查，类型不匹配只能在运行时发现。

---

*Last updated: 2026-07-20*
