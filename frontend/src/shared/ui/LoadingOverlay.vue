<script setup lang="ts">
import { computed } from 'vue'
import Spinner from './Spinner.vue'

const props = withDefaults(defineProps<{
  visible?: boolean
  label?: string
  description?: string
  class?: string
}>(), {
  visible: true,
  label: 'Loading'
})

const classes = computed(() => [
  'hermes-loading-overlay',
  props.class
])
</script>

<template>
  <div v-if="visible" :class="classes" role="status" aria-live="polite">
    <Spinner decorative size="lg" />
    <div class="hermes-loading-overlay-copy">
      <strong class="hermes-loading-overlay-label">{{ label }}</strong>
      <p v-if="description" class="hermes-loading-overlay-description">{{ description }}</p>
      <slot />
    </div>
  </div>
</template>
