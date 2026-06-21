<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import TelegramAccountManager from '../components/TelegramAccountManager.vue'
import TelegramCapabilityMatrix from '../components/TelegramCapabilityMatrix.vue'
import {
  useTelegramAccountsQuery,
  useTelegramCapabilitiesQuery,
} from '../queries/useTelegramQuery'
import {
  useRestartTelegramRuntimeMutation,
  useStartTelegramRuntimeMutation,
  useStopTelegramRuntimeMutation,
  useTelegramRuntimeStatusQuery,
} from '../queries/useTelegramRuntimeQuery'

const { t } = useI18n()
const realtimeStatus = useRealtimeStatusStore()
const selectedAccountId = ref<string | null>(null)
const actionMessage = ref('')
const actionError = ref('')

const accountsQuery = useTelegramAccountsQuery()
const capabilitiesQuery = useTelegramCapabilitiesQuery()
const accounts = computed(() => accountsQuery.data.value ?? [])
const selectedAccount = computed(() =>
  accounts.value.find((account) => account.account_id === selectedAccountId.value) ?? accounts.value[0] ?? null
)
const runtimeStatusQuery = useTelegramRuntimeStatusQuery(computed(() => selectedAccount.value?.account_id ?? null))
const startRuntimeMutation = useStartTelegramRuntimeMutation()
const stopRuntimeMutation = useStopTelegramRuntimeMutation()
const restartRuntimeMutation = useRestartTelegramRuntimeMutation()
const isRuntimeBusy = computed(() =>
  startRuntimeMutation.isPending.value ||
  stopRuntimeMutation.isPending.value ||
  restartRuntimeMutation.isPending.value
)

async function setTelegramRuntime(action: 'start' | 'stop' | 'restart') {
  const accountId = selectedAccount.value?.account_id
  if (!accountId || isRuntimeBusy.value) return
  actionMessage.value = ''
  actionError.value = ''
  try {
    const mutation = action === 'start'
      ? startRuntimeMutation
      : action === 'stop'
        ? stopRuntimeMutation
        : restartRuntimeMutation
    const status = await mutation.mutateAsync({ account_id: accountId })
    actionMessage.value = `Telegram runtime ${status.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}
</script>

<template>
  <section class="telegram-runtime-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-telegram" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('Telegram Runtime') }}</h1>
          <p>{{ t('Provider setup, runtime status and control') }}</p>
        </div>
      </div>
      <button type="button" class="primary-button" :disabled="accountsQuery.isFetching.value" @click="accountsQuery.refetch()">
        <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Refresh') }}
      </button>
    </header>

    <p
      class="telegram-realtime-state"
      :class="realtimeStatus.realtimeStatusTone"
      :title="realtimeStatus.realtimeStatusDetail"
    >
      {{ t('Realtime') }}: {{ realtimeStatus.realtimeStatusLabel }}
    </p>
    <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
    <p v-if="actionError" class="inline-error">{{ actionError }}</p>

    <div class="telegram-runtime-grid">
      <section class="panel telegram-runtime-card">
        <header>
          <h2>{{ t('Accounts') }}</h2>
          <span>{{ accounts.length }}</span>
        </header>
        <label class="runtime-field">
          <span>{{ t('Selected account') }}</span>
          <select v-model="selectedAccountId">
            <option :value="null">{{ t('Auto') }}</option>
            <option v-for="account in accounts" :key="account.account_id" :value="account.account_id">
              {{ account.display_name }} · {{ account.account_id }}
            </option>
          </select>
        </label>
        <TelegramAccountManager :selectedAccountId="selectedAccount?.account_id ?? null" />
      </section>

      <section class="panel telegram-runtime-card">
        <header>
          <h2>{{ t('Runtime') }}</h2>
          <span>{{ runtimeStatusQuery.data.value?.status ?? t('unknown') }}</span>
        </header>
        <div class="runtime-actions">
          <button type="button" :disabled="isRuntimeBusy || !selectedAccount" @click="setTelegramRuntime('start')">
            <Icon icon="tabler:player-play" width="16" height="16" />{{ t('Start') }}
          </button>
          <button type="button" :disabled="isRuntimeBusy || !selectedAccount" @click="setTelegramRuntime('stop')">
            <Icon icon="tabler:player-stop" width="16" height="16" />{{ t('Stop') }}
          </button>
          <button type="button" :disabled="isRuntimeBusy || !selectedAccount" @click="setTelegramRuntime('restart')">
            <Icon icon="tabler:reload" width="16" height="16" />{{ t('Restart') }}
          </button>
        </div>
        <dl class="runtime-details">
          <div><dt>{{ t('Account') }}</dt><dd>{{ selectedAccount?.account_id ?? '—' }}</dd></div>
          <div><dt>{{ t('Mode') }}</dt><dd>{{ capabilitiesQuery.data.value?.runtime_mode ?? '—' }}</dd></div>
          <div><dt>TDLib</dt><dd>{{ runtimeStatusQuery.data.value?.tdjson_runtime_available ? t('available') : t('unavailable') }}</dd></div>
          <div><dt>{{ t('Last sync') }}</dt><dd>{{ runtimeStatusQuery.data.value?.last_sync_status ?? '—' }}</dd></div>
        </dl>
      </section>

      <TelegramCapabilityMatrix :accountId="selectedAccount?.account_id ?? null" />
    </div>
  </section>
</template>

<style scoped>
.telegram-runtime-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: auto;
}
.view-header,
.view-title-with-icon,
.telegram-runtime-card header,
.runtime-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header,
.telegram-runtime-card header {
  justify-content: space-between;
}
.view-header {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.telegram-runtime-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 1rem;
  padding: 1rem;
}
.telegram-runtime-card {
  padding: 1rem;
}
.runtime-field,
.runtime-details {
  display: grid;
  gap: 0.5rem;
}
.runtime-field {
  margin-bottom: 1rem;
}
.runtime-field select {
  min-height: 2rem;
}
.runtime-actions button,
.primary-button {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
  background: var(--hh-bg-primary, #fff);
  color: inherit;
  padding: 0.4rem 0.65rem;
  cursor: pointer;
}
.runtime-details {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 1rem 0;
}
.runtime-details dd {
  margin: 0.15rem 0 0;
}
.telegram-realtime-state,
.setup-state,
.inline-error {
  padding: 0.6rem 1rem;
  margin: 0;
}
.success {
  color: #206a3a;
}
.inline-error {
  color: #b42318;
}
</style>
