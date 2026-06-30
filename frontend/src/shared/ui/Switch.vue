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

