<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'Offline',
  description: 'Local context remains available; remote refresh is paused.'
})

const classes = computed(() => ['hermes-state', 'hermes-state--offline', props.class])
</script>

<template>
  <section :class="classes" role="status">
    <Icon icon="tabler:wifi-off" size="2rem" class="hermes-state-icon" />
    <div class="hermes-state-copy">
      <h3 class="hermes-state-title">{{ title }}</h3>
      <p v-if="description" class="hermes-state-description">{{ description }}</p>
    </div>
  </section>
</template>
