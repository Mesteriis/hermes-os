<script setup lang="ts">
import { computed, useAttrs } from 'vue'
import Icon from '../Icon.vue'

defineOptions({
  inheritAttrs: false
})

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

const attrs = useAttrs()
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
    v-bind="attrs"
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
