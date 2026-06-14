<script setup lang="ts">
import { ToastProvider, ToastViewport, ToastRoot, ToastTitle, ToastDescription, ToastClose } from 'reka-ui'
import { ref, computed, provide, inject, type Ref } from 'vue'
import Icon from './Icon.vue'

export interface ToastItem {
  id: string
  title?: string
  description?: string
  variant?: 'default' | 'success' | 'warning' | 'error'
  duration?: number
}

const TOAST_INJECTION_KEY = 'hermes-toast-context'

const props = withDefaults(defineProps<{
  /** Swipe direction to dismiss */
  swipeDirection?: 'right' | 'left' | 'up' | 'down'
  /** Duration in ms before auto-dismiss */
  duration?: number
  class?: string
}>(), {
  swipeDirection: 'right',
  duration: 4000
})

const toasts = ref<ToastItem[]>([]) as Ref<ToastItem[]>

let toastCounter = 0

function addToast(item: Omit<ToastItem, 'id'>): string {
  const id = `toast-${++toastCounter}`
  toasts.value = [...toasts.value, { ...item, id }]
  return id
}

function removeToast(id: string): void {
  toasts.value = toasts.value.filter((t) => t.id !== id)
}

function success(title: string, description?: string): string {
  return addToast({ title, description, variant: 'success', duration: props.duration })
}

function warning(title: string, description?: string): string {
  return addToast({ title, description, variant: 'warning', duration: props.duration })
}

function error(title: string, description?: string): string {
  return addToast({ title, description, variant: 'error', duration: props.duration })
}

provide(TOAST_INJECTION_KEY, { addToast, removeToast, success, warning, error })

const viewportClasses = computed(() => [
  'hermes-toast-viewport',
  props.class
])

const variantIcons: Record<string, string> = {
  success: 'tabler:check-circle',
  warning: 'tabler:alert-triangle',
  error: 'tabler:alert-circle'
}
</script>

<template>
  <ToastProvider :swipe-direction="swipeDirection" :duration="duration">
    <slot />

    <ToastViewport :class="viewportClasses">
      <ToastRoot
        v-for="toast in toasts"
        :key="toast.id"
        :class="['hermes-toast-root', `hermes-toast--${toast.variant || 'default'}`]"
        @update:open="(open: boolean) => { if (!open) removeToast(toast.id) }"
      >
        <div class="hermes-toast-inner">
          <Icon
            v-if="toast.variant && toast.variant !== 'default'"
            :icon="variantIcons[toast.variant]"
            size="1.125rem"
            class="hermes-toast-variant-icon"
          />
          <div class="hermes-toast-content">
            <ToastTitle v-if="toast.title" class="hermes-toast-title">
              {{ toast.title }}
            </ToastTitle>
            <ToastDescription v-if="toast.description" class="hermes-toast-description">
              {{ toast.description }}
            </ToastDescription>
          </div>
          <ToastClose class="hermes-toast-close-btn">
            <Icon icon="tabler:x" size="1rem" />
          </ToastClose>
        </div>
      </ToastRoot>
    </ToastViewport>
  </ToastProvider>
</template>

<style scoped>
.hermes-toast-viewport {
  position: fixed;
  bottom: 1rem;
  right: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0;
  max-width: 360px;
  width: 100%;
  z-index: 200;
  outline: none;
  list-style: none;
}

.hermes-toast-root {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: var(--hh-shadow-modal);
  padding: 0.875rem 1rem;
  animation: toast-slide-in 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.hermes-toast-root[data-state="closed"] {
  animation: toast-slide-out 200ms ease;
}

.hermes-toast-inner {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
}

.hermes-toast-variant-icon {
  flex-shrink: 0;
  margin-top: 0.0625rem;
}

.hermes-toast--success .hermes-toast-variant-icon {
  color: var(--hh-status-success, #22c55e);
}

.hermes-toast--warning .hermes-toast-variant-icon {
  color: var(--hh-status-warning, #f59e0b);
}

.hermes-toast--error .hermes-toast-variant-icon {
  color: var(--hh-status-danger, #ef4444);
}

.hermes-toast--success {
  border-color: color-mix(in srgb, var(--hh-status-success, #22c55e) 30%, transparent);
}

.hermes-toast--warning {
  border-color: color-mix(in srgb, var(--hh-status-warning, #f59e0b) 30%, transparent);
}

.hermes-toast--error {
  border-color: color-mix(in srgb, var(--hh-status-danger, #ef4444) 30%, transparent);
}

.hermes-toast-content {
  flex: 1;
  min-width: 0;
}

.hermes-toast-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  line-height: 1.4;
}

.hermes-toast-description {
  font-size: 0.75rem;
  color: var(--hh-text-secondary);
  line-height: 1.4;
  margin-top: 0.125rem;
}

.hermes-toast-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 1.375rem;
  height: 1.375rem;
  border-radius: var(--hh-radius-xs);
  color: var(--hh-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.hermes-toast-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes toast-slide-in {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

@keyframes toast-slide-out {
  from {
    transform: translateX(0);
    opacity: 1;
  }
  to {
    transform: translateX(100%);
    opacity: 0;
  }
}
</style>
