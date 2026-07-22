import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import type {
  WhatsAppWebCompanionManifest,
  WhatsappAccountSummary,
  WhatsappWebSession,
} from '../../../shared/communications/types/whatsapp'
import { useWhatsappStore } from '../stores/whatsapp'
import {
  useRelinkWhatsappRuntimeMutation,
  useRemoveWhatsappRuntimeMutation,
  useRotateWhatsappRuntimeMutation,
  useRevokeWhatsappRuntimeMutation,
  useDeadLetterWhatsappProviderCommandMutation,
  usePublishWhatsappStatusMutation,
  useRetryWhatsappProviderCommandMutation,
  useStartHiddenWhatsappWebviewMutation,
  useStartWhatsappRuntimeMutation,
  useStopWhatsappRuntimeMutation,
  useWhatsappAccountsQuery,
  useWhatsappAccountCapabilitiesQuery,
  useWhatsappCapabilitiesQuery,
  useWhatsappProviderCommandsQuery,
  useWhatsappRuntimeHealthQuery,
  useWhatsappRuntimeStatusQuery,
  useWhatsappSessionsQuery,
  WHATSAPP_RUNTIME_COMMANDS_PAGE_SIZE,
  WHATSAPP_RUNTIME_SESSIONS_PAGE_SIZE,
  WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE,
  useWhatsappSyncChatsQuery,
  useWhatsappSyncCallsQuery,
  useWhatsappSyncContactsQuery,
  useWhatsappSyncHistoryQuery,
  useWhatsappSyncMediaQuery,
  useWhatsappSyncMembersQuery,
  useWhatsappSyncPresenceQuery,
  useWhatsappSyncStatusesQuery,
} from './useWhatsappQuery'

