<script setup lang="ts">
import { DialogRoot, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription, DialogClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  title?: string
  description?: string
  class?: string
  contentClass?: string
}>(), {
  open: false
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['hermes-dialog-content', props.contentClass])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogTrigger as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="hermes-dialog-overlay">
        <DialogContent :class="contentClasses">
          <div class="hermes-dialog-header">
            <DialogTitle v-if="title" class="hermes-dialog-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="hermes-dialog-description">{{ description }}</DialogDescription>
            <slot name="header" />
          </div>
          <div class="hermes-dialog-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="hermes-dialog-footer">
            <slot name="footer" />
          </div>
          <DialogClose class="hermes-dialog-close" as-child>
            <button class="hermes-dialog-close-btn">
              <Icon icon="tabler:x" size="1.125rem" />
            </button>
          </DialogClose>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.hermes-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: dialog-overlay-in 200ms ease;
}

.hermes-dialog-content {
  position: relative;
  width: 90vw;
  max-width: 500px;
  max-height: 85vh;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-lg);
  box-shadow: var(--hh-shadow-modal);
  overflow-y: auto;
  animation: dialog-content-in 200ms ease;
}

.hermes-dialog-header {
  padding: 1.5rem 1.5rem 0;
}

.hermes-dialog-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin: 0;
  line-height: 1.3;
}

.hermes-dialog-description {
  font-size: 0.8125rem;
  color: var(--hh-text-muted);
  margin: 0.25rem 0 0;
  line-height: 1.4;
}

.hermes-dialog-body {
  padding: 1.25rem 1.5rem;
}

.hermes-dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0 1.5rem 1.25rem;
}

.hermes-dialog-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
}

.hermes-dialog-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: var(--hh-radius-sm);
  border: none;
  background: transparent;
  color: var(--hh-text-muted);
  cursor: pointer;
  transition: all 150ms ease;
}

.hermes-dialog-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes dialog-overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes dialog-content-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
</style>
