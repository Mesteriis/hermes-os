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

<style scoped>
.hermes-scroll-area {
  overflow: hidden;
  position: relative;
}

.hermes-scroll-viewport {
  width: 100%;
  height: 100%;
}

.hermes-scrollbar {
  display: flex;
  user-select: none;
  touch-action: none;
  transition: background 160ms ease;
  background: transparent;
}

.hermes-scrollbar[data-orientation="vertical"] {
  width: 0.5rem;
  padding: 0.125rem 0;
}

.hermes-scrollbar[data-orientation="horizontal"] {
  height: 0.5rem;
  padding: 0 0.125rem;
  flex-direction: column;
}

.hermes-scrollbar:hover {
  background: var(--hh-hover-bg);
}

.hermes-scroll-thumb {
  flex: 1;
  background: var(--hh-border);
  border-radius: var(--hh-radius-pill);
  position: relative;
}

.hermes-scroll-thumb::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 100%;
  height: 100%;
  min-width: 2.5rem;
  min-height: 2.5rem;
}
</style>
