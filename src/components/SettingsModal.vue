<template>
  <Teleport to="body">
    <div v-if="visible" class="settings-overlay" @click.self="onCancel">
      <div class="settings-window">
        <!-- Header -->
        <div class="settings-header">
          <h2>设置</h2>
          <button class="icon-btn" @click="onCancel">
            <TablerIcon name="x" :size="18" />
          </button>
        </div>

        <!-- Body -->
        <div class="settings-body">
          <!-- Left tabs -->
          <nav class="settings-tabs">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              class="tab-btn"
              :class="{ active: activeTab === tab.id }"
              @click="activeTab = tab.id"
            >
              <TablerIcon :name="tab.icon" :size="18" />
              <span>{{ tab.label }}</span>
            </button>
          </nav>

          <!-- Right content -->
          <div class="settings-content">
            <!-- General tab (placeholder) -->
            <div v-if="activeTab === 'general'" class="tab-panel">
              <p class="placeholder-text">更多设置即将推出</p>
            </div>

            <!-- Theme tab -->
            <div v-else-if="activeTab === 'theme'" class="tab-panel">
              <!-- Section 1: Basic settings -->
              <div class="setting-section">
                <div class="setting-row">
                  <label>模式</label>
                  <div class="radio-group">
                    <label class="radio-label">
                      <input type="radio" v-model="localMode" value="light" /> 浅色
                    </label>
                    <label class="radio-label">
                      <input type="radio" v-model="localMode" value="dark" /> 深色
                    </label>
                  </div>
                </div>
                <div class="setting-row">
                  <label>强调色</label>
                  <input type="color" v-model="localAccentColor" class="color-input" />
                </div>
                <div class="setting-row">
                  <label>字号</label>
                  <div class="btn-group">
                    <button
                      v-for="s in fontSizes"
                      :key="s"
                      :class="{ active: localFontSize === s }"
                      @click="localFontSize = s"
                    >{{ s }}</button>
                  </div>
                </div>
              </div>

              <hr class="section-divider" />

              <!-- Section 2: Preset management -->
              <div class="setting-section">
                <label class="section-title">自定义主题</label>
                <div class="preset-row">
                  <select v-model="selectedPresetId" class="preset-select">
                    <option :value="null">默认</option>
                    <option v-for="p in settingsStore.presets" :key="p.id" :value="p.id">
                      {{ p.name }}
                    </option>
                  </select>
                  <button class="sm" @click="applyPreset">应用</button>
                  <button class="sm" @click="onSaveAs">另存为...</button>
                  <button
                    v-if="selectedPresetId"
                    class="sm danger"
                    @click="onDeletePreset"
                  >删除</button>
                </div>
              </div>

              <hr class="section-divider" />

              <!-- Section 3: CSS editor -->
              <div class="setting-section">
                <label class="section-title">CSS 变量覆盖</label>
                <textarea
                  v-model="localCSS"
                  class="css-editor"
                  spellcheck="false"
                  placeholder=":root {&#10;  --accent: #1A1C1E;&#10;}"
                ></textarea>
                <button class="sm" @click="onSaveAs">保存为预设</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="settings-footer">
          <button @click="onCancel">取消</button>
          <button class="primary" @click="onSave">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useThemeStore } from '@/stores/theme'
import TablerIcon from './TablerIcon.vue'

const settingsStore = useSettingsStore()
const themeStore = useThemeStore()

const tabs = [
  { id: 'general', icon: 'settings', label: '通用' },
  { id: 'theme', icon: 'palette', label: '主题' },
]

const fontSizes = ['small', 'medium', 'large'] as const

const visible = ref(false)
const activeTab = ref<string>('theme')

// Local copies for live preview + cancel support
const localMode = ref<'light' | 'dark'>('light')
const localAccentColor = ref('#1A1C1E')
const localFontSize = ref<'small' | 'medium' | 'large'>('medium')
const localCSS = ref('')
const selectedPresetId = ref<string | null>(null)

// Snapshot for cancel restore
let snapshot: {
  mode: 'light' | 'dark'
  accentColor: string
  fontSize: 'small' | 'medium' | 'large'
  presetCSS: string
  activePresetId: string | null
} | null = null

function open(): void {
  // Take snapshot
  snapshot = {
    mode: settingsStore.themeMode,
    accentColor: settingsStore.accentColor,
    fontSize: settingsStore.fontSize,
    presetCSS: settingsStore.presetCSS,
    activePresetId: settingsStore.activePresetId,
  }

  // Init local state from store
  localMode.value = settingsStore.themeMode
  localAccentColor.value = settingsStore.accentColor
  localFontSize.value = settingsStore.fontSize
  localCSS.value = settingsStore.presetCSS
  selectedPresetId.value = settingsStore.activePresetId

  visible.value = true
}

function close(): void {
  visible.value = false
  snapshot = null
}

// Live preview: watch local basic settings and apply immediately
watch([localMode, localAccentColor, localFontSize], () => {
  themeStore.mode = localMode.value
  themeStore.apply()
  // Temporarily set store state for live preview (will be restored on cancel)
  settingsStore.accentColor = localAccentColor.value
  settingsStore.fontSize = localFontSize.value
  settingsStore.applyTheme()
})

function applyPreset(): void {
  if (selectedPresetId.value) {
    const preset = settingsStore.presets.find(p => p.id === selectedPresetId.value)
    localCSS.value = preset?.css ?? ''
    // Apply the CSS immediately
    let el = document.getElementById('theme-preset') as HTMLStyleElement | null
    if (!el) {
      el = document.createElement('style')
      el.id = 'theme-preset'
      document.head.appendChild(el)
    }
    el.textContent = localCSS.value
  } else {
    localCSS.value = ''
    document.getElementById('theme-preset')?.remove()
  }
}

