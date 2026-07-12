<script setup lang="ts">
import { computed, ref } from 'vue'
import { Button, Icon } from '@/shared/ui'
import { useTelegramAutomationPoliciesQuery, useTelegramSendDryRunMutation } from '../queries/useTelegramAutomationQuery'
import { useTelegramCommandRetryMutation, useTelegramCommandsQuery } from '../queries/useTelegramLifecycleQuery'
import { useLogoutTelegramAccountMutation, useRemoveTelegramAccountMutation, useSetupTelegramAccountMutation, useSyncTelegramChatsMutation } from '../queries/useTelegramMutations'
import { useTelegramRuntimePanelSurface } from '../queries/useTelegramRuntimePanelSurface'
import { useTelegramCallTranscriptQuery } from '../queries/useTelegramQuery'
import { useTelegramAccountCapabilitiesQuery, useTelegramCallsQuery, useTelegramFoldersQuery, useTelegramProviderSearchMutation } from '../queries/useTelegramQuery'

const surface = useTelegramRuntimePanelSurface()
const selectedAccountId = computed(() => surface.selectedAccount.value?.account_id ?? null)
const accountCapabilitiesQuery = useTelegramAccountCapabilitiesQuery(selectedAccountId)
const commandsQuery = useTelegramCommandsQuery(selectedAccountId, 20)
const foldersQuery = useTelegramFoldersQuery(() => selectedAccountId.value ?? undefined)
const callsQuery = useTelegramCallsQuery(() => selectedAccountId.value ?? undefined, 20)
const syncChatsMutation = useSyncTelegramChatsMutation()
const logoutAccountMutation = useLogoutTelegramAccountMutation()
const removeAccountMutation = useRemoveTelegramAccountMutation()
const setupAccountMutation = useSetupTelegramAccountMutation()
const retryCommandMutation = useTelegramCommandRetryMutation()
const policiesQuery = useTelegramAutomationPoliciesQuery(selectedAccountId)
const dryRunMutation = useTelegramSendDryRunMutation()
const selectedCallId = ref<string | null>(null)
const selectedPolicyId = ref('')
const dryRunChatId = ref('')
const dryRunVariables = ref('{}')
const dryRunError = ref('')
const dryRunResult = ref('')
const providerSearchQuery = ref('')
const providerSearchStatus = ref('')
const providerSearchError = ref('')
const providerSearchMutation = useTelegramProviderSearchMutation()
const showSetup = ref(false)
const setupError = ref('')
const setupStatus = ref('')
const setupForm = ref({
  accountId: '',
  apiHash: '',
  apiId: '',
  botToken: '',
  displayName: '',
  externalAccountId: '',
  providerKind: 'telegram_user',
  qrAuthorized: false,
  sessionEncryptionKey: '',
  tdlibDataPath: '',
})
const transcriptQuery = useTelegramCallTranscriptQuery(selectedCallId)

async function syncChats(): Promise<void> {
  const accountId = selectedAccountId.value
  if (!accountId) return
  await syncChatsMutation.mutateAsync({ account_id: accountId, limit: 200 })
}

async function logoutAccount(): Promise<void> {
  const accountId = selectedAccountId.value
  if (!accountId || !window.confirm('Log out this Telegram account and stop its runtime?')) return
  await logoutAccountMutation.mutateAsync(accountId)
  await surface.refreshRuntime()
}

async function removeAccount(): Promise<void> {
  const accountId = selectedAccountId.value
  if (!accountId || !window.confirm('Remove this Telegram account locally? Source evidence remains preserved.')) return
  await removeAccountMutation.mutateAsync(accountId)
  await surface.refreshRuntime()
}

async function setupAccount(): Promise<void> {
  const form = setupForm.value
  const accountId = form.accountId.trim()
  const displayName = form.displayName.trim()
  const externalAccountId = form.externalAccountId.trim()
  if (!accountId || !displayName || !externalAccountId) return
  const apiId = form.apiId.trim() ? Number.parseInt(form.apiId, 10) : undefined
  if (apiId != null && !Number.isFinite(apiId)) {
    setupError.value = 'Telegram API id must be a number.'
    return
  }

  setupError.value = ''
  setupStatus.value = ''
  try {
    await setupAccountMutation.mutateAsync({
      account_id: accountId,
      provider_kind: form.providerKind,
      display_name: displayName,
      external_account_id: externalAccountId,
      api_id: apiId,
      api_hash: form.apiHash || undefined,
      bot_token: form.botToken || undefined,
      session_encryption_key: form.sessionEncryptionKey || undefined,
      tdlib_data_path: form.tdlibDataPath || undefined,
      qr_authorized: form.qrAuthorized,
      transcription_enabled: false,
    })
    setupForm.value.apiHash = ''
    setupForm.value.botToken = ''
    setupForm.value.sessionEncryptionKey = ''
    setupStatus.value = 'Telegram account was configured locally.'
    showSetup.value = false
    await surface.refreshRuntime()
  } catch (error) {
    setupError.value = error instanceof Error ? error.message : 'Telegram account setup failed.'
  }
}

