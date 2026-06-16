<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import TelegramCapabilityMatrix from './TelegramCapabilityMatrix.vue'
import TelegramQrLoginPanel from './TelegramQrLoginPanel.vue'
import {
  defaultTelegramAccountSetupValues,
  telegramAccountSetupSchema,
  type TelegramAccountSetupFormValues,
} from '../forms/telegramAccountSetupForm'
import {
  useLogoutTelegramAccountMutation,
  useRemoveTelegramAccountMutation,
  useSetupTelegramAccountMutation,
  useTelegramAccountsQuery,
} from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  selectedAccountId: string | null
}>()

const isSetupOpen = ref(false)
const setupMutation = useSetupTelegramAccountMutation()
const logoutMutation = useLogoutTelegramAccountMutation()
const removeMutation = useRemoveTelegramAccountMutation()
const accountsQuery = useTelegramAccountsQuery()

const {
  defineField,
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values,
} = useForm<TelegramAccountSetupFormValues>({
  validationSchema: telegramAccountSetupSchema,
  initialValues: defaultTelegramAccountSetupValues(),
})

const [accountId] = defineField('account_id')
const [providerKind] = defineField('provider_kind')
const [displayName] = defineField('display_name')
const [externalAccountId] = defineField('external_account_id')
const [apiId] = defineField('api_id')
const [apiHash] = defineField('api_hash')
const [botToken] = defineField('bot_token')
const [sessionEncryptionKey] = defineField('session_encryption_key')
const [tdlibDataPath] = defineField('tdlib_data_path')
const [qrAuthorized] = defineField('qr_authorized')
const [transcriptionEnabled] = defineField('transcription_enabled')

const isBusy = computed(
  () =>
    setupMutation.isPending.value ||
    logoutMutation.isPending.value ||
    removeMutation.isPending.value
)

const accounts = computed(() => accountsQuery.data.value ?? [])

const submit = handleSubmit(async (formValues) => {
  await setupMutation.mutateAsync({
    account_id: formValues.account_id.trim(),
    provider_kind: formValues.provider_kind,
    display_name: formValues.display_name.trim(),
    external_account_id: formValues.external_account_id.trim(),
    api_id: formValues.provider_kind === 'telegram_user' && !formValues.qr_authorized ? formValues.api_id : undefined,
    api_hash: formValues.provider_kind === 'telegram_user' && !formValues.qr_authorized ? formValues.api_hash?.trim() : undefined,
    bot_token: formValues.provider_kind === 'telegram_bot' ? formValues.bot_token?.trim() : undefined,
    session_encryption_key: formValues.session_encryption_key?.trim() || undefined,
    tdlib_data_path: formValues.tdlib_data_path?.trim() || undefined,
    qr_authorized: formValues.provider_kind === 'telegram_user' ? formValues.qr_authorized : false,
    transcription_enabled: formValues.transcription_enabled,
  })
  resetForm({ values: defaultTelegramAccountSetupValues() })
  isSetupOpen.value = false
})

async function logoutAccount(accountId: string) {
  await logoutMutation.mutateAsync(accountId)
}

async function removeAccount(accountId: string) {
  await removeMutation.mutateAsync(accountId)
}

function lifecycleTone(state: string): 'active' | 'muted' | 'danger' {
  if (state === 'removed') return 'danger'
  if (state === 'logged_out') return 'muted'
  return 'active'
}

function applyQrSuggestedAccount(payload: {
  account_id: string
  display_name: string
  external_account_id: string
  qr_authorized: boolean
}) {
  setFieldValue('account_id', payload.account_id)
  setFieldValue('display_name', payload.display_name)
  setFieldValue('external_account_id', payload.external_account_id)
  setFieldValue('qr_authorized', payload.qr_authorized)
}
</script>

