<script setup lang="ts">
import { TabsRoot, TabsList, TabsTrigger } from 'reka-ui'
import { computed } from 'vue'

type HermesTab = {
  id: string
  label: string
}

// Re-export with Hermes styling
const props = withDefaults(defineProps<{
  modelValue?: string
  active?: string
  defaultValue?: string
  orientation?: 'horizontal' | 'vertical'
  class?: string
  listClass?: string
  contentClass?: string
  tabs?: HermesTab[]
}>(), {
  orientation: 'horizontal'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  select: [value: string]
}>()

const tabs = computed(() => props.tabs ?? [])
const selectedValue = computed(() => props.modelValue ?? props.active)
const rootClasses = computed(() => ['hermes-tabs', props.class])
const listClasses = computed(() => ['hermes-tabs-list', `hermes-tabs-list--${props.orientation}`, props.listClass])

function handleUpdateModelValue(value: string | number) {
  const nextValue = String(value)
  emit('update:modelValue', nextValue)
  emit('select', nextValue)
}
</script>

<template>
  <TabsRoot
    :class="rootClasses"
    :model-value="selectedValue"
    :default-value="defaultValue"
    :orientation="orientation"
    @update:model-value="handleUpdateModelValue"
  >
    <TabsList :class="listClasses">
      <slot name="list">
        <TabsTrigger
          v-for="tab in tabs"
          :key="tab.id"
          :value="tab.id"
          class="hermes-tabs-trigger"
        >
          {{ tab.label }}
        </TabsTrigger>
      </slot>
    </TabsList>
    <slot />
  </TabsRoot>
</template>

<style scoped>
.hermes-tabs {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.hermes-tabs-list {
  display: flex;
  gap: 0.125rem;
  background: var(--hh-hover-bg);
  border-radius: var(--hh-radius-sm);
  padding: 0.1875rem;
}

.hermes-tabs-list--vertical {
  flex-direction: column;
}

.hermes-tabs-trigger {
  border: none;
  border-radius: var(--hh-radius-xs, 0.25rem);
  background: transparent;
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  font: inherit;
  font-size: 0.75rem;
  padding: 0.375rem 0.625rem;
}

.hermes-tabs-trigger:hover {
  background: var(--hh-bg-hover, #f3f4f6);
  color: var(--hh-text-primary, #1f2937);
}

.hermes-tabs-trigger[data-state='active'] {
  background: var(--hh-bg-primary, #ffffff);
  color: var(--hh-accent, #3b82f6);
  font-weight: 600;
}
</style>