async function retryCommand(commandId: string): Promise<void> {
  await retryCommandMutation.mutateAsync(commandId)
}

async function runDryRun(): Promise<void> {
  const policyId = selectedPolicyId.value.trim()
  const providerChatId = dryRunChatId.value.trim()
  if (!policyId || !providerChatId) return

  dryRunError.value = ''
  dryRunResult.value = ''
  try {
    const variables = JSON.parse(dryRunVariables.value) as Record<string, string>
    const result = await dryRunMutation.mutateAsync({
      command_id: crypto.randomUUID(),
      policy_id: policyId,
      provider_chat_id: providerChatId,
      variables,
    })
    dryRunResult.value = `${result.status}: ${result.rendered_preview_hash}`
  } catch (error) {
    dryRunError.value = error instanceof Error ? error.message : 'Telegram policy dry-run failed.'
  }
}

async function triggerProviderSearch(): Promise<void> {
  const accountId = selectedAccountId.value
  const query = providerSearchQuery.value.trim()
  if (!accountId || !query) return
  providerSearchStatus.value = ''
  providerSearchError.value = ''
  try {
    const result = await providerSearchMutation.mutateAsync({ account_id: accountId, q: query, limit: 50 })
    providerSearchStatus.value = `${result.status}: provider refresh requested`
  } catch (error) {
    providerSearchError.value = error instanceof Error ? error.message : 'Telegram provider search trigger failed.'
  }
}
</script>

