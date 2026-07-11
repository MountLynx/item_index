<template>
  <component :is="icon" v-if="icon" :size="size" :stroke="stroke" class="ti" />
  <span v-else class="ti-fb">{{ fallback }}</span>
</template>

<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import {
  IconDatabase, IconPlus, IconMoon, IconSun, IconSettings,
  IconFolder, IconFolderOpen, IconTag, IconHash, IconChevronRight,
  IconClipboard, IconTrash, IconFile, IconFileText, IconPhoto,
  IconBook, IconMusic, IconVideo, IconFileZip,
  IconPaperclip, IconArrowLeft, IconCheckbox, IconCircle, IconCheck, IconX,
} from '@tabler/icons-vue'

const map: Record<string, any> = {
  'database': IconDatabase, 'plus': IconPlus, 'moon': IconMoon, 'sun': IconSun,
  'settings': IconSettings, 'folder': IconFolder, 'folder-open': IconFolderOpen,
  'tag': IconTag, 'hash': IconHash, 'chevron-right': IconChevronRight,
  'clipboard': IconClipboard, 'trash': IconTrash, 'file': IconFile,
  'file-text': IconFileText, 'photo': IconPhoto, 'book': IconBook,
  'music': IconMusic, 'video': IconVideo, 'file-zip': IconFileZip,
  'paperclip': IconPaperclip, 'arrow-left': IconArrowLeft,
  'checkbox': IconCheckbox, 'circle': IconCircle, 'check': IconCheck, 'x': IconX,
}

const props = withDefaults(defineProps<{
  name: string; size?: number | string; stroke?: number; fallback?: string;
}>(), { size: 20, stroke: 1.5, fallback: '◆' })

const icon = computed(() => map[props.name] || null)
</script>

<style scoped>
.ti { flex-shrink: 0; display: inline-block; vertical-align: middle; }
.ti-fb { flex-shrink: 0; }
</style>
