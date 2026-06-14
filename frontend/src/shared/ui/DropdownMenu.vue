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

<style scoped>
.hermes-dropdown-content {
  min-width: 180px;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  padding: 0.25rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  animation: dropdown-in 150ms ease;
}

@keyframes dropdown-in {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
