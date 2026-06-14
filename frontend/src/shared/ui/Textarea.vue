<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  rows?: number
  error?: string
  class?: string
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  rows: 3
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const classes = computed(() => [
  'hermes-textarea',
  { 'hermes-textarea--error': props.error },
  props.class
])

function handleInput(event: Event): void {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <div class="hermes-textarea-wrapper">
    <textarea
      :class="classes"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :rows="rows"
      @input="handleInput"
    />
    <span v-if="error" class="hermes-textarea-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.hermes-textarea-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.hermes-textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, rgba(4, 18, 21, 0.8));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  transition: all 150ms ease;
  outline: none;
  resize: vertical;
  box-sizing: border-box;
  line-height: 1.5;
}

.hermes-textarea::placeholder {
  color: var(--hh-text-muted);
}

.hermes-textarea:hover:not(:disabled) {
  border-color: var(--hh-border-accent);
}

.hermes-textarea:focus {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 1px var(--hh-focus-ring);
}

.hermes-textarea:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hermes-textarea--error {
  border-color: var(--hh-color-danger);
}

.hermes-textarea-error {
  font-size: 0.75rem;
  color: var(--hh-color-danger);
}
</style>
