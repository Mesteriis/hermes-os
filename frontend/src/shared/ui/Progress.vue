<script setup lang="ts">
import { ProgressRoot, ProgressIndicator } from 'reka-ui'
import { computed, ref, watchEffect } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: number
  max?: number
  indeterminate?: boolean
  size?: 'sm' | 'md' | 'lg'
  class?: string
}>(), {
  modelValue: 0,
  max: 100,
  indeterminate: false,
  size: 'md'
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const percentage = computed(() => {
  if (props.max <= 0) return 0
  return Math.round((props.modelValue / props.max) * 100)
})

const rootClasses = computed(() => [
  'hermes-progress-root',
  `hermes-progress--${props.size}`,
  props.class,
  { 'hermes-progress--indeterminate': props.indeterminate }
])

const indicatorRef = ref<InstanceType<typeof ProgressIndicator> | null>(null)

watchEffect(() => {
  const element = indicatorRef.value?.$el as HTMLElement | undefined
  if (!element || props.indeterminate) return
  element.style.transform = `translateX(-${100 - percentage.value}%)`
})
</script>

<template>
  <ProgressRoot
    :model-value="modelValue"
    :max="max"
    :class="rootClasses"
    @update:model-value="(val: any) => emit('update:modelValue', Number(val))"
  >
    <ProgressIndicator ref="indicatorRef" class="hermes-progress-indicator" />
  </ProgressRoot>
</template>

<style scoped>
.hermes-progress-root {
  position: relative;
  overflow: hidden;
  background: var(--hh-hover-bg);
  border-radius: 9999px;
  width: 100%;
}

.hermes-progress--sm {
  height: 0.25rem;
}

.hermes-progress--md {
  height: 0.5rem;
}

.hermes-progress--lg {
  height: 0.75rem;
}

.hermes-progress-indicator {
  width: 100%;
  height: 100%;
  border-radius: inherit;
  background: var(--hh-accent);
  transition: transform 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

.hermes-progress--indeterminate .hermes-progress-indicator {
  animation: progress-indeterminate 1.5s ease-in-out infinite;
  width: 40%;
  background: linear-gradient(90deg, transparent, var(--hh-accent), transparent);
}

@keyframes progress-indeterminate {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(350%);
  }
}
</style>
