<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['hermes-card-header', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.hermes-card-header {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--hh-border);
}
</style>
