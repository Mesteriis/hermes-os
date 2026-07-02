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

type DefaultToastItem = Omit<ToastItem, 'id'> & {
  id?: string
}

const TOAST_INJECTION_KEY = 'hermes-toast-context'

const props = withDefaults(defineProps<{
  /** Swipe direction to dismiss */
  swipeDirection?: 'right' | 'left' | 'up' | 'down'
  /** Duration in ms before auto-dismiss */
  duration?: number
  /** Accessible label for dismiss controls. */
  closeLabel?: string
  /** Initial items for Storybook and deterministic visual tests. */
  defaultToasts?: DefaultToastItem[]
  class?: string
}>(), {
  swipeDirection: 'right',
  duration: 4000,
  closeLabel: 'Dismiss notification',
  defaultToasts: () => []
})

const toasts = ref<ToastItem[]>(
  props.defaultToasts.map((toast, index) => ({
    ...toast,
    id: toast.id ?? `default-toast-${index + 1}`
  }))
) as Ref<ToastItem[]>

let toastCounter = props.defaultToasts.length

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

    <ToastViewport as="div" :class="viewportClasses">
      <ToastRoot
        v-for="toast in toasts"
        :key="toast.id"
        as="div"
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
          <ToastClose class="hermes-toast-close-btn" :aria-label="closeLabel">
            <Icon icon="tabler:x" size="1rem" />
          </ToastClose>
        </div>
      </ToastRoot>
    </ToastViewport>
  </ToastProvider>
</template>
