<script setup lang="ts">
import { TabsContent } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  value: string
  class?: string
  forceMount?: boolean
}>(), {
  forceMount: false
})

const classes = computed(() => ['hermes-tab-content', props.class])
</script>

<template>
  <TabsContent
    :class="classes"
    :value="value"
    :force-mount="forceMount"
  >
    <slot />
  </TabsContent>
</template>

<style scoped>
.hermes-tab-content {
  outline: none;
}

.hermes-tab-content:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
  border-radius: var(--hh-radius-sm);
}
</style>
