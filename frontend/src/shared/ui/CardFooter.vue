<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['hermes-card-footer', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.hermes-card-footer {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--hh-border);
}
</style>
