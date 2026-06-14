<script setup lang="ts">
import { TabsTrigger } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  value: string
  disabled?: boolean
  class?: string
}>(), {
  disabled: false
})

const classes = computed(() => ['hermes-tab-trigger', { 'hermes-tab-trigger--disabled': props.disabled }, props.class])
</script>

<template>
  <TabsTrigger
    :class="classes"
    :value="value"
    :disabled="disabled"
  >
    <slot />
  </TabsTrigger>
</template>

<style scoped>
.hermes-tab-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  height: 1.75rem;
  padding: 0 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-muted);
  background: transparent;
  border: none;
  border-radius: calc(var(--hh-radius-sm) - 2px);
  cursor: pointer;
  transition: all 150ms ease;
  white-space: nowrap;
  outline: none;
}

.hermes-tab-trigger:hover {
  color: var(--hh-text-secondary);
}

.hermes-tab-trigger[data-state="active"] {
  color: var(--hh-accent);
  background: var(--hh-surface-panel);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.hermes-tab-trigger:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.hermes-tab-trigger--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
