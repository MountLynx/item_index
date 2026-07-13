<template>
  <header class="titlebar" data-tauri-drag-region>
    <!-- Left: Logo + repo name -->
    <div class="tb-left">
      <TablerIcon name="database" :size="18" :stroke="1.5" />
      <span class="tb-repo">{{ repoStore.repoPath ? basename(repoStore.repoPath) : 'Index' }}</span>
    </div>

    <!-- Center spacer (drag region) -->
    <div class="tb-spacer" />

    <!-- Right: action buttons + window controls -->
    <div class="tb-right">
      <button class="primary sm" @click.stop="$emit('newItem')">
        <TablerIcon name="plus" :size="15" /> {{ $t('topbar.newItem') }}
      </button>
      <button class="icon-btn tb-icon" @click.stop="openSettings" :title="$t('common.settings')">
        <TablerIcon name="settings" :size="17" />
      </button>
      <button class="icon-btn tb-icon" @click.stop="openDashboard" :title="$t('topbar.manageRepos')">
        <TablerIcon name="database" :size="17" />
      </button>

      <span class="tb-sep" />

      <!-- Window controls -->
      <button class="icon-btn tb-ctrl" @click.stop="winMinimize" title="Minimize">
        <TablerIcon name="minus" :size="16" />
      </button>
      <button class="icon-btn tb-ctrl" @click.stop="winToggleMaximize" title="Maximize">
        <TablerIcon name="square" :size="14" />
      </button>
      <button class="icon-btn tb-ctrl tb-close" @click.stop="winClose" title="Close">
        <TablerIcon name="x" :size="17" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { inject } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRepoStore } from '@/stores/repo'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useI18n } from 'vue-i18n'
import TablerIcon from './TablerIcon.vue'

useI18n()
const repoStore = useRepoStore()

defineEmits<{ newItem: [] }>()

const openSettings = inject<() => void>('openSettings', () => {})

function openDashboard() {
  invoke('open_dashboard_window')
}

function basename(p: string): string { return p.split(/[/\\]/).pop() || p }

const win = getCurrentWindow()

function winMinimize() { win.minimize() }
function winToggleMaximize() { win.toggleMaximize() }
function winClose() { win.close() }
</script>

<style scoped>
.titlebar {
  height: 40px; flex-shrink: 0;
  display: flex; align-items: center;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  user-select: none; -webkit-app-region: drag;
}

.tb-left {
  display: flex; align-items: center; gap: 8px;
  padding-left: 12px;
  color: var(--accent);
  min-width: 0; -webkit-app-region: no-drag;
}
.tb-repo {
  font-weight: 620; font-size: var(--fs-sm);
  color: var(--text);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

.tb-spacer { flex: 1; }

.tb-right {
  display: flex; align-items: center; gap: 2px;
  padding-right: 6px;
  -webkit-app-region: no-drag;
}

.tb-icon {
  width: 30px; height: 30px; color: var(--text-secondary);
}
.tb-icon:hover { color: var(--text); background: var(--surface-hover); }

.sm { font-size: var(--fs-xs); height: 28px; padding: 4px 10px; }

.tb-sep {
  width: 1px; height: 20px;
  background: var(--border-strong);
  margin: 0 6px;
}

/* Window control buttons */
.tb-ctrl {
  width: 34px; height: 28px;
  color: var(--text-secondary);
  border-radius: var(--r-sm);
}
.tb-ctrl:hover { background: var(--surface-hover); color: var(--text); }
.tb-close:hover { background: #C42B1C; color: #fff; }
</style>
