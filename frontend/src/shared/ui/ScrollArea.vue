<script setup lang="ts">
import { ScrollAreaRoot, ScrollAreaViewport, ScrollAreaScrollbar, ScrollAreaThumb } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  size?: 'sm' | 'md' | 'lg' | 'full'
  maxHeight?: string
}>(), {
  size: 'full'
})

const classes = computed(() => [
  'hermes-scroll-area',
  `hermes-scroll-area--${props.size}`,
  { 'hermes-scroll-area--bounded': props.maxHeight },
  props.class
])
</script>

<template>
  <ScrollAreaRoot :class="classes">
    <ScrollAreaViewport class="hermes-scroll-viewport">
      <slot />
    </ScrollAreaViewport>
    <ScrollAreaScrollbar class="hermes-scrollbar" orientation="vertical">
      <ScrollAreaThumb class="hermes-scroll-thumb" />
    </ScrollAreaScrollbar>
    <ScrollAreaScrollbar class="hermes-scrollbar" orientation="horizontal">
      <ScrollAreaThumb class="hermes-scroll-thumb" />
    </ScrollAreaScrollbar>
  </ScrollAreaRoot>
</template>
