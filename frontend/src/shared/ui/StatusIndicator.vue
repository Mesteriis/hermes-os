<script setup lang="ts">
import { computed } from 'vue'

type StatusTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
  tone?: StatusTone
  label?: string
  size?: 'sm' | 'md' | 'lg'
  pulse?: boolean
  class?: string
}>(), {
  tone: 'neutral',
  size: 'md',
  pulse: false
})

const classes = computed(() => [
  'hermes-status-indicator',
  `hermes-status-indicator--${props.tone}`,
  `hermes-status-indicator--${props.size}`,
  { 'hermes-status-indicator--pulse': props.pulse },
  props.class
])
</script>

<template>
  <span :class="classes">
    <span class="hermes-status-indicator-dot" aria-hidden="true" />
    <span v-if="label" class="hermes-status-indicator-label">{{ label }}</span>
  </span>
</template>
