<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()

defineProps<{
  actionMessage: string
  error: string
  realtimeStatusLabel: string
  realtimeStatusDetail: string
  realtimeStatusTone: 'neutral' | 'success' | 'warning' | 'danger'
}>()
</script>

<template>
  <p
    class="telegram-realtime-state"
    :class="realtimeStatusTone"
    :title="realtimeStatusDetail"
    :aria-label="realtimeStatusDetail"
  >
    {{ t('Realtime') }}: {{ realtimeStatusLabel }}
  </p>
  <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
  <p v-if="error" class="inline-error">{{ error }}</p>
</template>

<style scoped>
.setup-state {
  padding: 8px 16px;
  margin: 0;
  font-size: 13px;
  border-radius: 6px;
}
.success {
  background: var(--color-success-bg, #e6f7e6);
  color: var(--color-success-text, #2e7d32);
}
.inline-error {
  padding: 8px 16px;
  margin: 0;
  font-size: 13px;
  background: var(--color-error-bg, #fdecea);
  color: var(--color-error-text, #c62828);
  border-radius: 6px;
}
.telegram-realtime-state {
  padding: 6px 16px;
  margin: 0;
  font-size: 12px;
  color: var(--color-text-muted, #666);
  background: var(--color-surface-subtle, #f7f7f7);
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-realtime-state.success {
  color: var(--color-success-text, #2e7d32);
}
.telegram-realtime-state.warning {
  color: var(--color-warning-text, #946200);
}
.telegram-realtime-state.danger {
  color: var(--color-error-text, #c62828);
}
</style>
