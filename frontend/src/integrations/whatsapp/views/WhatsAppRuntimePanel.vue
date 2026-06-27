<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import type {
  WhatsAppWebCompanionManifest,
  WhatsappAccountSummary,
  WhatsappProviderShape,
  WhatsappWebProviderKind,
  WhatsappWebSession,
} from '../../../shared/communications/types/whatsapp'
import WhatsAppSessionList from '../components/WhatsAppSessionList.vue'
import WhatsAppRail from '../components/WhatsAppRail.vue'
import WhatsAppStatusMessages from '../components/WhatsAppStatusMessages.vue'
import WhatsAppRuntimeAccountList from '../components/WhatsAppRuntimeAccountList.vue'
import WhatsAppRuntimeAccountProvisioning from '../components/WhatsAppRuntimeAccountProvisioning.vue'
import WhatsAppRuntimeCapabilities from '../components/WhatsAppRuntimeCapabilities.vue'
import WhatsAppRuntimeCommandAudit from '../components/WhatsAppRuntimeCommandAudit.vue'
import WhatsAppRuntimeControl from '../components/WhatsAppRuntimeControl.vue'
import WhatsAppRuntimeLinking from '../components/WhatsAppRuntimeLinking.vue'
import WhatsAppRuntimeSnapshots from '../components/WhatsAppRuntimeSnapshots.vue'
import { useWhatsappStore } from '../stores/whatsapp'
import {
  ingestWhatsappWebMessageFixture,
  setupWhatsappWebFixture,
} from '../api/whatsapp'
import { openWhatsappWebCompanion } from '../api/whatsappCompanion'
import {
  useRelinkWhatsappRuntimeMutation,
  useRemoveWhatsappRuntimeMutation,
  useRotateWhatsappRuntimeMutation,
  useRevokeWhatsappRuntimeMutation,
  useDeadLetterWhatsappProviderCommandMutation,
  usePublishWhatsappStatusMutation,
  useRetryWhatsappProviderCommandMutation,
  useSetupWhatsappLiveAccountMutation,
  useStartWhatsappPairCodeLinkMutation,
  useStartWhatsappQrLinkMutation,
  useStartWhatsappRuntimeMutation,
  useStopWhatsappRuntimeMutation,
  useWhatsappAccountsQuery,
  useWhatsappAccountCapabilitiesQuery,
  useWhatsappCapabilitiesQuery,
  useWhatsappProviderCommandsQuery,
  useWhatsappRuntimeHealthQuery,
  useWhatsappRuntimeStatusQuery,
  useWhatsappSessionsQuery,
  useWhatsappSyncChatsQuery,
  useWhatsappSyncCallsQuery,
  useWhatsappSyncContactsQuery,
  useWhatsappSyncHistoryQuery,
  useWhatsappSyncMediaQuery,
  useWhatsappSyncMembersQuery,
  useWhatsappSyncPresenceQuery,
  useWhatsappSyncStatusesQuery,
} from '../queries/useWhatsappQuery'
const { t } = useI18n()
const realtimeStatus = useRealtimeStatusStore()
const store = useWhatsappStore()
const includeRemovedAccounts = ref(false)
const selectedAccountIdState = ref<string | null>(null)
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
const sessionsQuery = useWhatsappSessionsQuery(selectedAccountId, 100)
const sessions = computed(() => sessionsQuery.data.value ?? [])
const selectedAccountSummary = computed<WhatsappAccountSummary | null>(() =>
  accounts.value.find((account) => account.account_id === selectedAccountId.value) ?? null
)
const accountCapabilitiesQuery = useWhatsappAccountCapabilitiesQuery(selectedAccountId)
const runtimeStatusQuery = useWhatsappRuntimeStatusQuery(selectedAccountId)
const runtimeHealthQuery = useWhatsappRuntimeHealthQuery(selectedAccountId)
const providerCommandsQuery = useWhatsappProviderCommandsQuery(selectedAccountId, 25)
const selectedSyncChatId = ref<string | null>(null)
const chatsSyncQuery = useWhatsappSyncChatsQuery(selectedAccountId, 8)
const chatItems = computed(() => chatsSyncQuery.data.value ?? [])
const selectedSyncChatIdResolved = computed(() =>
  selectedSyncChatId.value
  ?? chatItems.value[0]?.provider_chat_id
  ?? null
)
const historySyncQuery = useWhatsappSyncHistoryQuery(
  selectedAccountId,
  selectedSyncChatIdResolved,
  8
)
const membersSyncQuery = useWhatsappSyncMembersQuery(
  selectedAccountId,
  selectedSyncChatIdResolved,
  8
)
const statusesSyncQuery = useWhatsappSyncStatusesQuery(selectedAccountId, 8)
const presenceSyncQuery = useWhatsappSyncPresenceQuery(selectedAccountId, selectedSyncChatIdResolved, 8)
const callsSyncQuery = useWhatsappSyncCallsQuery(selectedAccountId, selectedSyncChatIdResolved, 8)
const contactsSyncQuery = useWhatsappSyncContactsQuery(selectedAccountId, 8)
const mediaSyncQuery = useWhatsappSyncMediaQuery(selectedAccountId, selectedSyncChatIdResolved, 8)
const startRuntimeMutation = useStartWhatsappRuntimeMutation()
const stopRuntimeMutation = useStopWhatsappRuntimeMutation()
const revokeRuntimeMutation = useRevokeWhatsappRuntimeMutation()
const relinkRuntimeMutation = useRelinkWhatsappRuntimeMutation()
const rotateRuntimeMutation = useRotateWhatsappRuntimeMutation()
const removeRuntimeMutation = useRemoveWhatsappRuntimeMutation()
const retryCommandMutation = useRetryWhatsappProviderCommandMutation()
const deadLetterCommandMutation = useDeadLetterWhatsappProviderCommandMutation()
const publishStatusMutation = usePublishWhatsappStatusMutation()
const setupLiveAccountMutation = useSetupWhatsappLiveAccountMutation()
const qrLinkMutation = useStartWhatsappQrLinkMutation()
const pairCodeMutation = useStartWhatsappPairCodeLinkMutation()
const pairCodePhoneNumber = ref('')
const isCompanionOpening = ref(false)
const companionOpenManifest = ref<WhatsAppWebCompanionManifest | null>(null)
const liveAccountShape = ref<WhatsappProviderShape>('whatsapp_web_companion')
const liveAccountId = ref('whatsapp-live-primary')
const liveAccountDisplayName = ref('WhatsApp Live')
const liveAccountExternalId = ref('whatsapp-live-primary')
const liveAccountDeviceName = ref('Hermes WhatsApp companion')
const liveAccountLocalStatePath = ref('docker/data/whatsapp/blocked/whatsapp-live-primary')

