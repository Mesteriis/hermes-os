<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: boolean
  disabled?: boolean
  class?: string
}>(), {
  modelValue: false,
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const rootClasses = computed(() => ['hermes-switch', { 'hermes-switch--disabled': props.disabled }, props.class])
</script>

<template>
  <SwitchRoot
    :class="rootClasses"
    :checked="modelValue"
    :disabled="disabled"
    @update:checked="(val: boolean) => emit('update:modelValue', val)"
  >
    <SwitchThumb class="hermes-switch-thumb" />
  </SwitchRoot>
</template>

<style scoped>
.hermes-switch {
  position: relative;
  width: 2rem;
  height: 1.125rem;
  border-radius: var(--hh-radius-pill);
  background: var(--hh-border);
  border: none;
  cursor: pointer;
  transition: background 200ms ease;
  flex-shrink: 0;
}

.hermes-switch[data-state="checked"] {
  background: var(--hh-accent);
}

.hermes-switch--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hermes-switch:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.hermes-switch-thumb {
  display: block;
  width: 0.875rem;
  height: 0.875rem;
  border-radius: 50%;
  background: white;
  transition: transform 200ms ease;
  transform: translateX(0.125rem);
  will-change: transform;
}

.hermes-switch[data-state="checked"] .hermes-switch-thumb {
  transform: translateX(1rem);
}
</style>
