<script setup lang="ts">
import { ScrollAreaRoot, ScrollAreaViewport, ScrollAreaScrollbar, ScrollAreaThumb } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  maxHeight?: string
}>(), {})

const classes = computed(() => ['hermes-scroll-area', props.class])
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

