<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'destructive'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
  icon?: string
  class?: string
  type?: 'button' | 'submit' | 'reset'
}>(), {
  variant: 'default',
  size: 'md',
  disabled: false,
  loading: false,
  type: 'button'
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const classes = computed(() => {
  return [
    'hermes-btn',
    `hermes-btn--${props.variant}`,
    `hermes-btn--${props.size}`,
    { 'hermes-btn--disabled': props.disabled || props.loading },
    props.class
  ]
})

function handleClick(event: MouseEvent): void {
  if (!props.disabled && !props.loading) {
    emit('click', event)
  }
}
</script>

<template>
  <button
    :class="classes"
    :disabled="disabled || loading"
    :type="type"
    @click="handleClick"
  >
    <Icon v-if="loading" icon="tabler:loader-2" size="1em" class="hermes-btn-spinner" />
    <Icon v-else-if="icon" :icon="icon" size="1em" />
    <span v-if="$slots.default" class="hermes-btn-text">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.hermes-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  font-family: var(--hh-font-sans);
  font-weight: 500;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-sm);
  cursor: pointer;
  transition: all 150ms ease;
  white-space: nowrap;
  user-select: none;
  line-height: 1;
}

.hermes-btn:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.hermes-btn--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

/* Variants */
.hermes-btn--default {
  background: var(--hh-accent);
  color: var(--hh-accent-contrast);
  border-color: var(--hh-accent);
}
.hermes-btn--default:hover:not(:disabled) {
  filter: brightness(1.1);
}
.hermes-btn--default:active:not(:disabled) {
  filter: brightness(0.9);
}

.hermes-btn--secondary {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
  border-color: var(--hh-border);
}
.hermes-btn--secondary:hover:not(:disabled) {
  background: var(--hh-active-bg);
  border-color: var(--hh-border-accent);
}
.hermes-btn--secondary:active:not(:disabled) {
  background: var(--hh-accent-tint);
}

.hermes-btn--outline {
  background: transparent;
  color: var(--hh-text-primary);
  border-color: var(--hh-border);
}
.hermes-btn--outline:hover:not(:disabled) {
  background: var(--hh-hover-bg);
  border-color: var(--hh-border-accent);
}
.hermes-btn--outline:active:not(:disabled) {
  background: var(--hh-accent-tint);
}

.hermes-btn--ghost {
  background: transparent;
  color: var(--hh-text-secondary);
  border-color: transparent;
}
.hermes-btn--ghost:hover:not(:disabled) {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}
.hermes-btn--ghost:active:not(:disabled) {
  background: var(--hh-active-bg);
}

.hermes-btn--destructive {
  background: var(--hh-danger-tint);
  color: var(--hh-color-danger);
  border-color: transparent;
}
.hermes-btn--destructive:hover:not(:disabled) {
  background: var(--hh-color-danger-strong);
  color: white;
}
.hermes-btn--destructive:active:not(:disabled) {
  filter: brightness(0.9);
}

/* Sizes */
.hermes-btn--sm {
  height: 1.75rem;
  padding: 0 0.625rem;
  font-size: 0.75rem;
  border-radius: var(--hh-radius-xs);
}

.hermes-btn--md {
  height: 2.125rem;
  padding: 0 0.875rem;
  font-size: 0.8125rem;
}

.hermes-btn--lg {
  height: 2.5rem;
  padding: 0 1.125rem;
  font-size: 0.875rem;
}

.hermes-btn-spinner {
  animation: hermes-spin 1s linear infinite;
}

@keyframes hermes-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
