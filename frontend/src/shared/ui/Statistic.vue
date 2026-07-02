<script setup lang="ts">
import { computed } from 'vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  label: string
  value: string | number
  description?: string
  trend?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'hermes-statistic',
  `hermes-statistic--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <span class="hermes-statistic-label">{{ label }}</span>
    <strong class="hermes-statistic-value">{{ value }}</strong>
    <span v-if="trend" class="hermes-statistic-trend">{{ trend }}</span>
    <p v-if="description" class="hermes-statistic-description">{{ description }}</p>
  </article>
</template>
