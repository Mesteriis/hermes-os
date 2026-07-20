<!-- Shared status primitive. -->
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'accent' | 'success' | 'warning' | 'danger' | 'info' | 'neutral'
  size?: 'sm' | 'md'
  class?: string
}>(), {
  variant: 'default',
  size: 'sm'
})

const classes = computed(() => [
  'hermes-badge',
  `hermes-badge--${props.variant}`,
  `hermes-badge--${props.size}`,
  props.class
])
</script>

<template>
  <span :class="classes">
    <slot />
  </span>
</template>
