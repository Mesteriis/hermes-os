<script setup lang="ts">
import { computed } from 'vue'

type PresenceStatus = 'online' | 'away' | 'busy' | 'offline' | 'unknown'

const props = withDefaults(defineProps<{
  status?: PresenceStatus
  label?: string
  showLabel?: boolean
  size?: 'sm' | 'md' | 'lg'
  class?: string
}>(), {
  status: 'unknown',
  showLabel: true,
  size: 'md'
})

const statusTone: Record<PresenceStatus, string> = {
  online: 'success',
  away: 'warning',
  busy: 'danger',
  offline: 'neutral',
  unknown: 'info'
}

const defaultLabels: Record<PresenceStatus, string> = {
  online: 'Online',
  away: 'Away',
  busy: 'Busy',
  offline: 'Offline',
  unknown: 'Unknown'
}

const label = computed(() => props.label ?? defaultLabels[props.status])

const classes = computed(() => [
  'hermes-presence-indicator',
  `hermes-presence-indicator--${props.status}`,
  `hermes-presence-indicator--${props.size}`,
  `hermes-status-indicator--${statusTone[props.status]}`,
  props.class
])
</script>

<template>
  <span :class="classes" :aria-label="label">
    <span class="hermes-status-indicator-dot" aria-hidden="true" />
    <span v-if="showLabel" class="hermes-status-indicator-label">{{ label }}</span>
  </span>
</template>
