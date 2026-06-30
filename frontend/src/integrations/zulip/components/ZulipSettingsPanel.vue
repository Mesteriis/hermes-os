<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ProviderAccount } from '../../../shared/zulip/settingsBridge'
import {
  useEnqueueZulipDirectUploadCommandMutation,
  useEnqueueZulipStreamUploadCommandMutation,
  useEnqueueZulipUploadCommandMutation,
  useSetupZulipBotAccountMutation,
} from '../queries/useZulipRuntimeQuery'
import type { ZulipCommandEnqueueResponse } from '../types/zulip'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const { t } = useI18n()

const setupForm = ref({
  account_id: '',
  display_name: '',
  external_account_id: '',
  base_url: '',
  api_key: '',
})

const uploadRefForm = ref({
  attachment_id: '',
  blob_id: '',
  filename: '',
  idempotency_key: '',
})

const streamForm = ref({
  stream: '',
  topic: '',
  content: '',
})

const directForm = ref({
  recipients: '',
  content: '',
})

const activeAction = ref<string | null>(null)
const actionMessage = ref('')
const errorMessage = ref('')
const lastCommand = ref<ZulipCommandEnqueueResponse | null>(null)

const setupAccount = useSetupZulipBotAccountMutation()
const streamUploadCommand = useEnqueueZulipStreamUploadCommandMutation(() => selectedZulipAccountId.value)
const directUploadCommand = useEnqueueZulipDirectUploadCommandMutation(() => selectedZulipAccountId.value)
const uploadCommand = useEnqueueZulipUploadCommandMutation(() => selectedZulipAccountId.value)

const isSelectedZulip = computed(() => props.selectedAccount?.provider_kind === 'zulip_bot')
const selectedZulipAccountId = computed(() =>
  isSelectedZulip.value ? props.selectedAccount?.account_id ?? null : null
)
const selectedConfig = computed<Record<string, unknown>>(() =>
  asRecord(props.selectedAccount?.config) ?? {}
)
const selectedBaseUrl = computed(() => selectedString(selectedConfig.value, 'base_url') ?? '-')
const selectedRuntime = computed(() => selectedString(selectedConfig.value, 'runtime') ?? 'bot_api')
const selectedCredentialState = computed(() => {
  const credentials = asRecord(selectedConfig.value.credentials)
  return selectedString(credentials, 'store_kind') ?? selectedString(credentials, 'secret_kind') ?? 'host_vault'
})
const selectedLabel = computed(() => {
  const account = props.selectedAccount
  if (!account) return ''
  return account.display_name || account.external_account_id || account.account_id
})
const canRunSelectedCommands = computed(() => Boolean(selectedZulipAccountId.value))
const isBusy = computed(() =>
  setupAccount.isPending.value ||
  streamUploadCommand.isPending.value ||
  directUploadCommand.isPending.value ||
  uploadCommand.isPending.value ||
  activeAction.value !== null
)

async function handleSetup() {
  const account_id = setupForm.value.account_id.trim()
  const display_name = setupForm.value.display_name.trim()
  const external_account_id = setupForm.value.external_account_id.trim()
  const base_url = setupForm.value.base_url.trim()
  const api_key = setupForm.value.api_key.trim()
  if (!account_id || !display_name || !external_account_id || !base_url || !api_key) {
    setError(t('Account id, display name, bot email, realm URL and API key are required'))
    return
  }

  activeAction.value = 'setup'
  clearMessages()
  try {
    await setupAccount.mutateAsync({
      account_id,
      display_name,
      external_account_id,
      base_url,
      api_key,
    })
    setupForm.value.api_key = ''
    actionMessage.value = t('Zulip bot account connected')
  } catch (error) {
    setError(error instanceof Error ? error.message : 'Zulip setup failed')
  } finally {
    activeAction.value = null
  }
}

async function handleStreamUploadCommand() {
  const stream = streamForm.value.stream.trim()
  const topic = streamForm.value.topic.trim()
  const content = streamForm.value.content.trim()
  if (!stream || !topic || !content) {
    setError(t('Stream, topic and content are required'))
    return
  }
  const uploadRef = uploadRefOrError()
  if (!uploadRef) return

  activeAction.value = 'stream-upload'
  clearMessages()
  try {
    lastCommand.value = await streamUploadCommand.mutateAsync({
      ...uploadRef,
      idempotency_key: valueOrUndefined(uploadRefForm.value.idempotency_key),
      stream,
      topic,
      content,
    })
    actionMessage.value = t('Zulip stream upload command queued')
  } catch (error) {
    setError(error instanceof Error ? error.message : 'Zulip stream upload command failed')
  } finally {
    activeAction.value = null
  }
}

