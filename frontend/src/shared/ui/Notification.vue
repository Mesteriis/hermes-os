<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

type FeedbackTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  tone?: FeedbackTone
  icon?: string
  dismissible?: boolean
  closeLabel?: string
  class?: string
}>(), {
  tone: 'neutral',
  dismissible: false,
  closeLabel: 'Dismiss notification'
})

const emit = defineEmits<{
  close: []
}>()

const toneIcons: Record<FeedbackTone, string> = {
  neutral: 'tabler:bell',
  info: 'tabler:info-circle',
  success: 'tabler:check-circle',
  warning: 'tabler:alert-triangle',
  danger: 'tabler:alert-circle'
}

const classes = computed(() => [
  'hermes-feedback',
  'hermes-notification',
  `hermes-feedback--${props.tone}`,
  props.class
])

const role = computed(() => props.tone === 'danger' ? 'alert' : 'status')
const resolvedIcon = computed(() => props.icon ?? toneIcons[props.tone])
</script>

<template>
  <div :class="classes" :role="role">
    <Icon :icon="resolvedIcon" size="1.125rem" class="hermes-feedback-icon" />
    <div class="hermes-feedback-body">
      <strong v-if="title" class="hermes-feedback-title">{{ title }}</strong>
      <p v-if="description" class="hermes-feedback-description">{{ description }}</p>
      <slot />
    </div>
    <button
      v-if="dismissible"
      type="button"
      class="hermes-feedback-close"
      :aria-label="closeLabel"
      @click="emit('close')"
    >
      <Icon icon="tabler:x" size="1rem" />
    </button>
  </div>
</template>
