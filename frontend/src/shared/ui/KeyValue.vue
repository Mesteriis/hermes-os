<script setup lang="ts">
import { computed } from 'vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  label: string
  value: string | number
  description?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'hermes-key-value',
  `hermes-key-value--${props.tone}`,
  props.class
])
</script>

<template>
  <div :class="classes">
    <dt class="hermes-key-value-label">{{ label }}</dt>
    <dd class="hermes-key-value-value">{{ value }}</dd>
    <dd v-if="description" class="hermes-key-value-description">{{ description }}</dd>
  </div>
</template>
