<script setup lang="ts">
import { computed } from 'vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  label: string
  value: string | number
  unit?: string
  delta?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'hermes-metric',
  `hermes-metric--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <span class="hermes-metric-label">{{ label }}</span>
    <strong class="hermes-metric-value">
      {{ value }}<span v-if="unit" class="hermes-metric-unit">{{ unit }}</span>
    </strong>
    <span v-if="delta" class="hermes-metric-delta">{{ delta }}</span>
  </article>
</template>