<template>
  <article class="telegram-account-manager telegram-rail-card">
    <div class="telegram-account-manager__header">
      <div>
        <h3>{{ t('Accounts') }}</h3>
        <p>{{ t('Manage Telegram account metadata, setup and lifecycle locally.') }}</p>
      </div>
      <button type="button" @click="isSetupOpen = !isSetupOpen">
        <Icon :icon="isSetupOpen ? 'tabler:x' : 'tabler:user-plus'" width="16" height="16" />
        {{ isSetupOpen ? t('Close') : t('Add Account') }}
      </button>
    </div>

    <form v-if="isSetupOpen" class="telegram-account-manager__form" @submit.prevent="submit">
      <label>
        <span>{{ t('Account ID') }}</span>
        <input v-model="accountId" type="text" autocomplete="off" />
        <small>{{ errors.account_id }}</small>
      </label>
      <label>
        <span>{{ t('Provider Kind') }}</span>
        <select v-model="providerKind">
          <option value="telegram_user">telegram_user</option>
          <option value="telegram_bot">telegram_bot</option>
        </select>
        <small>{{ errors.provider_kind }}</small>
      </label>
      <label>
        <span>{{ t('Display Name') }}</span>
        <input v-model="displayName" type="text" autocomplete="off" />
        <small>{{ errors.display_name }}</small>
      </label>
      <label>
        <span>{{ t('External Account ID') }}</span>
        <input v-model="externalAccountId" type="text" autocomplete="off" />
        <small>{{ errors.external_account_id }}</small>
      </label>

      <template v-if="providerKind === 'telegram_user'">
        <label class="telegram-account-manager__checkbox">
          <input v-model="qrAuthorized" type="checkbox" />
          <span>{{ t('QR authorized user account') }}</span>
        </label>
        <label v-if="!qrAuthorized">
          <span>{{ t('API ID') }}</span>
          <input v-model.number="apiId" type="number" min="1" step="1" />
          <small>{{ errors.api_id }}</small>
        </label>
        <label v-if="!qrAuthorized">
          <span>{{ t('API hash') }}</span>
          <input v-model="apiHash" type="password" autocomplete="off" />
          <small>{{ errors.api_hash }}</small>
        </label>
      </template>

      <label v-else>
        <span>{{ t('Bot token') }}</span>
        <input v-model="botToken" type="password" autocomplete="off" />
        <small>{{ errors.bot_token }}</small>
      </label>

      <label>
        <span>{{ t('Session encryption key') }}</span>
        <input v-model="sessionEncryptionKey" type="password" autocomplete="off" />
        <small>{{ errors.session_encryption_key }}</small>
      </label>
      <label>
        <span>TDLib data path</span>
        <input v-model="tdlibDataPath" type="text" autocomplete="off" />
        <small>{{ errors.tdlib_data_path }}</small>
      </label>
      <label class="telegram-account-manager__checkbox">
        <input v-model="transcriptionEnabled" type="checkbox" />
        <span>{{ t('Enable transcription') }}</span>
      </label>
      <TelegramQrLoginPanel
        v-if="providerKind === 'telegram_user'"
        :formValues="values"
        @applySuggested="applyQrSuggestedAccount"
      />
      <div class="telegram-account-manager__actions">
        <button type="submit" :disabled="isBusy">
          <Icon icon="tabler:device-floppy" width="16" height="16" />
          {{ t('Save Account') }}
        </button>
      </div>
    </form>

    <div v-if="accounts.length === 0" class="telegram-account-manager__empty">
      {{ t('No Telegram accounts configured yet.') }}
    </div>
    <div v-else class="telegram-account-manager__list">
      <article
        v-for="account in accounts"
        :key="account.account_id"
        class="telegram-account-manager__item"
        :data-state="lifecycleTone(account.lifecycle_state)"
      >
        <div>
          <strong>{{ account.display_name }}</strong>
          <p>{{ account.account_id }} · {{ account.provider_kind }}</p>
          <small>{{ account.runtime }} · {{ account.lifecycle_state }}</small>
        </div>
        <div class="telegram-account-manager__item-actions">
          <button
            type="button"
            :disabled="isBusy || account.lifecycle_state !== 'active'"
            @click="void logoutAccount(account.account_id)"
          >
            {{ t('Logout') }}
          </button>
          <button
            type="button"
            class="danger"
            :disabled="isBusy || account.lifecycle_state === 'removed'"
            @click="void removeAccount(account.account_id)"
          >
            {{ t('Remove') }}
          </button>
        </div>
        <em v-if="props.selectedAccountId === account.account_id">{{ t('Selected') }}</em>
      </article>
    </div>

    <TelegramCapabilityMatrix :accountId="props.selectedAccountId" />
  </article>
</template>

<style scoped>
.telegram-account-manager__header,
.telegram-account-manager__item,
.telegram-account-manager__item-actions,
.telegram-account-manager__actions {
  display: flex;
  gap: 10px;
}
.telegram-account-manager__header,
.telegram-account-manager__item {
  justify-content: space-between;
  align-items: flex-start;
}
.telegram-account-manager__header p,
.telegram-account-manager__item p,
.telegram-account-manager__item small,
.telegram-account-manager__empty,
.telegram-account-manager__form small {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-account-manager__form,
.telegram-account-manager__list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 12px;
}
.telegram-account-manager__form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}
.telegram-account-manager__form input,
.telegram-account-manager__form select {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  padding: 6px 8px;
  font-size: 12px;
  background: var(--color-surface, #fff);
}
.telegram-account-manager__checkbox {
  flex-direction: row !important;
  align-items: center;
}
.telegram-account-manager__header button,
.telegram-account-manager__actions button,
.telegram-account-manager__item-actions button {
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
.telegram-account-manager__item {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 10px;
  background: var(--color-surface, #fff);
}
.telegram-account-manager__item[data-state='active'] {
  border-color: var(--color-primary-light, #bbdefb);
}
.telegram-account-manager__item[data-state='danger'] {
  opacity: 0.7;
}
.telegram-account-manager__item-actions .danger {
  color: #b42318;
}
</style>
