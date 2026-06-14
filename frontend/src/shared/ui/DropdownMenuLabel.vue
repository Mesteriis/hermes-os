<script setup lang="ts">
import { DropdownMenuLabel } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['hermes-dropdown-label', props.class])
</script>

<template>
  <DropdownMenuLabel :class="classes">
    <slot />
  </DropdownMenuLabel>
</template>

<style scoped>
.hermes-dropdown-label {
  padding: 0.5rem 0.75rem 0.25rem;
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--hh-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
</style>