function onSaveAs(): void {
  const name = prompt('输入预设名称：')
  if (!name?.trim()) return
  const id = settingsStore.createPreset(name.trim(), localCSS.value)
  selectedPresetId.value = id
}

function onDeletePreset(): void {
  if (!selectedPresetId.value) return
  const preset = settingsStore.presets.find(p => p.id === selectedPresetId.value)
  if (!preset) return
  if (!confirm(`确定删除预设 "${preset.name}"？`)) return
  settingsStore.deletePreset(selectedPresetId.value)
  selectedPresetId.value = null
  localCSS.value = ''
}

async function onSave(): Promise<void> {
  settingsStore.themeMode = localMode.value
  settingsStore.accentColor = localAccentColor.value
  settingsStore.fontSize = localFontSize.value
  settingsStore.save()

  // Persist preset selection first (doesn't touch presetCSS)
  if (selectedPresetId.value !== settingsStore.activePresetId) {
    await settingsStore.setActivePreset(selectedPresetId.value)
  }
  // Then apply editor content — overrides whatever setActivePreset did to presetCSS
  settingsStore.presetCSS = localCSS.value
  settingsStore.applyTheme()

  close()
}

function onCancel(): void {
  // Restore from snapshot
  if (snapshot) {
    themeStore.mode = snapshot.mode
    themeStore.apply()
    settingsStore.themeMode = snapshot.mode
    settingsStore.accentColor = snapshot.accentColor
    settingsStore.fontSize = snapshot.fontSize
    settingsStore.presetCSS = snapshot.presetCSS
    settingsStore.activePresetId = snapshot.activePresetId
    settingsStore.applyTheme()
  }
  close()
}

defineExpose({ open, close })
</script>

<style scoped>
.settings-overlay {
  position: fixed; inset: 0; z-index: 200;
  background: rgba(0,0,0,0.2);
  display: flex; align-items: center; justify-content: center;
}

.settings-window {
  width: 720px; height: 520px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-lg);
  display: flex; flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.settings-header h2 {
  margin: 0; font-size: var(--fs-base); font-weight: var(--fw-bold);
}

.settings-body {
  display: flex; flex: 1; overflow: hidden;
}

/* Left tabs */
.settings-tabs {
  width: 160px; flex-shrink: 0;
  border-right: 1px solid var(--border);
  padding: 8px;
  display: flex; flex-direction: column; gap: 2px;
}
.tab-btn {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px; border-radius: var(--r-md);
  border: none; background: transparent;
  font-size: var(--fs-sm); font-weight: var(--fw-medium);
  color: var(--text-secondary); cursor: pointer;
  transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  height: auto; text-align: left;
}
.tab-btn:hover { background: var(--surface-hover); color: var(--text); }
.tab-btn.active { background: var(--surface-active); color: var(--text); font-weight: var(--fw-semibold); }

/* Right content */
.settings-content {
  flex: 1; overflow-y: auto; padding: 16px 20px;
}

.tab-panel {
  display: flex; flex-direction: column; gap: 0;
}

.placeholder-text {
  color: var(--text-muted); font-size: var(--fs-sm);
  text-align: center; padding: 80px 0;
}

/* Sections */
.setting-section {
  margin-bottom: 16px;
}
.section-title {
  display: block;
  font-size: var(--fs-sm); font-weight: var(--fw-semibold);
  margin-bottom: 8px; color: var(--text-heading);
}
.section-divider {
  border: none; border-top: 1px solid var(--border-light);
  margin: 16px 0;
}

.setting-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 6px 0;
}
.setting-row > label {
  font-size: var(--fs-sm); font-weight: var(--fw-medium);
  color: var(--text);
}

.radio-group {
  display: flex; gap: 16px;
}
.radio-label {
  font-size: var(--fs-sm); cursor: pointer;
  display: flex; align-items: center; gap: 4px;
}

.color-input {
  width: 36px; height: 28px; padding: 2px;
  border: 1px solid var(--border); border-radius: var(--r-sm);
  cursor: pointer;
}

.btn-group {
  display: flex; gap: 0;
}
.btn-group button {
  padding: 4px 12px; height: 28px;
  border: 1px solid var(--border); background: var(--surface);
  font-size: var(--fs-xs); cursor: pointer; color: var(--text-secondary);
  transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
}
.btn-group button:first-child { border-radius: var(--r-sm) 0 0 var(--r-sm); }
.btn-group button:last-child { border-radius: 0 var(--r-sm) var(--r-sm) 0; }
.btn-group button.active {
  background: var(--accent); color: var(--accent-fg); border-color: var(--accent);
}

/* Preset row */
.preset-row {
  display: flex; gap: 6px; align-items: center;
}
.preset-select {
  flex: 1; font-size: var(--fs-sm);
}
.sm { font-size: var(--fs-xs); height: 28px; }
.danger { color: var(--danger); }

/* CSS editor */
.css-editor {
  width: 100%; height: 160px;
  font-family: var(--font-mono); font-size: var(--fs-xs);
  line-height: 1.6;
  padding: 8px 10px;
  border: 1px solid var(--border); border-radius: var(--r-md);
  background: var(--bg); color: var(--text);
  resize: vertical; tab-size: 2;
  margin-bottom: 8px;
}
.css-editor:focus { outline: none; border-color: var(--accent); }

/* Footer */
.settings-footer {
  display: flex; justify-content: flex-end; gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
</style>
