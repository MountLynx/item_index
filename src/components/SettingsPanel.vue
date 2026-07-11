<template>
  <Teleport to="body">
    <div class="overlay" @click.self="$emit('close')">
      <div class="panel">
        <h3>设置</h3>

        <div class="sec">
          <h4>条目类型</h4>
          <div v-for="t in typeStore.types" :key="t.id" class="row clickable" :class="{ sel: t.id === selId }" @click="selId = selId === t.id ? null : t.id">
            <span class="row-left"><TablerIcon :name="t.icon" :size="18" /> {{ t.name }}</span>
            <button v-if="t.id > 2" class="ghost sm danger" @click.stop="removeType(t.id)">删除</button>
            <span v-else class="badge">预设</span>
          </div>
          <div class="add-row">
            <div class="icon-input">
              <TablerIcon :name="newIcon || 'circle'" :size="18" />
              <input v-model="newIcon" placeholder="输入tabler图标名" />
            </div>
            <input v-model="newName" placeholder="类型名称" @keydown.enter="addType" />
            <button class="primary sm" @click="addType"><TablerIcon name="plus" :size="15" /></button>
          </div>
          <div class="hint">
            <a href="https://tabler.io/icons" target="_blank" class="link">全部图标</a> &mdash; 输入图标名如 database, book, file
          </div>
        </div>

        <div v-if="sel" class="sec">
          <h4><TablerIcon :name="sel.icon" :size="16" /> {{ sel.name }} — 字段</h4>
          <div v-for="f in sel.fields" :key="f.id" class="row">
            <span class="text-muted">{{ f.field_type === 'checkbox' ? '☑' : '✎' }} {{ f.name }} <span class="tag-type">{{ f.field_type }}</span></span>
            <button class="ghost sm danger" @click="removeField(f.id)">删除</button>
          </div>
          <div class="add-row">
            <input v-model="fName" placeholder="字段名" />
            <select v-model="fType"><option value="text">文本</option><option value="checkbox">复选框</option></select>
            <button class="primary sm" @click="addField"><TablerIcon name="plus" :size="15" /></button>
          </div>
        </div>

        <div class="acts"><button @click="$emit('close')">关闭</button></div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTypeStore } from '@/stores/types'
import TablerIcon from './TablerIcon.vue'

const typeStore = useTypeStore()
const selId = ref<number | null>(null)
const newName = ref('')
const newIcon = ref('')
const fName = ref('')
const fType = ref('text')
const sel = computed(() => typeStore.types.find(t => t.id === selId.value))

async function addType() { if (newName.value.trim()) { await typeStore.create(newName.value.trim(), newIcon.value || 'file'); newName.value = ''; newIcon.value = '' } }
async function removeType(id: number) { if (confirm('确定删除？')) await typeStore.remove(id) }
async function addField() { if (selId.value && fName.value.trim()) { await typeStore.addField(selId.value, fName.value.trim(), fType.value); fName.value = '' } }
async function removeField(id: number) { await typeStore.removeField(id) }

defineEmits<{ close: [] }>()
</script>

<style scoped>
.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.2); display: flex; align-items: center; justify-content: center; z-index: 200; }
.panel { background: var(--surface); border-radius: var(--r-xl); padding: 24px; min-width: 460px; max-height: 80vh; overflow-y: auto; box-shadow: var(--shadow-lg); border: 1px solid var(--border); }
h3 { font-size: var(--fs-lg); font-weight: var(--fw-semibold); margin: 0 0 16px; }
h4 { font-size: var(--fs-sm); color: var(--text-secondary); margin: 0 0 8px; font-weight: var(--fw-semibold); }
.sec { margin-bottom: 20px; }
.row { display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; font-size: var(--fs-sm); border-radius: var(--r-sm); }
.row.clickable { cursor: pointer; transition: background var(--fast) var(--ease); }
.row.clickable:hover { background: var(--surface-hover); }
.row.sel { background: var(--accent); color: var(--accent-fg); }
.row.sel .text-muted, .row.sel .tag-type, .row.sel .badge { color: rgba(255,255,255,0.7); }
.row-left { display: flex; align-items: center; gap: 6px; }
.badge { font-size: var(--fs-xs); color: var(--text-muted); }
.tag-type { font-size: 10px; color: var(--text-muted); background: var(--bg); padding: 0 4px; border-radius: 2px; }
.sm { font-size: var(--fs-xs); height: 28px; }
.danger { color: var(--danger); }
.add-row { display: flex; gap: 4px; margin-top: 8px; }
.add-row input, .add-row select { font-size: var(--fs-sm); }
.icon-input {
  display: flex; align-items: center; gap: 4px; padding: 0 8px;
  border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface);
}
.icon-input input {
  border: none; background: transparent; padding: 5px 8px; width: 180px;
  font-size: var(--fs-sm);
}
.icon-input input::placeholder { color: var(--text-muted); font-style: italic; }
.icon-input input:focus { outline: none; }
.icon-input:focus-within { border-color: var(--accent); }
.hint { font-size: var(--fs-xs); color: var(--text-muted); margin-top: 6px; }
.link { color: var(--accent); text-decoration: none; }
.link:hover { text-decoration: underline; }
.acts { display: flex; justify-content: flex-end; margin-top: 16px; }
</style>
