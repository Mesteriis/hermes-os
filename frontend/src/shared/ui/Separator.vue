<script setup lang="ts">
import { Separator as RekaSeparator } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  orientation?: 'horizontal' | 'vertical'
  decorative?: boolean
  class?: string
}>(), {
  orientation: 'horizontal',
  decorative: true
})

const classes = computed(() => [
  'hermes-separator',
  `hermes-separator--${props.orientation}`,
  props.class
])
</script>

<template>
  <RekaSeparator
    :class="classes"
    :orientation="orientation"
    :decorative="decorative"
  />
</template>

<style scoped>
.hermes-separator {
  flex-shrink: 0;
  background: var(--hh-border);
}

.hermes-separator--horizontal {
  height: 1px;
  width: 100%;
}

.hermes-separator--vertical {
  width: 1px;
  height: 100%;
}
</style>
