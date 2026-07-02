<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  title: string
  description?: string
  time?: string
  icon?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'hermes-timeline-item',
  `hermes-timeline-item--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <div class="hermes-timeline-marker" aria-hidden="true">
      <Icon v-if="icon" :icon="icon" size="0.875rem" />
    </div>
    <div class="hermes-timeline-copy">
      <div class="hermes-timeline-heading">
        <strong class="hermes-timeline-title">{{ title }}</strong>
        <time v-if="time" class="hermes-timeline-time">{{ time }}</time>
      </div>
      <p v-if="description" class="hermes-timeline-description">{{ description }}</p>
      <slot />
    </div>
  </article>
</template>