const runtimeCapabilities = computed(
  () => accountCapabilitiesQuery.data.value ?? capabilities.value
)
const runtimeStatus = computed(() => runtimeStatusQuery.data.value ?? null)
const runtimeHealth = computed(() => runtimeHealthQuery.data.value ?? null)
const runtimeHealthChecks = computed(() =>
  Object.entries(runtimeHealth.value?.checks ?? {})
)
const providerCommands = computed(() => providerCommandsQuery.data.value ?? [])
const historyItems = computed(() => historySyncQuery.data.value ?? [])
const memberItems = computed(() => membersSyncQuery.data.value ?? [])
const statusItems = computed(() => statusesSyncQuery.data.value ?? [])
const presenceItems = computed(() => presenceSyncQuery.data.value ?? [])
const callItems = computed(() => callsSyncQuery.data.value ?? [])
const contactItems = computed(() => contactsSyncQuery.data.value ?? [])
const mediaItems = computed(() => mediaSyncQuery.data.value ?? [])
const activeQrSession = computed(() => qrLinkMutation.data.value ?? null)
const activePairCodeSession = computed(() => pairCodeMutation.data.value ?? null)
const selectedProviderShapeMeta = computed(() =>
  capabilities.value?.provider_shapes.find((shape) => shape.provider_shape === liveAccountShape.value) ?? null
)
const liveAccountProviderKind = computed<WhatsappWebProviderKind>(() =>
  liveAccountShape.value === 'whatsapp_business_cloud'
    ? 'whatsapp_business_cloud'
    : 'whatsapp_web'
)
const liveAccountSessionMode = computed(() =>
  liveAccountShape.value === 'whatsapp_business_cloud' ? 'api_credentials' : 'device_session'
)
const liveAccountSupportsDeviceFields = computed(
  () => liveAccountShape.value !== 'whatsapp_business_cloud'
)
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
  setupLiveAccountMutation.isPending.value ||
  startRuntimeMutation.isPending.value ||
  stopRuntimeMutation.isPending.value ||
  revokeRuntimeMutation.isPending.value ||
  relinkRuntimeMutation.isPending.value ||
  rotateRuntimeMutation.isPending.value ||
  removeRuntimeMutation.isPending.value ||
  retryCommandMutation.isPending.value ||
  deadLetterCommandMutation.isPending.value ||
  publishStatusMutation.isPending.value ||
  qrLinkMutation.isPending.value ||
  pairCodeMutation.isPending.value
)
const statusPublishText = ref('')