<template>
  <aside class="telegram-runtime-panel" aria-label="Telegram runtime controls">
    <header class="telegram-runtime-panel__header">
      <Icon icon="tabler:brand-telegram" size="1.15rem" />
      <div>
        <strong>Telegram runtime</strong>
        <span>Provider controls and durable command queue</span>
      </div>
      <Button size="sm" variant="ghost" icon="tabler:refresh" :disabled="surface.isRuntimeBusy.value" @click="surface.refreshRuntime" />
    </header>

    <label class="telegram-runtime-panel__field">
      <span>Account</span>
      <select v-model="surface.selectedAccountId.value">
        <option v-for="account in surface.accounts.value" :key="account.account_id" :value="account.account_id">
          {{ account.display_name }} · {{ account.lifecycle_state }}
        </option>
      </select>
    </label>

    <section class="telegram-runtime-panel__section">
      <div class="telegram-runtime-panel__section-heading">
        <h3>Account setup</h3>
        <Button size="sm" variant="outline" @click="showSetup = !showSetup">{{ showSetup ? 'Hide' : 'Add account' }}</Button>
      </div>
      <form v-if="showSetup" class="telegram-runtime-panel__setup" @submit.prevent="setupAccount">
        <input v-model="setupForm.accountId" autocomplete="off" placeholder="Local account id" required />
        <input v-model="setupForm.displayName" autocomplete="off" placeholder="Display name" required />
        <input v-model="setupForm.externalAccountId" autocomplete="off" placeholder="Telegram user or bot id" required />
        <select v-model="setupForm.providerKind"><option value="telegram_user">User</option><option value="telegram_bot">Bot</option></select>
        <input v-model="setupForm.apiId" inputmode="numeric" placeholder="Telegram API id (optional)" />
        <input v-model="setupForm.apiHash" type="password" autocomplete="new-password" placeholder="Telegram API hash" />
        <input v-model="setupForm.botToken" type="password" autocomplete="new-password" placeholder="Bot token" />
        <input v-model="setupForm.sessionEncryptionKey" type="password" autocomplete="new-password" placeholder="Session encryption key" />
        <input v-model="setupForm.tdlibDataPath" autocomplete="off" placeholder="TDLib data path (optional)" />
        <label><input v-model="setupForm.qrAuthorized" type="checkbox" /> QR authorization completed</label>
        <Button size="sm" type="submit" :loading="setupAccountMutation.isPending.value">Save account</Button>
      </form>
      <p v-if="setupStatus" class="telegram-runtime-panel__success">{{ setupStatus }}</p>
      <p v-if="setupError" class="telegram-runtime-panel__error">{{ setupError }}</p>
    </section>

    <section v-if="surface.runtimeStatusQuery.data.value" class="telegram-runtime-panel__section">
      <h3>Runtime status</h3>
      <p>{{ surface.runtimeStatusQuery.data.value.status }} · {{ surface.runtimeStatusQuery.data.value.runtime_kind }}</p>
      <p v-if="surface.runtimeStatusQuery.data.value.last_error">{{ surface.runtimeStatusQuery.data.value.last_error }}</p>
      <div class="telegram-runtime-panel__actions">
        <Button size="sm" variant="outline" :disabled="surface.isRuntimeBusy.value" @click="surface.setTelegramRuntime('start')">Start</Button>
        <Button size="sm" variant="outline" :disabled="surface.isRuntimeBusy.value" @click="surface.setTelegramRuntime('restart')">Restart</Button>
        <Button size="sm" variant="outline" :disabled="surface.isRuntimeBusy.value" @click="surface.setTelegramRuntime('stop')">Stop</Button>
        <Button size="sm" variant="outline" :disabled="syncChatsMutation.isPending.value" @click="syncChats">Sync chats</Button>
      </div>
      <p v-if="surface.actionMessage.value" class="telegram-runtime-panel__success">{{ surface.actionMessage.value }}</p>
      <p v-if="surface.actionError.value" class="telegram-runtime-panel__error">{{ surface.actionError.value }}</p>
    </section>

    <section class="telegram-runtime-panel__section">
      <h3>Account lifecycle</h3>
      <p>{{ surface.selectedAccount.value?.lifecycle_state ?? 'No selected account' }}</p>
      <div class="telegram-runtime-panel__actions">
        <Button size="sm" variant="outline" :disabled="logoutAccountMutation.isPending.value" @click="logoutAccount">Log out</Button>
        <Button size="sm" variant="destructive" :disabled="removeAccountMutation.isPending.value" @click="removeAccount">Remove account</Button>
      </div>
      <p>Use Settings → Accounts to start or resume the QR connection wizard.</p>
    </section>

    <section class="telegram-runtime-panel__section">
      <h3>Capabilities</h3>
      <p v-if="!accountCapabilitiesQuery.data.value?.capabilities.length">No account capability record.</p>
      <p v-for="capability in accountCapabilitiesQuery.data.value?.capabilities ?? []" :key="capability.operation">
        {{ capability.operation }} · {{ capability.status }}
      </p>
    </section>

    <section class="telegram-runtime-panel__section">
      <h3>Folders</h3>
      <p v-if="!foldersQuery.data.value?.length">No provider folders.</p>
      <p v-for="folder in foldersQuery.data.value ?? []" :key="folder.id">{{ folder.label }} · {{ folder.count }}</p>
    </section>

    <section class="telegram-runtime-panel__section">
      <h3>Command queue</h3>
      <p v-if="!commandsQuery.data.value?.length">No queued commands.</p>
      <div v-for="command in commandsQuery.data.value ?? []" :key="command.command_id" class="telegram-runtime-panel__command">
        <span>{{ command.command_kind }} · {{ command.status }}</span>
        <Button v-if="command.status === 'failed' || command.status === 'dead_letter'" size="sm" variant="ghost" :disabled="retryCommandMutation.isPending.value" @click="retryCommand(command.command_id)">Retry</Button>
      </div>
    </section>

    <section class="telegram-runtime-panel__section">
      <h3>Provider refresh search</h3>
      <input v-model="providerSearchQuery" type="search" placeholder="Query provider, then refresh projections" />
      <Button size="sm" variant="outline" @click="triggerProviderSearch">Request provider search</Button>
      <p v-if="providerSearchStatus" class="telegram-runtime-panel__success">{{ providerSearchStatus }}</p>
      <p v-if="providerSearchError" class="telegram-runtime-panel__error">{{ providerSearchError }}</p>
    </section>

    <section class="telegram-runtime-panel__section">
      <h3>Calls</h3>
      <button v-for="call in callsQuery.data.value ?? []" :key="call.call_id" class="telegram-runtime-panel__call" type="button" @click="selectedCallId = call.call_id">
        {{ call.status }} · {{ call.occurred_at ?? call.call_id }}
      </button>
      <p v-if="transcriptQuery.data.value">{{ transcriptQuery.data.value.transcript_status }} · {{ transcriptQuery.data.value.transcript_text }}</p>
    </section>

    <section class="telegram-runtime-panel__section">
      <h3>Policy dry-run</h3>
      <select v-model="selectedPolicyId">
        <option value="">Select a policy</option>
        <option v-for="policy in policiesQuery.data.value ?? []" :key="policy.policy_id" :value="policy.policy_id">{{ policy.name }}</option>
      </select>
      <input v-model="dryRunChatId" type="text" placeholder="Provider chat id" />
      <textarea v-model="dryRunVariables" rows="2" aria-label="Template variables as JSON" />
      <Button size="sm" variant="outline" :disabled="dryRunMutation.isPending.value" @click="runDryRun">Run dry-run</Button>
      <p v-if="dryRunResult" class="telegram-runtime-panel__success">{{ dryRunResult }}</p>
      <p v-if="dryRunError" class="telegram-runtime-panel__error">{{ dryRunError }}</p>
    </section>
  </aside>
</template>
