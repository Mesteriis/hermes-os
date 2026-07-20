<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'Coming soon',
  description: 'This UI contract is reserved for a future product surface.'
})

const classes = computed(() => ['hermes-state', 'hermes-state--coming-soon', props.class])
</script>

<template>
  <section :class="classes">
    <Icon icon="tabler:sparkles" size="2rem" class="hermes-state-icon" />
    <div class="hermes-state-copy">
      <h3 class="hermes-state-title">{{ title }}</h3>
      <p v-if="description" class="hermes-state-description">{{ description }}</p>
    </div>
  </section>
</template>
