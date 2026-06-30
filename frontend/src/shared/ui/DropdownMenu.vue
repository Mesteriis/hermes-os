<script setup lang="ts">
import { DropdownMenuRoot, DropdownMenuTrigger, DropdownMenuPortal, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuLabel } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  align?: 'start' | 'center' | 'end'
  side?: 'top' | 'bottom'
  sideOffset?: number
}>(), {
  align: 'start',
  side: 'bottom',
  sideOffset: 4
})

const contentClasses = computed(() => ['hermes-dropdown-content', props.class])
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger as-child>
      <slot name="trigger" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        :class="contentClasses"
        :align="align"
        :side="side"
        :side-offset="sideOffset"
      >
        <slot />
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>

