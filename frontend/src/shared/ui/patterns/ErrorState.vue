<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'Something went wrong',
  description: 'The owning surface can provide a retry action.'
})

const classes = computed(() => ['hermes-state', 'hermes-state--error', props.class])
</script>

<template>
  <section :class="classes" role="alert">
    <Icon icon="tabler:alert-circle" size="2rem" class="hermes-state-icon" />
    <div class="hermes-state-copy">
      <h3 class="hermes-state-title">{{ title }}</h3>
      <p v-if="description" class="hermes-state-description">{{ description }}</p>
    </div>
    <div v-if="$slots.action" class="hermes-state-action">
      <slot name="action" />
    </div>
  </section>
</template>
