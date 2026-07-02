<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  title: string
  description?: string
  meta?: string
  icon?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral',
  icon: 'tabler:activity'
})

const classes = computed(() => [
  'hermes-activity-item',
  `hermes-activity-item--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <span class="hermes-activity-icon" aria-hidden="true">
      <Icon :icon="icon" size="1rem" />
    </span>
    <div class="hermes-activity-copy">
      <strong class="hermes-activity-title">{{ title }}</strong>
      <span v-if="description" class="hermes-activity-description">{{ description }}</span>
    </div>
    <span v-if="meta" class="hermes-activity-meta">{{ meta }}</span>
  </article>
</template>
