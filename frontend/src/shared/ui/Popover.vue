<script setup lang="ts">
import { PopoverRoot, PopoverTrigger, PopoverPortal, PopoverContent, PopoverArrow, PopoverClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  side?: 'top' | 'bottom' | 'left' | 'right'
  sideOffset?: number
  align?: 'start' | 'center' | 'end'
  class?: string
}>(), {
  side: 'bottom',
  sideOffset: 4,
  align: 'center'
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
          <button class="hermes-popover-close-btn">
            <Icon icon="tabler:x" size="0.875rem" />
          </button>
        </PopoverClose>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>

<style scoped>
.hermes-popover-content {
  min-width: 200px;
  padding: 1rem;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  animation: popover-in 150ms ease;
}

.hermes-popover-arrow {
  fill: var(--hh-surface-panel);
}

.hermes-popover-close {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
}

.hermes-popover-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: var(--hh-radius-xs);
  border: none;
  background: transparent;
  color: var(--hh-text-muted);
  cursor: pointer;
  transition: all 150ms ease;
}

.hermes-popover-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes popover-in {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
