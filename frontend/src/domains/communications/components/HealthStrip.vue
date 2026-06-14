<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { MailboxHealth } from '../types/communications'

const props = defineProps<{
  health: MailboxHealth | null
}>()

function healthToneClass(value: number, max: number): string {
  const ratio = value / Math.max(max, 1)
  if (ratio > 0.8) return 'health-item--danger'
  if (ratio > 0.5) return 'health-item--warning'
  return 'health-item--success'
}
</script>

<template>
  <div v-if="health" class="health-strip">
    <div class="health-item">
      <Icon icon="tabler:mail" class="health-icon" />
      <span class="health-value">{{ health.total_messages }}</span>
      <span class="health-label">Total</span>
    </div>
    <div class="health-item" :class="healthToneClass(health.unread, health.total_messages)">
      <Icon icon="tabler:mail-opened" class="health-icon" />
      <span class="health-value">{{ health.unread }}</span>
      <span class="health-label">Unread</span>
    </div>
    <div class="health-item" :class="healthToneClass(health.needs_action, health.total_messages)">
      <Icon icon="tabler:alert-circle" class="health-icon" />
      <span class="health-value">{{ health.needs_action }}</span>
      <span class="health-label">Action</span>
    </div>
    <div class="health-item" :class="healthToneClass(health.waiting, health.total_messages)">
      <Icon icon="tabler:clock" class="health-icon" />
      <span class="health-value">{{ health.waiting }}</span>
      <span class="health-label">Waiting</span>
    </div>
    <div class="health-item">
      <Icon icon="tabler:star" class="health-icon" />
      <span class="health-value">{{ health.important }}</span>
      <span class="health-label">Important</span>
    </div>
  </div>
</template>

<style scoped>
.health-strip {
  display: flex;
  gap: 1rem;
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-secondary, #f9fafb);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  overflow-x: auto;
}

.health-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.75rem;
  white-space: nowrap;
}

.health-item--danger {
  color: var(--hh-status-danger-text, #ef4444);
}

.health-item--warning {
  color: var(--hh-status-warning-text, #f59e0b);
}

.health-item--success {
  color: var(--hh-status-success-text, #16a34a);
}

.health-icon {
  width: 14px;
  height: 14px;
}

.health-value {
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.health-label {
  color: var(--hh-text-secondary, #6b7280);
}
</style>