async function handleDirectUploadCommand() {
  const recipients = splitRecipients(directForm.value.recipients)
  const content = directForm.value.content.trim()
  if (!recipients.length || !content) {
    setError(t('Recipients and content are required'))
    return
  }
  const uploadRef = uploadRefOrError()
  if (!uploadRef) return

  activeAction.value = 'direct-upload'
  clearMessages()
  try {
    lastCommand.value = await directUploadCommand.mutateAsync({
      ...uploadRef,
      idempotency_key: valueOrUndefined(uploadRefForm.value.idempotency_key),
      recipients,
      content,
    })
    actionMessage.value = t('Zulip direct upload command queued')
  } catch (error) {
    setError(error instanceof Error ? error.message : 'Zulip direct upload command failed')
  } finally {
    activeAction.value = null
  }
}

async function handleUploadOnlyCommand() {
  const uploadRef = uploadRefOrError()
  if (!uploadRef) return

  activeAction.value = 'upload-only'
  clearMessages()
  try {
    lastCommand.value = await uploadCommand.mutateAsync({
      ...uploadRef,
      idempotency_key: valueOrUndefined(uploadRefForm.value.idempotency_key),
    })
    actionMessage.value = t('Zulip upload command queued')
  } catch (error) {
    setError(error instanceof Error ? error.message : 'Zulip upload command failed')
  } finally {
    activeAction.value = null
  }
}

function uploadRefOrError() {
  const attachment_id = valueOrUndefined(uploadRefForm.value.attachment_id)
  const blob_id = valueOrUndefined(uploadRefForm.value.blob_id)
  if (!attachment_id && !blob_id) {
    setError(t('Attachment id or blob id is required'))
    return null
  }
  return {
    attachment_id,
    blob_id,
    filename: valueOrUndefined(uploadRefForm.value.filename),
  }
}

function splitRecipients(input: string): string[] {
  return input
    .split(/[,\n]+/)
    .map((value) => value.trim())
    .filter(Boolean)
}

function valueOrUndefined(input: string): string | undefined {
  const trimmed = input.trim()
  return trimmed.length ? trimmed : undefined
}

function clearMessages() {
  errorMessage.value = ''
  actionMessage.value = ''
}

