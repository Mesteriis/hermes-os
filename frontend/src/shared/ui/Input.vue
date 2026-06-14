<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  type?: string
  error?: string
  class?: string
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  readonly: false,
  type: 'text'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  focus: [event: FocusEvent]
  blur: [event: FocusEvent]
}>()

const classes = computed(() => [
  'hermes-input',
  { 'hermes-input--error': props.error },
  props.class
])

function handleInput(event: Event): void {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

function handleFocus(event: FocusEvent): void {
  emit('focus', event)
}

function handleBlur(event: FocusEvent): void {
  emit('blur', event)
}
</script>

<template>
  <div class="hermes-input-wrapper">
    <input
      :class="classes"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :type="type"
      @input="handleInput"
      @focus="handleFocus"
      @blur="handleBlur"
    />
    <span v-if="error" class="hermes-input-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.hermes-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.hermes-input {
  width: 100%;
  height: 2.125rem;
  padding: 0 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, rgba(4, 18, 21, 0.8));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  transition: all 150ms ease;
  outline: none;
  box-sizing: border-box;
}

.hermes-input::placeholder {
  color: var(--hh-text-muted);
}

.hermes-input:hover:not(:disabled):not(:read-only) {
  border-color: var(--hh-border-accent);
}

.hermes-input:focus {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 1px var(--hh-focus-ring);
}

.hermes-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hermes-input--error {
  border-color: var(--hh-color-danger);
}

.hermes-input--error:focus {
  box-shadow: 0 0 0 1px var(--hh-color-danger);
}

.hermes-input-error {
  font-size: 0.75rem;
  color: var(--hh-color-danger);
}
</style>