watch(liveAccountShape, (shape) => {
  if (shape === 'whatsapp_business_cloud') {
    liveAccountDeviceName.value = ''
    liveAccountLocalStatePath.value = `docker/data/whatsapp/business-cloud/${liveAccountId.value.trim() || 'whatsapp-business-cloud'}`
    return
  }
  if (!liveAccountDeviceName.value.trim()) {
    liveAccountDeviceName.value =
      shape === 'whatsapp_native_md'
        ? 'Hermes WhatsApp native runtime'
        : 'Hermes WhatsApp companion'
  }
  liveAccountLocalStatePath.value = `docker/data/whatsapp/blocked/${liveAccountId.value.trim() || 'whatsapp-live-primary'}`
}, { immediate: true })

watch(liveAccountId, (accountId) => {
  const trimmed = accountId.trim()
  if (!trimmed) return
  liveAccountExternalId.value = trimmed
  liveAccountLocalStatePath.value = liveAccountShape.value === 'whatsapp_business_cloud'
    ? `docker/data/whatsapp/business-cloud/${trimmed}`
    : `docker/data/whatsapp/blocked/${trimmed}`
})

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

watch(selectedAccountId, (accountId) => {
  if (!accountId) return
  store.whatsappMessageForm = {
    ...store.whatsappMessageForm,
    account_id: accountId,
  }
})

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

async function createLiveAccount() {
  if (store.isWhatsappActionSubmitting) return
  if (
    !liveAccountId.value.trim() ||
    !liveAccountDisplayName.value.trim() ||
    !liveAccountExternalId.value.trim()
  ) {
    store.setWhatsappError('Account id, display name and external account id are required')
    return
  }
  store.setWhatsappActionSubmitting(true)
  store.setWhatsappActionMessage('')
  store.setWhatsappError('')
  try {
    const response = await setupLiveAccountMutation.mutateAsync({
      account_id: liveAccountId.value.trim(),
      provider_kind: liveAccountProviderKind.value,
      provider_shape: liveAccountShape.value,
      display_name: liveAccountDisplayName.value.trim(),
      external_account_id: liveAccountExternalId.value.trim(),
      ...(liveAccountSupportsDeviceFields.value && liveAccountDeviceName.value.trim()
        ? { device_name: liveAccountDeviceName.value.trim() }
        : {}),
      ...(liveAccountLocalStatePath.value.trim()
        ? { local_state_path: liveAccountLocalStatePath.value.trim() }
        : {}),
    })
    store.setWhatsappActionMessage(
      `WhatsApp account ${response.account_id} created for ${liveAccountShape.value}`
    )
    await refreshRuntime()
    selectedAccountIdState.value = response.account_id
    store.selectWhatsappSession(response.session)
  } catch (error) {
    store.setWhatsappError(error instanceof Error ? error.message : String(error))
  } finally {
    store.setWhatsappActionSubmitting(false)
  }
}