function setError(message: string) {
  actionMessage.value = ''
  errorMessage.value = message
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function selectedString(record: Record<string, unknown> | null, key: string): string | null {
  const value = record?.[key]
  return typeof value === 'string' && value.trim() ? value.trim() : null
}
</script>

<template>
  <section class="integration-section zulip-panel">
    <header class="panel-title-row">
      <div>
        <h3>{{ t('Zulip') }}</h3>
        <p class="integration-section-description">
          {{ t('Bot API account setup and reference-provider upload commands.') }}
        </p>
      </div>
    </header>

    <div v-if="actionMessage" class="setup-state success">{{ actionMessage }}</div>
    <div v-if="errorMessage" class="inline-error">{{ errorMessage }}</div>

    <form class="integration-form" @submit.prevent="handleSetup">
      <h4>{{ t('Connect bot account') }}</h4>
      <label>{{ t('Account id') }}<input v-model="setupForm.account_id" autocomplete="off" /></label>
      <label>{{ t('Display name') }}<input v-model="setupForm.display_name" autocomplete="off" /></label>
      <label>{{ t('Bot email') }}<input v-model="setupForm.external_account_id" autocomplete="off" placeholder="bot@example.zulipchat.com" /></label>
      <label>{{ t('Realm URL') }}<input v-model="setupForm.base_url" autocomplete="off" placeholder="https://example.zulipchat.com" /></label>
      <label>{{ t('API key') }}<input v-model="setupForm.api_key" type="password" autocomplete="off" /></label>
      <button type="submit" class="hermes-btn hermes-btn--primary" :disabled="isBusy">
        {{ t('Connect Zulip bot') }}
      </button>
    </form>

    <div v-if="isSelectedZulip" class="integration-section nested">
      <h4>{{ t('Selected Zulip account') }}: {{ selectedLabel }}</h4>
      <div class="zulip-state-grid">
        <div><span>{{ t('Realm') }}</span><strong>{{ selectedBaseUrl }}</strong></div>
        <div><span>{{ t('Runtime') }}</span><strong>{{ selectedRuntime }}</strong></div>
        <div><span>{{ t('Credential') }}</span><strong>{{ selectedCredentialState }}</strong></div>
      </div>

      <div class="integration-form compact">
        <h4>{{ t('Attachment reference') }}</h4>
        <p class="integration-section-description">
          {{ t('Use an existing Communications attachment_id or local blob_id; file bytes stay behind the backend boundary.') }}
        </p>
        <label>{{ t('Attachment id') }}<input v-model="uploadRefForm.attachment_id" autocomplete="off" /></label>
        <label>{{ t('Blob id') }}<input v-model="uploadRefForm.blob_id" autocomplete="off" placeholder="blob:v1:..." /></label>
        <label>{{ t('Filename override') }}<input v-model="uploadRefForm.filename" autocomplete="off" /></label>
        <label>{{ t('Idempotency key') }}<input v-model="uploadRefForm.idempotency_key" autocomplete="off" /></label>
      </div>

      <div class="zulip-command-grid">
        <form class="integration-form command-card" @submit.prevent="handleStreamUploadCommand">
          <h4>{{ t('Stream message with upload') }}</h4>
          <label>{{ t('Stream') }}<input v-model="streamForm.stream" autocomplete="off" /></label>
          <label>{{ t('Topic') }}<input v-model="streamForm.topic" autocomplete="off" /></label>
          <label>{{ t('Content') }}<textarea v-model="streamForm.content" rows="4" /></label>
          <button type="submit" class="hermes-btn hermes-btn--outline" :disabled="!canRunSelectedCommands || isBusy">
            {{ t('Queue stream command') }}
          </button>
        </form>

        <form class="integration-form command-card" @submit.prevent="handleDirectUploadCommand">
          <h4>{{ t('Direct message with upload') }}</h4>
          <label>{{ t('Recipients') }}<textarea v-model="directForm.recipients" rows="3" placeholder="user@example.com, 123" /></label>
          <label>{{ t('Content') }}<textarea v-model="directForm.content" rows="4" /></label>
          <button type="submit" class="hermes-btn hermes-btn--outline" :disabled="!canRunSelectedCommands || isBusy">
            {{ t('Queue direct command') }}
          </button>
        </form>
      </div>

      <div class="upload-only-row">
        <button type="button" class="hermes-btn hermes-btn--secondary" :disabled="!canRunSelectedCommands || isBusy" @click="handleUploadOnlyCommand">
          {{ t('Queue upload only') }}
        </button>
        <p class="integration-section-description">
          {{ t('Upload-only commands stage provider media without creating durable business truth.') }}
        </p>
      </div>

      <div v-if="lastCommand" class="last-command">
        <h4>{{ t('Last queued command') }}</h4>
        <div class="zulip-state-grid">
          <div><span>{{ t('Command') }}</span><strong>{{ lastCommand.command_kind }}</strong></div>
          <div><span>{{ t('Status') }}</span><strong>{{ lastCommand.status }}</strong></div>
          <div><span>{{ t('Reconciliation') }}</span><strong>{{ lastCommand.reconciliation_status }}</strong></div>
        </div>
        <code>{{ lastCommand.command_id }}</code>
      </div>
    </div>
  </section>
</template>

<style scoped>
.zulip-panel {
  display: grid;
  gap: 12px;
}

.integration-section {
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  background: var(--hh-surface-deep);
  padding: 12px;
}

.integration-section.nested {
  display: grid;
  gap: 12px;
  margin-top: 4px;
}

.integration-section h3,
.integration-section h4 {
  margin: 0 0 6px;
}

.integration-section-description {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--hh-text-muted);
}

.integration-form {
  display: grid;
  gap: 8px;
}

.integration-form.compact {
  border-top: 1px solid var(--hh-border);
  padding-top: 10px;
}

.integration-form label {
  display: grid;
  gap: 4px;
  font-size: 11px;
  color: var(--hh-text-muted);
}

.integration-form input,
.integration-form textarea {
  width: 100%;
  min-width: 0;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-input-bg, var(--hh-surface));
  color: var(--hh-text-primary);
  font: inherit;
  font-size: 12px;
  padding: 8px;
  outline: none;
}

.integration-form textarea {
  resize: vertical;
}

.integration-form input:focus-visible,
.integration-form textarea:focus-visible {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
}

.zulip-state-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}

.zulip-state-grid div {
  display: grid;
  gap: 3px;
  min-width: 0;
  padding: 8px;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-panel-bg);
}

.zulip-state-grid span {
  font-size: 11px;
  color: var(--hh-text-muted);
}

.zulip-state-grid strong {
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--hh-text-primary);
  font-size: 12px;
}

.zulip-command-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.command-card {
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  padding: 10px;
  background: var(--hh-panel-bg);
}

.upload-only-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.upload-only-row p {
  margin: 0;
}

.last-command {
  display: grid;
  gap: 8px;
  border-top: 1px solid var(--hh-border);
  padding-top: 10px;
}

.last-command code {
  overflow-wrap: anywhere;
  color: var(--hh-text-secondary);
  font-size: 12px;
}

.setup-state.success,
.inline-error {
  padding: 8px 12px;
  border-radius: var(--hh-radius-sm);
  font-size: 12px;
}

.setup-state.success {
  background: color-mix(in srgb, var(--hh-status-success) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-success) 30%, transparent);
  color: var(--hh-status-success);
}

.inline-error {
  background: color-mix(in srgb, var(--hh-status-danger) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  color: var(--hh-status-danger);
}
</style>
