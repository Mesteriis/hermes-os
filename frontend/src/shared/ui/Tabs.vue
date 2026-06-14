<script setup lang="ts">
import { TabsRoot, TabsList, TabsTrigger, TabsContent } from 'reka-ui'
import { computed } from 'vue'
import type { TabsRootEmits, TabsRootProps } from 'reka-ui'

// Re-export with Hermes styling
const props = withDefaults(defineProps<{
  modelValue?: string
  defaultValue?: string
  orientation?: 'horizontal' | 'vertical'
  class?: string
  listClass?: string
  contentClass?: string
}>(), {
  orientation: 'horizontal'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const rootClasses = computed(() => ['hermes-tabs', props.class])
const listClasses = computed(() => ['hermes-tabs-list', `hermes-tabs-list--${props.orientation}`, props.listClass])
</script>

<template>
  <TabsRoot
    :class="rootClasses"
    :model-value="modelValue"
    :default-value="defaultValue"
    :orientation="orientation"
    @update:model-value="(val) => emit('update:modelValue', val)"
  >
    <TabsList :class="listClasses">
      <slot name="list" />
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
</style>
