<script setup lang="ts">
import { computed } from 'vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  value: number
  label?: string
  max?: number
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'hermes-counter',
  `hermes-counter--${props.tone}`,
  props.class
])

const displayValue = computed(() => props.max === undefined ? String(props.value) : `${props.value}/${props.max}`)
</script>

<template>
  <span :class="classes" :aria-label="label">
    <strong class="hermes-counter-value">{{ displayValue }}</strong>
    <span v-if="label" class="hermes-counter-label">{{ label }}</span>
  </span>
</template>
