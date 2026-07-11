<template>
  <div class="icon-picker" ref="rootEl">
    <button class="picker-btn" @click="open = !open" type="button">
      <TablerIcon :name="modelValue || 'circle'" :size="18" />
      <TablerIcon name="chevron-right" :size="12" class="arrow" :class="{ open }" />
    </button>

    <Teleport to="body">
      <div v-if="open" class="picker-drop" :style="dropStyle" @click.stop>
        <input
          ref="searchEl"
          v-model="search"
          class="picker-search"
          placeholder="搜索图标或输入 emoji…"
          @keydown.escape="open = false"
        />
        <div class="picker-grid">
          <button
            v-for="name in filtered"
            :key="name"
            class="picker-item"
            :class="{ sel: modelValue === name }"
            :title="name"
            @click="select(name)"
          >
            <TablerIcon :name="name" :size="18" />
          </button>
        </div>
        <div v-if="search && !filtered.length && !isEmojiSearch" class="picker-empty">
          无匹配图标，输入 emoji 可直接使用
        </div>
        <div v-if="isEmojiSearch" class="picker-emoji-hint">
          按回车使用 "{{ search }}" 作为图标
        </div>
      </div>
    </Teleport>
  </div>

  <div v-if="open" class="picker-backdrop" @click="open = false" />
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { ICON_NAMES } from './TablerIcon.vue'
import TablerIcon from './TablerIcon.vue'

defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [v: string] }>()

const open = ref(false)
const search = ref('')
const rootEl = ref<HTMLElement | null>(null)
const searchEl = ref<HTMLInputElement | null>(null)
const dropStyle = ref<Record<string, string>>({})

const isEmojiSearch = computed(() =>
  /^(\p{Emoji}|\p{Emoji_Presentation}|\p{Emoji_Modifier_Base}|\p{Emoji_Component})+$/u.test(search.value.trim())
)

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return ICON_NAMES
  return ICON_NAMES.filter(n => n.includes(q) || q.split(/\s+/).every(w => n.includes(w)))
})

function select(name: string) {
  emit('update:modelValue', name)
  open.value = false
  search.value = ''
}

function position() {
  if (!rootEl.value) return
  const rect = rootEl.value.getBoundingClientRect()
  dropStyle.value = {
    position: 'fixed',
    top: (rect.bottom + 4) + 'px',
    left: Math.min(rect.left, window.innerWidth - 220) + 'px',
  }
}

watch(open, async (v) => {
  if (v) {
    position()
    await nextTick()
    searchEl.value?.focus()
  }
})
</script>

<style scoped>
.icon-picker {
  display: inline-flex;
  flex-shrink: 0;
}
.picker-btn {
  display: flex; align-items: center; gap: 2px;
  padding: 0 6px; height: 32px;
  border: 1px solid var(--border); border-radius: var(--r-md);
  background: var(--surface); cursor: pointer;
  transition: border-color var(--fast) var(--ease);
}
.picker-btn:hover { border-color: var(--accent); }
.arrow { transition: transform var(--fast) var(--ease); color: var(--text-muted); }
.arrow.open { transform: rotate(90deg); }

.picker-backdrop {
  position: fixed; inset: 0; z-index: 300; background: transparent;
}
.picker-drop {
  z-index: 301;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-lg);
  width: 210px;
  max-height: 260px;
  display: flex; flex-direction: column;
}
.picker-search {
  border: none; border-bottom: 1px solid var(--border);
  border-radius: var(--r-lg) var(--r-lg) 0 0;
  padding: 8px 10px; font-size: var(--fs-sm); height: 34px;
  flex-shrink: 0;
}
.picker-search:focus { outline: none; }
.picker-grid {
  display: grid; grid-template-columns: repeat(6, 1fr);
  gap: 2px; padding: 4px; overflow-y: auto; flex: 1;
}
.picker-item {
  display: flex; align-items: center; justify-content: center;
  width: 30px; height: 30px; padding: 0;
  border: none; border-radius: var(--r-sm);
  background: transparent; color: var(--text-secondary);
  cursor: pointer; transition: all var(--fast) var(--ease);
}
.picker-item:hover { background: var(--surface-hover); color: var(--text); }
.picker-item.sel { background: var(--accent-subtle); color: var(--accent); }
.picker-empty, .picker-emoji-hint {
  padding: 8px 10px; font-size: var(--fs-xs); color: var(--text-muted);
  border-top: 1px solid var(--border-light);
}
</style>
