<script setup lang="ts">
import { TooltipRoot, TooltipTrigger, TooltipPortal, TooltipContent, TooltipArrow } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  content?: string
  side?: 'top' | 'bottom' | 'left' | 'right'
  sideOffset?: number
  delayDuration?: number
  class?: string
}>(), {
  side: 'top',
  sideOffset: 4,
  delayDuration: 400
})

const contentClasses = computed(() => ['hermes-tooltip-content', props.class])
</script>

<template>
  <TooltipRoot :delay-duration="delayDuration">
    <TooltipTrigger as-child>
      <slot name="trigger" />
    </TooltipTrigger>
    <TooltipPortal>
      <TooltipContent :class="contentClasses" :side="side" :side-offset="sideOffset">
        <slot>{{ content }}</slot>
        <TooltipArrow class="hermes-tooltip-arrow" />
      </TooltipContent>
    </TooltipPortal>
  </TooltipRoot>
</template>

<style scoped>
.hermes-tooltip-content {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, #041215);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-xs);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  line-height: 1.3;
  z-index: 150;
  animation: tooltip-in 150ms ease;
}

.hermes-tooltip-arrow {
  fill: var(--hh-surface-deep, #041215);
}

@keyframes tooltip-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
</style>
