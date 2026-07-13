<template>
  <div class="repo-card" @click="$emit('open')" :title="repo.path">
    <div class="card-icon">{{ displayIcon }}</div>
    <div class="card-name truncate">{{ displayName }}</div>
    <div class="card-meta">
      <span class="meta-item" v-if="repo.item_count != null">
        <TablerIcon name="file" :size="11" />
        {{ repo.item_count }}{{ $t('dashboard.items') }}
      </span>
      <span class="meta-item">
        <TablerIcon name="clock" :size="11" />
        {{ timeAgo }}
      </span>
    </div>
    <button class="card-menu-btn" @click.stop="showMenu = !showMenu" :title="$t('dashboard.removeRepo')">
      <TablerIcon name="dots" :size="14" />
    </button>
    <div v-if="showMenu" class="card-menu" @click.stop>
      <button class="danger" @click="doDelete">{{ $t('dashboard.removeRepo') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ManagedRepo } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const { t } = useI18n()

const props = defineProps<{ repo: ManagedRepo }>()
const emit = defineEmits<{ open: []; delete: [] }>()

const showMenu = ref(false)

const displayIcon = computed(() => props.repo.icon || '📁')
const displayName = computed(() => props.repo.name || basename(props.repo.path))

function basename(p: string): string {
  return p.split(/[/\\]/).pop() || p
}

const timeAgo = computed(() => {
  try {
    const then = new Date(props.repo.last_opened_at).getTime()
    const diff = Date.now() - then
    const mins = Math.floor(diff / 60000)
    if (mins < 1) return t('centerList.justNow')
    if (mins < 60) return `${mins}${t('centerList.minAgo')}`
    const hours = Math.floor(mins / 60)
    if (hours < 24) return `${hours}${t('centerList.hourAgo')}`
    const days = Math.floor(hours / 24)
    if (days === 1) return t('centerList.yesterday')
    if (days < 30) return `${days}${t('centerList.dayAgo')}`
    return `${Math.floor(days / 30)}${t('centerList.monthAgo')}`
  } catch {
    return ''
  }
})

function doDelete() {
  if (confirm(t('dashboard.confirmRemove'))) {
    showMenu.value = false
    emit('delete')
  }
}
</script>

<style scoped>
.repo-card {
  width: 180px; height: 140px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: 4px;
  padding: 16px 12px;
  cursor: pointer;
  position: relative;
  transition: border-color var(--slow) var(--ease), transform var(--slow) var(--ease), box-shadow var(--slow) var(--ease);
  user-select: none;
}
.repo-card:hover {
  border-color: var(--border-strong);
  transform: translateY(-2px);
  box-shadow: var(--shadow-sm);
}
.card-icon { font-size: 40px; line-height: 1; margin-bottom: 2px; }
.card-name {
  font-weight: var(--fw-semibold);
  font-size: var(--fs-sm);
  color: var(--text-heading);
  max-width: 100%;
  text-align: center;
}
.card-meta {
  display: flex; gap: 10px;
  font-size: var(--fs-xs); color: var(--text-secondary);
  margin-top: 2px;
}
.meta-item {
  display: inline-flex; align-items: center; gap: 3px;
}
.card-menu-btn {
  position: absolute; top: 6px; right: 6px;
  width: 26px; height: 26px; padding: 0;
  border: none; background: transparent;
  color: var(--text-muted); border-radius: var(--r-sm);
  cursor: pointer;
  display: none;
  align-items: center; justify-content: center;
}
.repo-card:hover .card-menu-btn { display: flex; }
.card-menu-btn:hover { background: var(--surface-hover); color: var(--text); }
.card-menu {
  position: absolute; top: 34px; right: 6px;
  background: var(--bg);
  border: 1px solid var(--border-strong);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-md);
  padding: 4px;
  z-index: 10;
  min-width: 120px;
}
.card-menu button {
  display: block; width: 100%;
  text-align: left; padding: 6px 10px;
  border: none; background: transparent;
  font-size: var(--fs-sm); cursor: pointer;
  color: var(--text);
  border-radius: var(--r-sm);
  height: auto;
}
.card-menu button:hover { background: var(--surface-hover); }
.card-menu button.danger { color: var(--danger); }
.card-menu button.danger:hover { background: var(--danger-subtle); }
</style>
