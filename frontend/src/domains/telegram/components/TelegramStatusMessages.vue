<script setup lang="ts">
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()
const realtimeStatus = useRealtimeStatusStore()

defineProps<{
  actionMessage: string
  error: string
  realtimeStatusLabel: string
  realtimeStatusDetail: string
  realtimeRecoveryDetail: string
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
  <p class="telegram-recovery-state" :title="realtimeRecoveryDetail" :aria-label="realtimeRecoveryDetail">
    {{ t('Recovery') }}: {{ realtimeRecoveryDetail }}
    <button
      v-if="realtimeStatus.canTriggerReconnect"
      type="button"
      class="telegram-recovery-state__action"
      :title="t('Reconnect realtime')"
      @click="realtimeStatus.requestReconnect()"
    >
      {{ t('Reconnect realtime') }}
    </button>
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
.telegram-recovery-state {
  padding: 4px 16px 6px;
  margin: 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
  background: var(--color-surface-subtle, #f7f7f7);
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-recovery-state__action {
  margin-left: 8px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--color-accent, #2563eb);
  font: inherit;
  cursor: pointer;
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
