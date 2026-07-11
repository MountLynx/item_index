<template>
  <component :is="icon" v-if="icon" :size="size" :stroke="stroke" class="ti" />
  <span v-else class="ti-emoji">{{ isEmoji ? props.name : fallback }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  IconDatabase, IconPlus, IconMoon, IconSun, IconSettings,
  IconFolder, IconFolderOpen, IconTag, IconHash, IconChevronRight,
  IconClipboard, IconTrash, IconFile, IconFileText, IconPhoto,
  IconBook, IconMusic, IconVideo, IconFileZip,
  IconPaperclip, IconArrowLeft, IconCheckbox, IconCircle, IconCheck, IconX,
  IconGripVertical, IconCategory, IconFileDescription, IconDots, IconPencil,
  IconFileCode, IconFileSpreadsheet,
  IconCamera, IconMovie, IconMicrophone,
  IconCode, IconTerminal2, IconServer, IconCpu, IconCloud, IconWifi,
  IconUser, IconUsers, IconHeart, IconStar, IconBookmark, IconPin, IconFlag,
  IconCalendar, IconClock, IconMail, IconPhone, IconMap, IconChartBar, IconChartPie,
  IconDownload, IconUpload, IconShare, IconLink, IconLock, IconEye, IconSend,
  IconHome, IconBuilding, IconBriefcase, IconAward, IconFlame, IconBolt, IconRocket,
  IconPackage, IconTruck, IconShoppingCart,
  IconSearch, IconFilter, IconList, IconLayoutGrid, IconMenu2,
  IconWorld, IconLanguage, IconPalette, IconColorSwatch,
  IconBrain, IconBulb, IconPuzzle, IconTools,
  IconSchool, IconDeviceFloppy, IconCoffee,
  IconAlertTriangle, IconInfoCircle, IconBell, IconMessageCircle,
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
  'grip-vertical': IconGripVertical,
  'category': IconCategory, 'file-description': IconFileDescription,
  'dots': IconDots, 'pencil': IconPencil,
  'file-code': IconFileCode, 'file-spreadsheet': IconFileSpreadsheet,
  'camera': IconCamera, 'movie': IconMovie, 'microphone': IconMicrophone,
  'code': IconCode, 'terminal': IconTerminal2, 'server': IconServer, 'cpu': IconCpu,
  'cloud': IconCloud, 'wifi': IconWifi,
  'user': IconUser, 'users': IconUsers, 'heart': IconHeart, 'star': IconStar,
  'bookmark': IconBookmark, 'pin': IconPin, 'flag': IconFlag,
  'calendar': IconCalendar, 'clock': IconClock, 'mail': IconMail, 'phone': IconPhone,
  'map': IconMap, 'chart-bar': IconChartBar, 'chart-pie': IconChartPie,
  'download': IconDownload, 'upload': IconUpload, 'share': IconShare, 'link': IconLink,
  'lock': IconLock, 'eye': IconEye, 'send': IconSend,
  'home': IconHome, 'building': IconBuilding, 'briefcase': IconBriefcase,
  'award': IconAward, 'flame': IconFlame, 'bolt': IconBolt, 'rocket': IconRocket,
  'package': IconPackage, 'truck': IconTruck, 'shopping-cart': IconShoppingCart,
  'search': IconSearch, 'filter': IconFilter, 'list': IconList, 'layout-grid': IconLayoutGrid,
  'menu': IconMenu2,
  'world': IconWorld, 'language': IconLanguage, 'palette': IconPalette, 'color-swatch': IconColorSwatch,
  'brain': IconBrain, 'bulb': IconBulb, 'puzzle': IconPuzzle, 'tools': IconTools,
  'school': IconSchool, 'device-floppy': IconDeviceFloppy, 'coffee': IconCoffee,
  'alert-triangle': IconAlertTriangle, 'info-circle': IconInfoCircle,
  'bell': IconBell, 'message-circle': IconMessageCircle,
}

export const ICON_NAMES = Object.keys(map)

const props = withDefaults(defineProps<{
  name: string; size?: number | string; stroke?: number; fallback?: string;
}>(), { size: 20, stroke: 1.5, fallback: '◆' })

const icon = computed(() => map[props.name] || null)
const isEmoji = computed(() => /^(\p{Emoji}|\p{Emoji_Presentation}|\p{Emoji_Modifier_Base}|\p{Emoji_Component})+$/u.test(props.name))
</script>

<style scoped>
.ti { flex-shrink: 0; display: inline-block; vertical-align: middle; }
.ti-emoji { flex-shrink: 0; font-size: 1.1em; line-height: 1; }
</style>
