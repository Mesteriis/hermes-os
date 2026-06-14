<script setup lang="ts">
import { DropdownMenuItem } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  icon?: string
  disabled?: boolean
  class?: string
  inset?: boolean
}>(), {
  disabled: false,
  inset: false
})

const classes = computed(() => [
  'hermes-dropdown-item',
  { 'hermes-dropdown-item--inset': props.inset, 'hermes-dropdown-item--disabled': props.disabled },
  props.class
])
</script>

<template>
  <DropdownMenuItem :class="classes" :disabled="disabled">
    <Icon v-if="icon" :icon="icon" size="1rem" class="hermes-dropdown-item-icon" />
    <slot />
  </DropdownMenuItem>
</template>

<style scoped>
.hermes-dropdown-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  color: var(--hh-text-secondary);
  border-radius: var(--hh-radius-xs);
  cursor: pointer;
  outline: none;
  user-select: none;
  transition: background 100ms ease;
  border: none;
  background: transparent;
  text-align: left;
  font-family: var(--hh-font-sans);
}

.hermes-dropdown-item:hover,
.hermes-dropdown-item[data-highlighted] {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.hermes-dropdown-item--inset {
  padding-left: 2.25rem;
}

.hermes-dropdown-item--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hermes-dropdown-item-icon {
  flex-shrink: 0;
  color: var(--hh-text-muted);
}
</style>
