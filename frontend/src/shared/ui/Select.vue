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

