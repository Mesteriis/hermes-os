<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  width?: string
  height?: string
  rounded?: boolean
}>(), {
  width: '100%',
  height: '1rem',
  rounded: false
})

const classes = computed(() => [
  'hermes-skeleton',
  { 'hermes-skeleton--rounded': props.rounded },
  props.class
])
</script>

<template>
  <div :class="classes" :style="{ width, height }" />
</template>

<style scoped>
.hermes-skeleton {
  background: var(--hh-hover-bg);
  border-radius: var(--hh-radius-xs);
  animation: hermes-skeleton-pulse 1.5s ease-in-out infinite;
}

.hermes-skeleton--rounded {
  border-radius: var(--hh-radius-pill);
}

@keyframes hermes-skeleton-pulse {
  0%, 100% {
    opacity: 0.4;
  }
  50% {
    opacity: 0.8;
  }
}
</style>
