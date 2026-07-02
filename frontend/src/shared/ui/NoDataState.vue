<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'No data',
  description: 'There is no local data for this surface yet.'
})

const classes = computed(() => ['hermes-state', 'hermes-state--no-data', props.class])
</script>

<template>
  <section :class="classes">
    <Icon icon="tabler:database-off" size="2rem" class="hermes-state-icon" />
    <div class="hermes-state-copy">
      <h3 class="hermes-state-title">{{ title }}</h3>
      <p v-if="description" class="hermes-state-description">{{ description }}</p>
    </div>
  </section>
</template>
