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