function selectWhatsappAccount(accountId: string) {
  selectedAccountIdState.value = accountId
  const session = sessions.value.find((item) => item.account_id === accountId)
  if (session) {
    store.selectWhatsappSession(session)
    return
  }
  store.whatsappMessageForm = {
    ...store.whatsappMessageForm,
    account_id: accountId,
  }
}

function selectWhatsappSession(session: WhatsappWebSession) {
  selectedAccountIdState.value = session.account_id
  store.selectWhatsappSession(session)
}

async function ingestFixtureMessage() {
  if (store.isWhatsappActionSubmitting) return
  store.setWhatsappActionSubmitting(true)
  store.setWhatsappActionMessage('')
  store.setWhatsappError('')
  try {
    const result = await ingestWhatsappWebMessageFixture({
      account_id: store.whatsappMessageForm.account_id,
      provider_chat_id: store.whatsappMessageForm.provider_chat_id,
      provider_message_id: store.whatsappMessageForm.provider_message_id,
      chat_title: store.whatsappMessageForm.chat_title,
      sender_id: store.whatsappMessageForm.sender_id,
      sender_display_name: store.whatsappMessageForm.sender_display_name,
      text: store.whatsappMessageForm.text,
      import_batch_id: store.whatsappMessageForm.import_batch_id,
      occurred_at: store.whatsappMessageForm.occurred_at,
      delivery_state: store.whatsappMessageForm.delivery_state as 'received' | 'sent' | 'send_dry_run' | 'send_blocked',
    })
    store.setWhatsappActionMessage(result.message)
    store.whatsappMessageForm = {
      ...store.whatsappMessageForm,
      provider_message_id: `wa-fixture-msg-${crypto.randomUUID()}`,
      occurred_at: new Date().toISOString(),
    }
  } catch (error) {
    store.setWhatsappError(error instanceof Error ? error.message : String(error))
  } finally {
    store.setWhatsappActionSubmitting(false)
  }
}

async function setupFixtureAccount() {
  if (store.isWhatsappActionSubmitting) return
  store.setWhatsappActionSubmitting(true)
  store.setWhatsappActionMessage('')
  store.setWhatsappError('')
  try {
    const result = await setupWhatsappWebFixture({
      account_id: store.whatsappMessageForm.account_id,
      display_name: 'WhatsApp Fixture',
      external_account_id: store.whatsappMessageForm.account_id,
      device_name: 'Local fixture device',
      local_state_path: 'docker/data/whatsapp/fixture',
    })
    if (result.error) {
      store.setWhatsappError(result.error)
    } else {
      store.setWhatsappActionMessage(result.message)
      await refreshRuntime()
    }
  } catch (error) {
    store.setWhatsappError(error instanceof Error ? error.message : String(error))
  } finally {
    store.setWhatsappActionSubmitting(false)
  }
}

function requireAccountId(): string | null {
  const accountId = selectedAccountId.value?.trim() ?? ''
  if (!accountId) {
    store.setWhatsappError('Select a WhatsApp account first')
    return null
  }
  return accountId
}

