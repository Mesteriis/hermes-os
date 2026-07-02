<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  query?: string
  class?: string
}>(), {
  title: 'No results'
})

const classes = computed(() => ['hermes-state', 'hermes-state--no-results', props.class])
</script>

<template>
  <section :class="classes" role="status" aria-live="polite">
    <Icon icon="tabler:search-off" size="2rem" class="hermes-state-icon" />
    <div class="hermes-state-copy">
      <h3 class="hermes-state-title">{{ title }}</h3>
      <p v-if="description" class="hermes-state-description">{{ description }}</p>
      <p v-if="query" class="hermes-state-query">{{ query }}</p>
    </div>
  </section>
</template>
