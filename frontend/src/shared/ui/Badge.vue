<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'accent' | 'success' | 'warning' | 'danger' | 'info' | 'neutral'
  size?: 'sm' | 'md'
  class?: string
}>(), {
  variant: 'default',
  size: 'sm'
})

const classes = computed(() => [
  'hermes-badge',
  `hermes-badge--${props.variant}`,
  `hermes-badge--${props.size}`,
  props.class
])
</script>

<template>
  <span :class="classes">
    <slot />
  </span>
</template>

<style scoped>
.hermes-badge {
  display: inline-flex;
  align-items: center;
  font-family: var(--hh-font-sans);
  font-weight: 500;
  white-space: nowrap;
  border-radius: var(--hh-radius-pill);
  line-height: 1;
}

.hermes-badge--sm {
  height: 1.25rem;
  padding: 0 0.5rem;
  font-size: 0.6875rem;
}

.hermes-badge--md {
  height: 1.5rem;
  padding: 0 0.625rem;
  font-size: 0.75rem;
}

/* Variants */
.hermes-badge--default {
  background: var(--hh-hover-bg);
  color: var(--hh-text-secondary);
}

.hermes-badge--accent {
  background: var(--hh-accent-tint);
  color: var(--hh-accent);
}

.hermes-badge--success {
  background: var(--hh-status-success-surface);
  color: var(--hh-status-success-text);
}

.hermes-badge--warning {
  background: var(--hh-status-warning-surface);
  color: var(--hh-status-warning-text);
}

.hermes-badge--danger {
  background: var(--hh-status-danger-surface);
  color: var(--hh-status-danger-text);
}

.hermes-badge--info {
  background: var(--hh-status-info-surface);
  color: var(--hh-status-info-text);
}

.hermes-badge--neutral {
  background: var(--hh-status-neutral-surface);
  color: var(--hh-status-neutral-text);
}
</style>