async function setRuntimeState(
  action: 'start' | 'stop' | 'revoke' | 'relink' | 'rotate' | 'remove' | 'qr' | 'pair_code'
) {
  const accountId = requireAccountId()
  if (!accountId || isRuntimeBusy.value) return
  if (action === 'pair_code' && !pairCodePhoneNumber.value.trim()) {
    store.setWhatsappError('Pair-code login requires a phone number')
    return
  }
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
    } else if (action === 'qr') {
      const response = await qrLinkMutation.mutateAsync({ account_id: accountId })
      store.setWhatsappActionMessage(`WhatsApp QR linking ${response.status}`)
    } else {
      const response = await pairCodeMutation.mutateAsync({
        account_id: accountId,
        phone_number: pairCodePhoneNumber.value.trim(),
      })
      store.setWhatsappActionMessage(`WhatsApp pair-code linking ${response.status}`)
    }
    await refreshRuntime()
  } catch (error) {
    store.setWhatsappError(error instanceof Error ? error.message : String(error))
  }
}

async function openVisibleWebCompanion() {
  const accountId = requireAccountId()
  if (!accountId || isRuntimeBusy.value) return
  if (!canOpenWebCompanion.value) {
    store.setWhatsappError('Visible WebView companion is available only for whatsapp_web_companion accounts')
    return
  }
  isCompanionOpening.value = true
  store.setWhatsappActionMessage('')
  store.setWhatsappError('')
  try {
    const manifest = await openWhatsappWebCompanion(accountId)
    companionOpenManifest.value = manifest
    store.setWhatsappActionMessage(
      manifest.opened_window
        ? `Visible WhatsApp companion opened for ${manifest.account_id}`
        : `Visible WhatsApp companion focused for ${manifest.account_id}`
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
      reason: 'manual_dead_letter_from_runtime_panel',
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
</script>

<template>
  <section class="whatsapp-runtime-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-whatsapp" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('WhatsApp Runtime') }}</h1>
          <p>{{ t('Provider sessions, capabilities and fixture controls') }}</p>
        </div>
      </div>
      <button
        type="button"
        class="primary-button"
        :disabled="accountsQuery.isFetching.value || sessionsQuery.isFetching.value"
        @click="refreshRuntime"
      >
        <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Refresh') }}
      </button>
    </header>

    <p
      class="whatsapp-realtime-state"
      :class="realtimeStatus.realtimeStatusTone"
      :title="realtimeStatus.realtimeStatusDetail"
    >
      {{ t('Realtime') }}: {{ realtimeStatus.realtimeStatusLabel }}
    </p>

    <div class="metric-grid">
      <article class="metric-card">
        <span>{{ t('Accounts') }}</span>
        <strong>{{ accounts.length }}</strong>
        <small>{{ accounts.filter((item) => item.lifecycle_state === 'removed').length }} {{ t('removed') }}</small>
      </article>
      <article class="metric-card">
        <span>{{ t('Sessions') }}</span>
        <strong>{{ sessions.length }}</strong>
        <small>{{ store.selectedWhatsappSession?.link_state ?? t('not linked') }}</small>
      </article>
      <article class="metric-card">
        <span>{{ t('Runtime') }}</span>
        <strong>{{ runtimeStatus?.status ?? runtimeCapabilities?.runtime_mode ?? t('unknown') }}</strong>
        <small>{{ runtimeStatus?.runtime_kind ?? t('Fixture/manual foundation') }}</small>
      </article>
      <article class="metric-card">
        <span>{{ t('Blocked') }}</span>
        <strong>{{ (runtimeCapabilities?.capabilities ?? []).filter((item) => item.status === 'blocked').length }}</strong>
        <small>{{ runtimeHealth?.healthy ? t('Health checks passed') : t('Live runtime remains blocked') }}</small>
      </article>
    </div>

    <WhatsAppStatusMessages
      :action-message="store.whatsappActionMessage"
      :error="store.whatsappError"
    />

    <div class="whatsapp-runtime-grid">
      <div class="sidebar-stack">
        <WhatsAppRuntimeAccountList
          v-model:include-removed-accounts="includeRemovedAccounts"
          :accounts="accounts"
          :selected-account-id="selectedAccountId"
          @select-account="selectWhatsappAccount"
        />

        <WhatsAppSessionList
          :whatsapp-sessions="sessions"
          :selected-whatsapp-session-id="store.selectedWhatsappSessionId"
          :is-whatsapp-loading="sessionsQuery.isLoading.value"
          @select-session="selectWhatsappSession"
        />
      </div>

      <div class="runtime-stack">
        <WhatsAppRuntimeAccountProvisioning
          v-model:live-account-shape="liveAccountShape"
          v-model:live-account-id="liveAccountId"
          v-model:live-account-display-name="liveAccountDisplayName"
          v-model:live-account-external-id="liveAccountExternalId"
          v-model:live-account-device-name="liveAccountDeviceName"
          v-model:live-account-local-state-path="liveAccountLocalStatePath"
          :capabilities="capabilities"
          :live-account-provider-kind="liveAccountProviderKind"
          :live-account-supports-device-fields="liveAccountSupportsDeviceFields"
          :selected-provider-shape-meta="selectedProviderShapeMeta"
          :live-account-session-mode="liveAccountSessionMode"
          :is-submitting="store.isWhatsappActionSubmitting"
          @create-live-account="createLiveAccount"
        />

        <WhatsAppRuntimeControl
          :selected-account-id="selectedAccountId"
          :selected-account-summary="selectedAccountSummary"
          :runtime-status="runtimeStatus"
          :runtime-capabilities="runtimeCapabilities"
          :runtime-health="runtimeHealth"
          :runtime-health-checks="runtimeHealthChecks"
          :companion-open-manifest="companionOpenManifest"
          :can-open-web-companion="canOpenWebCompanion"
          :is-runtime-busy="isRuntimeBusy"
          @open-companion="openVisibleWebCompanion"
          @set-runtime-state="setRuntimeState"
        />

        <WhatsAppRuntimeLinking
          v-model:pair-code-phone-number="pairCodePhoneNumber"
          :runtime-status="runtimeStatus"
          :selected-account-id="selectedAccountId"
          :is-runtime-busy="isRuntimeBusy"
          :active-qr-session="activeQrSession"
          :active-pair-code-session="activePairCodeSession"
          @set-runtime-state="setRuntimeState"
        />

        <WhatsAppRuntimeCapabilities :runtime-capabilities="runtimeCapabilities" />

        <WhatsAppRuntimeCommandAudit
          :provider-commands="providerCommands"
          :is-runtime-busy="isRuntimeBusy"
          @retry="retryCommand"
          @dead-letter="deadLetterCommand"
        />

        <WhatsAppRuntimeSnapshots
          v-model:status-publish-text="statusPublishText"
          :selected-account-id="selectedAccountId"
          :selected-sync-chat-id-resolved="selectedSyncChatIdResolved"
          :chat-items="chatItems"
          :history-items="historyItems"
          :member-items="memberItems"
          :status-items="statusItems"
          :presence-items="presenceItems"
          :call-items="callItems"
          :contact-items="contactItems"
          :media-items="mediaItems"
          :is-runtime-busy="isRuntimeBusy"
          @select-chat="(providerChatId) => { selectedSyncChatId = providerChatId }"
          @publish-status="publishStatus"
        />

        <WhatsAppRail
          :whatsapp-capabilities="runtimeCapabilities"
          :whatsapp-closure-capabilities="runtimeCapabilities?.capabilities.filter((item) => item.closure_gate) ?? []"
          :whatsapp-blocked-capabilities="runtimeCapabilities?.capabilities.filter((item) => item.status === 'blocked') ?? []"
          :whatsapp-provider-accounts="accounts.length"
          :is-whatsapp-action-submitting="store.isWhatsappActionSubmitting"
          :open-account-drawer="setupFixtureAccount"
          :ingest-whatsapp-web-message-fixture="ingestFixtureMessage"
          :whatsapp-message-form="store.whatsappMessageForm"
        />
      </div>
    </div>
  </section>
</template>

<style scoped src="./WhatsAppRuntimePanel.css"></style>
