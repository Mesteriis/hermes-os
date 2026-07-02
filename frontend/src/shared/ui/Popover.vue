<script setup lang="ts">
import { PopoverRoot, PopoverTrigger, PopoverPortal, PopoverContent, PopoverArrow, PopoverClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  side?: 'top' | 'bottom' | 'left' | 'right'
  sideOffset?: number
  align?: 'start' | 'center' | 'end'
  closeLabel?: string
  class?: string
}>(), {
  side: 'bottom',
  sideOffset: 4,
  align: 'center',
  closeLabel: 'Close popover'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['hermes-popover-content', props.class])
</script>

<template>
  <PopoverRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <PopoverTrigger as-child>
      <slot name="trigger" />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent :class="contentClasses" :side="side" :side-offset="sideOffset" :align="align">
        <PopoverArrow class="hermes-popover-arrow" />
        <slot />
        <PopoverClose class="hermes-popover-close" as-child>
          <button class="hermes-popover-close-btn" :aria-label="closeLabel">
            <Icon icon="tabler:x" size="0.875rem" />
          </button>
        </PopoverClose>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
