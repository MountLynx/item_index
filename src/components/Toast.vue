<template>
  <Teleport to="body">
    <div class="stack">
      <TransitionGroup name="t">
        <div v-for="t in items" :key="t.id" class="toast" :class="t.type">
          <span>{{ t.message }}</span>
          <button class="x" @click="remove(t.id)"><TablerIcon name="x" :size="15" /></button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToast } from '@/composables/toast'
import TablerIcon from './TablerIcon.vue'

const { items, remove } = useToast()
</script>

<style scoped>
.stack { position: fixed; bottom: 16px; right: 16px; display: flex; flex-direction: column; gap: 8px; z-index: 300; pointer-events: none; }
.toast {
  display: flex; align-items: center; gap: 8px; padding: 8px 16px; border-radius: var(--r-lg);
  font-size: var(--fs-sm); font-weight: var(--fw-medium); box-shadow: var(--shadow-md); pointer-events: auto;
}
.toast.success { background: var(--success); color: #fff; }
.toast.error { background: var(--danger); color: #fff; }
.toast.info { background: var(--surface); color: var(--text); border: 1px solid var(--border); }
.x { background: none; border: none; color: inherit; cursor: pointer; padding: 0; opacity: 0.6; }
.x:hover { opacity: 1; }
.t-enter-active { transition: all var(--slow) var(--ease); }
.t-leave-active { transition: all var(--fast) ease-in; }
.t-enter-from { opacity: 0; transform: translateY(12px); }
.t-leave-to { opacity: 0; transform: translateY(-8px); }
</style>
