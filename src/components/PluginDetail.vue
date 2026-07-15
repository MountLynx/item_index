<template>
  <div class="modal-overlay" @click.self="emit('cancel')">
    <div class="detail-dialog">
      <div class="dd-header">
        <h3>{{ mode === 'overwrite' ? '覆盖插件' : '导入插件预览' }}</h3>
        <button class="icon-btn" @click="emit('cancel')"><TablerIcon name="x" :size="16" /></button>
      </div>
      <div class="dd-body">
        <div class="dd-preview">
          <div class="dd-icon"><TablerIcon :name="manifest.icon" :size="32" /></div>
          <div class="dd-info">
            <h4>{{ manifest.title }}</h4>
            <div class="dd-version">v{{ manifest.version }}</div>
            <div class="dd-author" v-if="manifest.author">{{ manifest.author }}</div>
            <div class="dd-desc" v-if="manifest.description">{{ manifest.description }}</div>
            <div class="dd-meta">
              <span>扩展点: {{ manifest.extends }}</span>
              <span v-if="manifest.requiresFields.length">字段需求: {{ manifest.requiresFields.join(', ') }}</span>
            </div>
          </div>
        </div>
        <div v-if="mode === 'overwrite' && usage" class="dd-warning">
          <p>⚠ 全局库已存在此插件，将覆盖现有版本。</p>
          <div v-if="usage.repos.length" class="dd-refs"><strong>受影响的仓库:</strong><ul><li v-for="r in usage.repos" :key="r">{{ r }}</li></ul></div>
          <div v-if="usage.presets.length" class="dd-refs"><strong>受影响的预设:</strong><ul><li v-for="p in usage.presets" :key="p">{{ p }}</li></ul></div>
          <p class="dd-note">覆盖后仓库中的副本不会自动更新，需手动重新安装。</p>
        </div>
        <div v-if="mode === 'overwrite'" class="dd-warning">
          <p>⚠ 覆盖将替换全局库中的插件版本，仓库中已安装的旧版本不受影响（需手动更新）。</p>
        </div>
      </div>
      <div class="dd-footer">
        <button @click="emit('cancel')">取消</button>
        <button class="primary" @click="emit('confirm')">{{ mode === 'overwrite' ? '确认覆盖' : '确认导入' }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PluginManifest, PluginUsage } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

defineProps<{
  manifest: PluginManifest
  usage: PluginUsage | null
  mode: 'import' | 'overwrite'
}>()

const emit = defineEmits<{ confirm: []; cancel: [] }>()
</script>

<style scoped>
.modal-overlay { position: fixed; inset: 0; z-index: 1001; background: rgba(0,0,0,0.4); display: flex; align-items: center; justify-content: center; }
.detail-dialog { width: 420px; max-height: 80vh; background: var(--bg); border-radius: var(--r-lg); box-shadow: 0 8px 32px rgba(0,0,0,0.2); display: flex; flex-direction: column; overflow: hidden; }
.dd-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; border-bottom: 1px solid var(--border); }
.dd-header h3 { margin: 0; font-size: var(--fs-base); font-weight: var(--fw-semibold); }
.dd-body { flex: 1; overflow-y: auto; padding: 20px; display: flex; flex-direction: column; gap: 16px; }
.dd-preview { display: flex; gap: 14px; }
.dd-icon { flex-shrink: 0; color: var(--text-secondary); }
.dd-info h4 { margin: 0 0 4px; font-size: var(--fs-base); font-weight: var(--fw-semibold); }
.dd-version { font-size: var(--fs-sm); color: var(--text-muted); }
.dd-author { font-size: var(--fs-sm); color: var(--text-secondary); margin-top: 2px; }
.dd-desc { font-size: var(--fs-sm); color: var(--text-muted); margin-top: 4px; }
.dd-meta { font-size: var(--fs-xs); color: var(--text-muted); margin-top: 6px; display: flex; gap: 12px; }
.dd-warning { background: var(--warning-bg, #fff3cd); border: 1px solid var(--warning-border, #ffc107); border-radius: var(--r-md); padding: 12px; font-size: var(--fs-sm); }
.dd-warning p { margin: 0; }
.dd-refs { margin-top: 8px; }
.dd-refs ul { margin: 4px 0 0 16px; padding: 0; }
.dd-refs li { font-size: var(--fs-xs); }
.dd-note { font-size: var(--fs-xs); color: var(--text-muted); margin-top: 8px; }
.dd-footer { display: flex; justify-content: flex-end; gap: 8px; padding: 16px 20px; border-top: 1px solid var(--border); }
.icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; padding: 0; border: none; border-radius: var(--r-sm); background: transparent; color: var(--text-secondary); cursor: pointer; }
.icon-btn:hover { background: var(--surface-hover); }
</style>
