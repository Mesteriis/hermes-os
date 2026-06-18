<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramQrLoginStatusResponse } from '../types/telegram'
import type { TelegramAccountSetupFormValues } from '../forms/telegramAccountSetupForm'
import {
  useCancelTelegramQrLoginMutation,
  useStartTelegramQrLoginMutation,
  useSubmitTelegramQrPasswordMutation,
  useTelegramQrLoginStatusQuery,
} from '../queries/useTelegramQrLoginQuery'

const { t } = useI18n()

const props = defineProps<{
  formValues: TelegramAccountSetupFormValues
}>()

const emit = defineEmits<{
  applySuggested: [payload: {
    account_id: string
    display_name: string
    external_account_id: string
    qr_authorized: boolean
  }]
}>()

const setupId = ref<string | null>(null)
const localStatus = ref<TelegramQrLoginStatusResponse | null>(null)
const password = ref('')
const startMutation = useStartTelegramQrLoginMutation()
const statusQuery = useTelegramQrLoginStatusQuery(setupId)
const cancelMutation = useCancelTelegramQrLoginMutation(setupId)
const passwordMutation = useSubmitTelegramQrPasswordMutation(setupId)

const activeStatus = computed(() => statusQuery.data.value ?? localStatus.value)
const isBusy = computed(
  () =>
    startMutation.isPending.value ||
    statusQuery.isFetching.value ||
    cancelMutation.isPending.value ||
    passwordMutation.isPending.value
)
const canStart = computed(
  () =>
    props.formValues.provider_kind === 'telegram_user' &&
    props.formValues.account_id.trim().length > 0 &&
    props.formValues.display_name.trim().length > 0 &&
    props.formValues.external_account_id.trim().length > 0 &&
    (props.formValues.tdlib_data_path?.trim().length ?? 0) > 0
)

watch(
  () => statusQuery.data.value,
  (value) => {
    if (!value) return
    localStatus.value = value
  }
)

function clearSessionState() {
  setupId.value = null
  localStatus.value = null
  password.value = ''
}

async function startQrLogin() {
  const response = await startMutation.mutateAsync({
    account_id: props.formValues.account_id.trim(),
    display_name: props.formValues.display_name.trim(),
    external_account_id: props.formValues.external_account_id.trim(),
    api_id: props.formValues.api_id,
    api_hash: props.formValues.api_hash?.trim() || undefined,
    session_encryption_key: props.formValues.session_encryption_key?.trim() || undefined,
    tdlib_data_path: props.formValues.tdlib_data_path?.trim() || undefined,
    transcription_enabled: props.formValues.transcription_enabled,
  })
  setupId.value = response.setup_id
  localStatus.value = response
  password.value = ''
}

async function cancelQrLogin() {
  await cancelMutation.mutateAsync()
  clearSessionState()
}

async function submitPassword() {
  const response = await passwordMutation.mutateAsync({ password: password.value })
  localStatus.value = response
  password.value = ''
}

async function refreshStatus() {
  await statusQuery.refetch()
}

function applySuggested() {
  const status = activeStatus.value
  if (!status) return
  emit('applySuggested', {
    account_id: status.suggested_account_id ?? props.formValues.account_id.trim(),
    display_name: status.suggested_display_name ?? props.formValues.display_name.trim(),
    external_account_id: status.suggested_external_account_id ?? props.formValues.external_account_id.trim(),
    qr_authorized: status.status === 'ready',
  })
}
</script>

<template>
  <section class="telegram-qr-panel">
    <header class="telegram-qr-panel__header">
      <div>
        <strong>{{ t('QR Login') }}</strong>
        <p>{{ t('Use TDLib QR authorization for a Telegram user account before saving it locally.') }}</p>
      </div>
      <button type="button" :disabled="isBusy || !canStart" @click="void startQrLogin()">
        <Icon icon="tabler:qrcode" width="16" height="16" />
        {{ t('Start QR') }}
      </button>
    </header>

    <p v-if="!canStart" class="telegram-qr-panel__hint">
      {{ t('Fill account ID, display name, external account ID and TDLib data path before starting QR login.') }}
    </p>

    <div v-if="activeStatus" class="telegram-qr-panel__status">
      <div class="telegram-qr-panel__meta">
        <strong>{{ activeStatus.status }}</strong>
        <small>{{ activeStatus.message ?? activeStatus.setup_id }}</small>
      </div>

      <div v-if="activeStatus.qr_svg" class="telegram-qr-panel__qr" v-html="activeStatus.qr_svg" />

      <label v-if="activeStatus.status === 'waiting_password'" class="telegram-qr-panel__password">
        <span>{{ t('2FA Password') }}</span>
        <input v-model="password" type="password" autocomplete="off" />
      </label>

      <div class="telegram-qr-panel__actions">
        <button
          v-if="activeStatus.status === 'waiting_qr_scan' || activeStatus.status === 'waiting_password'"
          type="button"
          :disabled="isBusy"
          @click="void refreshStatus()"
        >
          <Icon icon="tabler:refresh" width="16" height="16" />
          {{ t('Refresh Status') }}
        </button>
        <button
          v-if="activeStatus.status === 'waiting_password'"
          type="button"
          :disabled="isBusy || password.trim().length === 0"
          @click="void submitPassword()"
        >
          {{ t('Submit Password') }}
        </button>
        <button
          v-if="activeStatus.status === 'ready'"
          type="button"
          :disabled="isBusy"
          @click="applySuggested"
        >
          {{ t('Apply Suggested Account') }}
        </button>
        <button
          v-if="activeStatus.status !== 'ready' && activeStatus.status !== 'expired' && activeStatus.status !== 'failed'"
          type="button"
          :disabled="isBusy"
          @click="void cancelQrLogin()"
        >
          {{ t('Cancel QR') }}
        </button>
      </div>

      <dl v-if="activeStatus.status === 'ready'" class="telegram-qr-panel__identity">
        <div><dt>{{ t('Suggested Account ID') }}</dt><dd>{{ activeStatus.suggested_account_id ?? '—' }}</dd></div>
        <div><dt>{{ t('Suggested Display Name') }}</dt><dd>{{ activeStatus.suggested_display_name ?? '—' }}</dd></div>
        <div><dt>{{ t('Suggested External ID') }}</dt><dd>{{ activeStatus.suggested_external_account_id ?? '—' }}</dd></div>
      </dl>
    </div>
  </section>
</template>

<style scoped>
.telegram-qr-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 10px;
  background: var(--color-surface, #fff);
}

.telegram-qr-panel__header,
.telegram-qr-panel__actions {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.telegram-qr-panel__header p,
.telegram-qr-panel__hint,
.telegram-qr-panel__meta small,
.telegram-qr-panel__identity dt {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-qr-panel__header button,
.telegram-qr-panel__actions button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-surface, #fff);
  padding: 5px 10px;
  font-size: 12px;
  cursor: pointer;
}

.telegram-qr-panel__status {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-qr-panel__meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.telegram-qr-panel__qr {
  display: flex;
  justify-content: center;
  padding: 8px;
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
}

.telegram-qr-panel__password {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.telegram-qr-panel__password input {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  padding: 6px 8px;
  font-size: 12px;
  background: var(--color-surface, #fff);
}

.telegram-qr-panel__identity {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 8px;
  margin: 0;
}

.telegram-qr-panel__identity dd {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}
</style>