export function useWhatsappRuntimePanelSurface() {
  const { t } = useI18n()
  const realtimeStatus = useRealtimeStatusStore()
  const store = useWhatsappStore()

  const includeRemovedAccounts = ref(false)
  const selectedAccountIdState = ref<string | null>(null)
  const selectedSyncChatId = ref<string | null>(null)
  const isCompanionOpening = ref(false)
  const companionOpenManifest = ref<WhatsAppWebCompanionManifest | null>(null)
  const statusPublishText = ref('')

  const accountsQuery = useWhatsappAccountsQuery(includeRemovedAccounts)
  const capabilitiesQuery = useWhatsappCapabilitiesQuery()
  const accounts = computed(() => accountsQuery.data.value ?? [])
  const capabilities = computed(() => capabilitiesQuery.data.value ?? null)
  const selectedAccountId = computed(() =>
    selectedAccountIdState.value
    ?? store.selectedWhatsappSession?.account_id
    ?? accounts.value.find((account) => account.lifecycle_state !== 'removed')?.account_id
    ?? accounts.value[0]?.account_id
    ?? null
  )
  const sessionsQuery = useWhatsappSessionsQuery(selectedAccountId, WHATSAPP_RUNTIME_SESSIONS_PAGE_SIZE)
  const sessions = computed(() => sessionsQuery.data.value ?? [])
  const selectedAccountSummary = computed<WhatsappAccountSummary | null>(() =>
    accounts.value.find((account) => account.account_id === selectedAccountId.value) ?? null
  )
  const accountCapabilitiesQuery = useWhatsappAccountCapabilitiesQuery(selectedAccountId)
  const runtimeStatusQuery = useWhatsappRuntimeStatusQuery(selectedAccountId)
  const runtimeHealthQuery = useWhatsappRuntimeHealthQuery(selectedAccountId)
  const providerCommandsQuery = useWhatsappProviderCommandsQuery(selectedAccountId, WHATSAPP_RUNTIME_COMMANDS_PAGE_SIZE)
  const chatsSyncQuery = useWhatsappSyncChatsQuery(selectedAccountId, WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE)
  const chatItems = computed(() => chatsSyncQuery.data.value ?? [])
  const selectedSyncChatIdResolved = computed(() =>
    selectedSyncChatId.value
    ?? chatItems.value[0]?.provider_chat_id
    ?? null
  )
  const historySyncQuery = useWhatsappSyncHistoryQuery(
    selectedAccountId,
    selectedSyncChatIdResolved,
    WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE
  )
  const membersSyncQuery = useWhatsappSyncMembersQuery(
    selectedAccountId,
    selectedSyncChatIdResolved,
    WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE
  )
  const statusesSyncQuery = useWhatsappSyncStatusesQuery(selectedAccountId, WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE)
  const presenceSyncQuery = useWhatsappSyncPresenceQuery(
    selectedAccountId,
    selectedSyncChatIdResolved,
    WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE
  )
  const callsSyncQuery = useWhatsappSyncCallsQuery(
    selectedAccountId,
    selectedSyncChatIdResolved,
    WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE
  )
  const contactsSyncQuery = useWhatsappSyncContactsQuery(selectedAccountId, WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE)
  const mediaSyncQuery = useWhatsappSyncMediaQuery(
    selectedAccountId,
    selectedSyncChatIdResolved,
    WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE
  )

  const startRuntimeMutation = useStartWhatsappRuntimeMutation()
  const stopRuntimeMutation = useStopWhatsappRuntimeMutation()
  const revokeRuntimeMutation = useRevokeWhatsappRuntimeMutation()
  const relinkRuntimeMutation = useRelinkWhatsappRuntimeMutation()
  const rotateRuntimeMutation = useRotateWhatsappRuntimeMutation()
  const removeRuntimeMutation = useRemoveWhatsappRuntimeMutation()
  const retryCommandMutation = useRetryWhatsappProviderCommandMutation()
  const deadLetterCommandMutation = useDeadLetterWhatsappProviderCommandMutation()
  const publishStatusMutation = usePublishWhatsappStatusMutation()
  const hiddenWebviewMutation = useStartHiddenWhatsappWebviewMutation()

  const runtimeCapabilities = computed(() => accountCapabilitiesQuery.data.value ?? capabilities.value)
  const runtimeStatus = computed(() => runtimeStatusQuery.data.value ?? null)
  const runtimeHealth = computed(() => runtimeHealthQuery.data.value ?? null)
  const runtimeHealthChecks = computed(() => Object.entries(runtimeHealth.value?.checks ?? {}))
  const providerCommands = computed(() => providerCommandsQuery.data.value ?? [])
  const historyItems = computed(() => historySyncQuery.data.value ?? [])
  const memberItems = computed(() => membersSyncQuery.data.value ?? [])
  const statusItems = computed(() => statusesSyncQuery.data.value ?? [])
  const presenceItems = computed(() => presenceSyncQuery.data.value ?? [])
  const callItems = computed(() => callsSyncQuery.data.value ?? [])
  const contactItems = computed(() => contactsSyncQuery.data.value ?? [])
  const mediaItems = computed(() => mediaSyncQuery.data.value ?? [])
  const selectedRuntimeProviderShape = computed(
    () =>
      runtimeStatus.value?.provider_shape
      ?? runtimeCapabilities.value?.account_scope?.provider_shape
      ?? selectedAccountSummary.value?.provider_shape
      ?? null
  )
  const canOpenWebCompanion = computed(
    () => selectedRuntimeProviderShape.value === 'whatsapp_web_companion'
  )
  const isRuntimeBusy = computed(() =>
    isCompanionOpening.value ||
    startRuntimeMutation.isPending.value ||
    stopRuntimeMutation.isPending.value ||
    revokeRuntimeMutation.isPending.value ||
    relinkRuntimeMutation.isPending.value ||
    rotateRuntimeMutation.isPending.value ||
    removeRuntimeMutation.isPending.value ||
    retryCommandMutation.isPending.value ||
    deadLetterCommandMutation.isPending.value ||
    publishStatusMutation.isPending.value ||
    hiddenWebviewMutation.isPending.value
  )

  watch(accounts, (nextAccounts) => {
    if (!nextAccounts.length) {
      selectedAccountIdState.value = null
      return
    }
    const current = selectedAccountIdState.value
    if (current && nextAccounts.some((account) => account.account_id === current)) {
      return
    }
    selectedAccountIdState.value =
      nextAccounts.find((account) => account.lifecycle_state !== 'removed')?.account_id
      ?? nextAccounts[0]?.account_id
      ?? null
  }, { immediate: true })

  watch(chatItems, (items) => {
    if (!items.length) {
      selectedSyncChatId.value = null
      return
    }
    if (selectedSyncChatId.value && items.some((item) => item.provider_chat_id === selectedSyncChatId.value)) {
      return
    }
    selectedSyncChatId.value = items[0]?.provider_chat_id ?? null
  }, { immediate: true })

  watch(
    [sessions, capabilities],
    ([nextSessions, nextCapabilities]) => {
      const selectedSessionId = nextSessions.some((session) => session.session_id === store.selectedWhatsappSessionId)
        ? store.selectedWhatsappSessionId
        : nextSessions[0]?.session_id ?? ''
      store.setWhatsappData({
        sessions: nextSessions,
        capabilities: nextCapabilities,
        selectedSessionId,
        error: '',
      })
    },
    { immediate: true }
  )

  async function refreshRuntime() {
    await Promise.all([
      accountsQuery.refetch(),
      capabilitiesQuery.refetch(),
      accountCapabilitiesQuery.refetch(),
      sessionsQuery.refetch(),
      runtimeStatusQuery.refetch(),
      runtimeHealthQuery.refetch(),
      providerCommandsQuery.refetch(),
      chatsSyncQuery.refetch(),
      historySyncQuery.refetch(),
      membersSyncQuery.refetch(),
      statusesSyncQuery.refetch(),
      presenceSyncQuery.refetch(),
      callsSyncQuery.refetch(),
      contactsSyncQuery.refetch(),
      mediaSyncQuery.refetch(),
    ])
  }

  function selectWhatsappAccount(accountId: string) {
    selectedAccountIdState.value = accountId
    const session = sessions.value.find((item) => item.account_id === accountId)
    if (session) {
      store.selectWhatsappSession(session)
    }
  }

  function selectWhatsappSession(session: WhatsappWebSession) {
    selectedAccountIdState.value = session.account_id
    store.selectWhatsappSession(session)
  }

  function requireAccountId(): string | null {
    const accountId = selectedAccountId.value?.trim() ?? ''
    if (!accountId) {
      store.setWhatsappError('Select a WhatsApp account first')
      return null
    }
    return accountId
  }

  async function setRuntimeState(action: 'start' | 'stop' | 'revoke' | 'relink' | 'rotate' | 'remove') {
    const accountId = requireAccountId()
    if (!accountId || isRuntimeBusy.value) return
    store.setWhatsappActionMessage('')
    store.setWhatsappError('')
    try {
      if (action === 'start') {
        const status = await startRuntimeMutation.mutateAsync({ account_id: accountId })
        store.setWhatsappActionMessage(`WhatsApp runtime ${status.status}`)
      } else if (action === 'stop') {
        const status = await stopRuntimeMutation.mutateAsync({ account_id: accountId })
        store.setWhatsappActionMessage(`WhatsApp runtime ${status.status}`)
      } else if (action === 'revoke') {
        const status = await revokeRuntimeMutation.mutateAsync({ account_id: accountId })
        store.setWhatsappActionMessage(`WhatsApp runtime ${status.status}`)
      } else if (action === 'relink') {
        const status = await relinkRuntimeMutation.mutateAsync({ account_id: accountId })
        store.setWhatsappActionMessage(`WhatsApp runtime ${status.status}`)
      } else if (action === 'rotate') {
        const status = await rotateRuntimeMutation.mutateAsync({ account_id: accountId })
        store.setWhatsappActionMessage(`WhatsApp runtime ${status.status}`)
      } else if (action === 'remove') {
        const response = await removeRuntimeMutation.mutateAsync({ account_id: accountId })
        store.setWhatsappActionMessage(
          response.unbound_secret_refs.length
            ? `WhatsApp runtime removed and unbound ${response.unbound_secret_refs.length} secret reference(s)`
            : 'WhatsApp runtime removed'
        )
      }
      await refreshRuntime()
    } catch (error) {
      store.setWhatsappError(error instanceof Error ? error.message : String(error))
    }
  }

  async function startHiddenWebview() {
    const accountId = requireAccountId()
    if (!accountId || isRuntimeBusy.value) return
    if (!canOpenWebCompanion.value) {
      store.setWhatsappError('Hidden WebView runtime is available only for whatsapp_web_companion accounts')
      return
    }
    isCompanionOpening.value = true
    store.setWhatsappActionMessage('')
    store.setWhatsappError('')
    try {
      const manifest = await hiddenWebviewMutation.mutateAsync({ account_id: accountId })
      companionOpenManifest.value = manifest
      store.setWhatsappActionMessage(
        manifest.opened_window
          ? `Hidden WhatsApp WebView started for ${manifest.account_id}`
          : `Hidden WhatsApp WebView already running for ${manifest.account_id}`
      )
      await refreshRuntime()
    } catch (error) {
      store.setWhatsappError(error instanceof Error ? error.message : String(error))
    } finally {
      isCompanionOpening.value = false
    }
  }

  async function retryCommand(commandId: string) {
    if (isRuntimeBusy.value) return
    store.setWhatsappActionMessage('')
    store.setWhatsappError('')
    try {
      const command = await retryCommandMutation.mutateAsync({ command_id: commandId })
      store.setWhatsappActionMessage(`WhatsApp command ${command.command_kind} moved to ${command.status}`)
      await refreshRuntime()
    } catch (error) {
      store.setWhatsappError(error instanceof Error ? error.message : String(error))
    }
  }

  async function deadLetterCommand(commandId: string) {
    if (isRuntimeBusy.value) return
    store.setWhatsappActionMessage('')
    store.setWhatsappError('')
    try {
      const command = await deadLetterCommandMutation.mutateAsync({
        command_id: commandId,
        reason: 'operator_dead_letter_from_runtime_panel',
      })
      store.setWhatsappActionMessage(`WhatsApp command ${command.command_kind} moved to ${command.status}`)
      await refreshRuntime()
    } catch (error) {
      store.setWhatsappError(error instanceof Error ? error.message : String(error))
    }
  }

  async function publishStatus() {
    const accountId = requireAccountId()
    if (!accountId || isRuntimeBusy.value) return
    const text = statusPublishText.value.trim()
    if (!text) {
      store.setWhatsappError('Status text is required')
      return
    }
    store.setWhatsappActionMessage('')
    store.setWhatsappError('')
    try {
      const command = await publishStatusMutation.mutateAsync({
        account_id: accountId,
        idempotency_key: `status:${crypto.randomUUID()}`,
        text,
      })
      store.setWhatsappActionMessage(`WhatsApp status publish ${command.status}`)
      statusPublishText.value = ''
      await refreshRuntime()
    } catch (error) {
      store.setWhatsappError(error instanceof Error ? error.message : String(error))
    }
  }

  return {
    accounts,
    accountsQuery,
    callItems,
    canOpenWebCompanion,
    chatItems,
    capabilities,
    capabilitiesQuery,
    companionOpenManifest,
    contactItems,
    historyItems,
    includeRemovedAccounts,
    isCompanionOpening,
    isRuntimeBusy,
    mediaItems,
    memberItems,
    startHiddenWebview,
    presenceItems,
    providerCommands,
    publishStatus,
    realtimeStatus,
    refreshRuntime,
    retryCommand,
    runtimeCapabilities,
    runtimeHealth,
    runtimeHealthChecks,
    runtimeStatus,
    selectedAccountId,
    selectedAccountSummary,
    selectedSyncChatId,
    selectedSyncChatIdResolved,
    selectWhatsappAccount,
    selectWhatsappSession,
    sessions,
    sessionsQuery,
    setRuntimeState,
    statusItems,
    statusPublishText,
    store,
    t,
    deadLetterCommand,
  }
}

export type WhatsappRuntimePanelSurface = ReturnType<typeof useWhatsappRuntimePanelSurface>
