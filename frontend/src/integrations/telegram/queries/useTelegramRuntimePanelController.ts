import { computed, ref } from 'vue'
import { useTelegramAutomationPoliciesQuery, useTelegramSendDryRunMutation } from './useTelegramAutomationQuery'
import { useTelegramCommandRetryMutation, useTelegramCommandsQuery } from './useTelegramLifecycleQuery'
import { useLogoutTelegramAccountMutation, useRemoveTelegramAccountMutation, useSetupTelegramAccountMutation, useSyncTelegramChatsMutation } from './useTelegramMutations'
import { useTelegramRuntimePanelSurface } from './useTelegramRuntimePanelSurface'
import { useTelegramAccountCapabilitiesQuery, useTelegramCallTranscriptQuery, useTelegramCallsQuery, useTelegramFoldersQuery, useTelegramProviderSearchMutation } from './useTelegramQuery'
import {
  buildTelegramAccountSetupRequest,
  buildTelegramDryRunRequest,
  canRunTelegramDryRun,
  canTriggerTelegramProviderSearch,
  buildTelegramProviderSearchRequest,
  buildTelegramSyncChatsRequest,
  TELEGRAM_RUNTIME_CALLS_PAGE_SIZE,
  TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE,
  createTelegramRuntimeSetupForm,
  parseTelegramDryRunVariables,
  telegramRuntimeCommandId,
  type TelegramRuntimeSetupForm,
} from './telegramRuntimePanelActions'

export function useTelegramRuntimePanelController() {
  const surface = useTelegramRuntimePanelSurface()
  const selectedAccountId = computed(() => surface.selectedAccount.value?.account_id ?? null)
  const accountCapabilitiesQuery = useTelegramAccountCapabilitiesQuery(selectedAccountId)
  const commandsQuery = useTelegramCommandsQuery(selectedAccountId, TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE)
  const foldersQuery = useTelegramFoldersQuery(() => selectedAccountId.value ?? undefined)
  const callsQuery = useTelegramCallsQuery(() => selectedAccountId.value ?? undefined, TELEGRAM_RUNTIME_CALLS_PAGE_SIZE)
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
  const setupForm = ref<TelegramRuntimeSetupForm>(createTelegramRuntimeSetupForm())
  const transcriptQuery = useTelegramCallTranscriptQuery(selectedCallId)

  async function syncChats(): Promise<void> {
    const accountId = selectedAccountId.value
    if (!accountId) return
    await syncChatsMutation.mutateAsync(buildTelegramSyncChatsRequest(accountId))
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
    const result = buildTelegramAccountSetupRequest(setupForm.value)
    if ('error' in result) {
      if (result.error === 'api_id_invalid') setupError.value = 'Telegram API id must be a number.'
      return
    }

    setupError.value = ''
    setupStatus.value = ''
    try {
      await setupAccountMutation.mutateAsync(result.request)
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
    if (!canRunTelegramDryRun(policyId, providerChatId)) return

    dryRunError.value = ''
    dryRunResult.value = ''
    try {
      const variables = parseTelegramDryRunVariables(dryRunVariables.value)
      const result = await dryRunMutation.mutateAsync(buildTelegramDryRunRequest(
        policyId,
        providerChatId,
        variables,
        telegramRuntimeCommandId(),
      ))
      dryRunResult.value = `${result.status}: ${result.rendered_preview_hash}`
    } catch (error) {
      dryRunError.value = error instanceof Error ? error.message : 'Telegram policy dry-run failed.'
    }
  }

  async function triggerProviderSearch(): Promise<void> {
    const accountId = selectedAccountId.value
    const query = providerSearchQuery.value.trim()
    if (!canTriggerTelegramProviderSearch(accountId, query)) return
    providerSearchStatus.value = ''
    providerSearchError.value = ''
    try {
      const result = await providerSearchMutation.mutateAsync(
        buildTelegramProviderSearchRequest(accountId, query)
      )
      providerSearchStatus.value = `${result.status}: provider refresh requested`
    } catch (error) {
      providerSearchError.value = error instanceof Error ? error.message : 'Telegram provider search trigger failed.'
    }
  }

  return {
    surface,
    selectedAccountId,
    accountCapabilitiesQuery,
    commandsQuery,
    foldersQuery,
    callsQuery,
    syncChatsMutation,
    logoutAccountMutation,
    removeAccountMutation,
    setupAccountMutation,
    retryCommandMutation,
    policiesQuery,
    dryRunMutation,
    selectedCallId,
    selectedPolicyId,
    dryRunChatId,
    dryRunVariables,
    dryRunError,
    dryRunResult,
    providerSearchQuery,
    providerSearchStatus,
    providerSearchError,
    providerSearchMutation,
    showSetup,
    setupError,
    setupStatus,
    setupForm,
    transcriptQuery,
    syncChats,
    logoutAccount,
    removeAccount,
    setupAccount,
    retryCommand,
    runDryRun,
    triggerProviderSearch,
  }
}
