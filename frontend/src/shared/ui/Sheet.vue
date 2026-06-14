<script setup lang="ts">
import { DialogRoot, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription, DialogClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  title?: string
  description?: string
  side?: 'left' | 'right' | 'top' | 'bottom'
  class?: string
  contentClass?: string
}>(), {
  open: false,
  side: 'right'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => [
  'hermes-sheet-content',
  `hermes-sheet--${props.side}`,
  props.contentClass
])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogTrigger as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="hermes-sheet-overlay">
        <DialogContent :class="contentClasses">
          <div class="hermes-sheet-header">
            <DialogTitle v-if="title" class="hermes-sheet-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="hermes-sheet-description">{{ description }}</DialogDescription>
            <slot name="header" />
          </div>
          <div class="hermes-sheet-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="hermes-sheet-footer">
            <slot name="footer" />
          </div>
          <DialogClose class="hermes-sheet-close" as-child>
            <button class="hermes-sheet-close-btn">
              <Icon icon="tabler:x" size="1.125rem" />
            </button>
          </DialogClose>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.hermes-sheet-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  z-index: 100;
  animation: sheet-overlay-in 200ms ease;
}

/* Side alignment */
.hermes-sheet--left {
  align-self: stretch;
  margin-right: auto;
  animation: sheet-slide-left 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.hermes-sheet--right {
  align-self: stretch;
  margin-left: auto;
  animation: sheet-slide-right 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.hermes-sheet--top {
  align-self: flex-start;
  width: 100%;
  animation: sheet-slide-top 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.hermes-sheet--bottom {
  align-self: flex-end;
  width: 100%;
  animation: sheet-slide-bottom 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.hermes-sheet-content {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 90vw;
  max-width: 400px;
  max-height: 100vh;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  box-shadow: var(--hh-shadow-modal);
  overflow-y: auto;
}

.hermes-sheet-header {
  padding: 1.5rem 1.5rem 0;
  flex-shrink: 0;
}

.hermes-sheet-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin-bottom: 0.25rem;
}

.hermes-sheet-description {
  font-size: 0.8125rem;
  color: var(--hh-text-muted);
  line-height: 1.4;
}

.hermes-sheet-body {
  padding: 1.25rem 1.5rem;
  flex: 1;
  overflow-y: auto;
}

.hermes-sheet-footer {
  padding: 1rem 1.5rem;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  border-top: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.hermes-sheet-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
}

.hermes-sheet-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--hh-radius-xs);
  color: var(--hh-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.hermes-sheet-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes sheet-overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes sheet-slide-right {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

@keyframes sheet-slide-left {
  from { transform: translateX(-100%); }
  to { transform: translateX(0); }
}

@keyframes sheet-slide-top {
  from { transform: translateY(-100%); }
  to { transform: translateY(0); }
}

@keyframes sheet-slide-bottom {
  from { transform: translateY(100%); }
  to { transform: translateY(0); }
}
</style>
