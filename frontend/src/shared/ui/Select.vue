<script setup lang="ts">
import { SelectRoot, SelectTrigger, SelectValue, SelectContent, SelectItem, SelectItemIndicator, SelectViewport, SelectPortal } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  error?: string
  class?: string
  options?: Array<{ value: string; label: string }>
}>(), {
  modelValue: '',
  placeholder: 'Select…',
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const triggerClasses = computed(() => [
  'hermes-select-trigger',
  { 'hermes-select--error': props.error },
  props.class
])
</script>

<template>
  <div class="hermes-select-wrapper">
    <SelectRoot
      :model-value="modelValue || undefined"
      :disabled="disabled"
      @update:model-value="(val) => emit('update:modelValue', val || '')"
    >
      <SelectTrigger :class="triggerClasses">
        <SelectValue :placeholder="placeholder" class="hermes-select-value" />
        <Icon icon="tabler:chevron-down" size="1rem" class="hermes-select-chevron" />
      </SelectTrigger>
      <SelectPortal>
        <SelectContent class="hermes-select-content" :side-offset="4">
          <SelectViewport class="hermes-select-viewport">
            <SelectItem
              v-for="opt in options"
              :key="opt.value"
              :value="opt.value"
              class="hermes-select-item"
            >
              <SelectItemIndicator>
                <Icon icon="tabler:check" size="0.875rem" class="hermes-select-check" />
              </SelectItemIndicator>
              <span>{{ opt.label }}</span>
            </SelectItem>
          </SelectViewport>
        </SelectContent>
      </SelectPortal>
    </SelectRoot>
    <span v-if="error" class="hermes-select-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.hermes-select-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.hermes-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  width: 100%;
  height: 2.125rem;
  padding: 0 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, rgba(4, 18, 21, 0.8));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  cursor: pointer;
  transition: all 150ms ease;
  outline: none;
  text-align: left;
  box-sizing: border-box;
}

.hermes-select-trigger:hover {
  border-color: var(--hh-border-accent);
}

.hermes-select-trigger:focus-visible {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 1px var(--hh-focus-ring);
}

.hermes-select-trigger[data-placeholder] .hermes-select-value {
  color: var(--hh-text-muted);
}

.hermes-select--error {
  border-color: var(--hh-color-danger);
}

.hermes-select-chevron {
  color: var(--hh-text-muted);
  flex-shrink: 0;
  transition: transform 200ms ease;
}

.hermes-select-trigger[data-state="open"] .hermes-select-chevron {
  transform: rotate(180deg);
}

.hermes-select-content {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  min-width: var(--reka-select-trigger-width);
  overflow: hidden;
}

.hermes-select-viewport {
  padding: 0.25rem;
}

.hermes-select-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  color: var(--hh-text-secondary);
  border-radius: var(--hh-radius-xs);
  cursor: pointer;
  outline: none;
  user-select: none;
  transition: background 100ms ease;
}

.hermes-select-item[data-highlighted] {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.hermes-select-item[data-state="checked"] {
  color: var(--hh-accent);
}

.hermes-select-check {
  color: var(--hh-accent);
  flex-shrink: 0;
}

.hermes-select-error {
  font-size: 0.75rem;
  color: var(--hh-color-danger);
}
</style>
