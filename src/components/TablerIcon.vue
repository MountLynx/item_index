<template>
  <i v-if="!isEmoji" :class="['ti', 'ti-' + props.name]" :style="{ fontSize: sizePx, '--stroke': strokeWidth }" />
  <span v-else class="ti-emoji">{{ props.name }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import '@tabler/icons-webfont/dist/tabler-icons.css'

const props = withDefaults(defineProps<{
  name: string; size?: number | string; stroke?: number;
}>(), { size: 20, stroke: 1.5 })

const sizePx = computed(() => typeof props.size === 'number' ? props.size + 'px' : props.size)
const strokeWidth = computed(() => String(props.stroke))
const isEmoji = computed(() => /^(\p{Emoji}|\p{Emoji_Presentation}|\p{Emoji_Modifier_Base}|\p{Emoji_Component})+$/u.test(props.name))
</script>

<style scoped>
.ti-icn {
  display: inline-block;
  vertical-align: middle;
  flex-shrink: 0;
  -webkit-text-stroke: calc(var(--stroke, 1.5) * 0.5px) currentColor;
  paint-order: stroke fill;
}
.ti-emoji {
  flex-shrink: 0;
  font-size: 1.1em;
  line-height: 1;
}
</style>
