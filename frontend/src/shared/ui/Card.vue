<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['hermes-card', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.hermes-card {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  overflow: hidden;
}
</style>
