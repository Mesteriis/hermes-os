import { computed, ref, watch } from 'vue'
import {
  useMailContentEgressSettingsQuery,
  useDeleteMailSensitiveForwardingPolicyMutation,
  useMailSensitiveForwardingPoliciesQuery,
  useMailSyncSettingsQuery,
  useMailLocalFoldersQuery,
  useMailProviderCommandDiagnosticsQuery,
  useRetryMailProviderCommandMutation,
  useMailProviderResourcesQuery,
  useSyncStatusesQuery,
  useUpdateMailContentEgressSettingsMutation,
  useUpsertMailSensitiveForwardingPolicyMutation,
  useUpdateMailProviderResourceMappingMutation,
  useUpdateMailSyncSettingsMutation,
} from '../../../shared/mailSync/runtimeQueries'
import type {
  MailProviderResource,
  MailProviderResourceMappingUpdate,
  MailProviderSemanticRole,
} from '../../../shared/mailSync/providerResources'
import type {
  MailContentEgressSettings,
  MailSensitiveForwardingPolicyInput,
} from '../../../shared/mailSync/types'
import { findSetting } from './useSettingsQuery'
import { isMailProvider } from './integrationAccountPredicates'
import { useSettingsStore } from '../stores/settings'
import type { useApplicationSettingsSurface } from './useApplicationSettingsSurface'
import type { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'
import { saveMailSyncSettings, toggleMailSync } from './communicationsMailSyncActions'
import { updateMailContentEgress } from './communicationsContentEgressActions'
import {
  deleteSensitiveForwardingPolicyAction,
  saveSensitiveForwardingPolicyAction,
} from './communicationsForwardingActions'
import { saveProviderResourceMappingAction } from './communicationsResourceMappingActions'
import {
  newSensitiveForwardingPolicyDraft,
  sensitiveForwardingPolicyInput,
} from './communicationsForwardingPresentation'

const MAIL_DEGRADATION_THRESHOLD_KEY = 'communications.mail.consecutive_failures_before_degraded'
const TELEGRAM_READ_RECEIPT_REPORTS_ENABLED_KEY = 'communications.telegram.read_receipt_reports_enabled'

type ApplicationSettingsSurface = ReturnType<typeof useApplicationSettingsSurface>
type IntegrationsSettingsSurface = ReturnType<typeof useIntegrationsSettingsSurface>

export interface CommunicationsSettingsDependencies {
  applicationSettings: ApplicationSettingsSurface
  integrationsSettings: IntegrationsSettingsSurface
}

export function useCommunicationsSettingsSurface({
  applicationSettings,
  integrationsSettings,
}: CommunicationsSettingsDependencies) {
  const store = useSettingsStore()
  const selectedMailAccountId = ref<string | null>(null)
  const batchSizeDraft = ref('')
  const pollIntervalDraft = ref('')
  const windowsDraft = ref('')
  const commandDiagnosticsStatus = ref('')
  const syncSettingsQuery = useMailSyncSettingsQuery(() => selectedMailAccountId.value)
  const contentEgressQuery = useMailContentEgressSettingsQuery(() => selectedMailAccountId.value)
  const sensitiveForwardingPoliciesQuery = useMailSensitiveForwardingPoliciesQuery(
    () => selectedMailAccountId.value
  )
  const commandDiagnosticsQuery = useMailProviderCommandDiagnosticsQuery(
    () => selectedMailAccountId.value,
    () => commandDiagnosticsStatus.value || null
  )
  const retryProviderCommand = useRetryMailProviderCommandMutation()
  const providerResourcesQuery = useMailProviderResourcesQuery(() => selectedMailAccountId.value)
  const localFoldersQuery = useMailLocalFoldersQuery(() => selectedMailAccountId.value)
  const syncStatusesQuery = useSyncStatusesQuery()
  const updateSyncSettings = useUpdateMailSyncSettingsMutation()
  const updateContentEgressSettings = useUpdateMailContentEgressSettingsMutation()
  const upsertSensitiveForwardingPolicy = useUpsertMailSensitiveForwardingPolicyMutation()
  const deleteSensitiveForwardingPolicy = useDeleteMailSensitiveForwardingPolicyMutation()
  const updateProviderResourceMapping = useUpdateMailProviderResourceMappingMutation()
  const mailSyncActionDependencies = {
    t: (key: string) => key,
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    updateSyncSettings: updateSyncSettings.mutateAsync,
  }
  const contentEgressActionDependencies = {
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    updateContentEgressSettings: updateContentEgressSettings.mutateAsync,
  }
  const forwardingActionDependencies = {
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    upsertPolicy: upsertSensitiveForwardingPolicy.mutateAsync,
    deletePolicy: deleteSensitiveForwardingPolicy.mutateAsync,
  }
  const resourceMappingActionDependencies = {
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    updateProviderResourceMapping: updateProviderResourceMapping.mutateAsync,
  }

  const mailAccounts = computed(() =>
    integrationsSettings.accounts.value.filter((account) => isMailProvider(account.provider_kind))
  )
  const selectedMailAccount = computed(() =>
    mailAccounts.value.find((account) => account.account_id === selectedMailAccountId.value) ?? null
  )
  const selectedSyncSettings = computed(() => syncSettingsQuery.data.value ?? null)
  const selectedContentEgress = computed(() => contentEgressQuery.data.value ?? null)
  const sensitiveForwardingPolicies = computed(() => sensitiveForwardingPoliciesQuery.data.value ?? [])
  const selectedSensitiveForwardingPolicyId = ref<string | null>(null)
  const sensitiveForwardingDraft = ref<MailSensitiveForwardingPolicyInput>(
    newSensitiveForwardingPolicyDraft('')
  )
  const selectedSyncStatus = computed(() =>
    (syncStatusesQuery.data.value ?? []).find((status) => status.account_id === selectedMailAccountId.value) ?? null
  )
  const commandDiagnostics = computed(() => commandDiagnosticsQuery.data.value ?? null)
  const providerResources = computed(() => providerResourcesQuery.data.value?.items ?? [])
  const localFolders = computed(() => localFoldersQuery.data.value ?? [])
  const degradationThresholdSetting = computed(() =>
    findSetting(applicationSettings.allApplicationSettings.value, MAIL_DEGRADATION_THRESHOLD_KEY)
  )
  const telegramReadReceiptReportsSetting = computed(() =>
    findSetting(applicationSettings.allApplicationSettings.value, TELEGRAM_READ_RECEIPT_REPORTS_ENABLED_KEY)
  )
  const degradationThresholdDraft = computed(() =>
    degradationThresholdSetting.value
      ? applicationSettings.settingDraftValue(degradationThresholdSetting.value)
      : '3'
  )
  const isLoading = computed(() =>
    applicationSettings.isLoading.value || syncSettingsQuery.isLoading.value || syncStatusesQuery.isLoading.value
  )

  watch(mailAccounts, (accounts) => {
    if (accounts.some((account) => account.account_id === selectedMailAccountId.value)) return
    selectedMailAccountId.value = accounts[0]?.account_id ?? null
  }, { immediate: true })

  watch(selectedSyncSettings, (settings) => {
    batchSizeDraft.value = settings ? String(settings.batch_size) : ''
    pollIntervalDraft.value = settings ? String(settings.poll_interval_seconds) : ''
    windowsDraft.value = settings ? String(settings.windows) : ''
  }, { immediate: true })

  watch([selectedMailAccount, sensitiveForwardingPolicies], ([account, policies]) => {
    const selected = policies.find((policy) => policy.policy_id === selectedSensitiveForwardingPolicyId.value)
    if (selected) return
    const first = policies[0]
    selectedSensitiveForwardingPolicyId.value = first?.policy_id ?? null
    sensitiveForwardingDraft.value = first
      ? sensitiveForwardingPolicyInput(first)
      : newSensitiveForwardingPolicyDraft(account?.account_id ?? '')
  }, { immediate: true })

  function selectMailAccount(accountId: string) {
    selectedMailAccountId.value = accountId
  }

  function updateDegradationThreshold(value: string) {
    const setting = degradationThresholdSetting.value
    if (setting) applicationSettings.handleInput(setting, value)
  }

  async function saveDegradationThreshold() {
    const setting = degradationThresholdSetting.value
    if (!setting) return
    await applicationSettings.handleSave(setting)
  }

  async function updateTelegramReadReceiptReports(enabled: boolean) {
    const setting = telegramReadReceiptReportsSetting.value
    if (!setting) return
    applicationSettings.handleInput(setting, String(enabled))
    await applicationSettings.handleSave(setting)
  }

  async function saveSelectedMailSyncSettings() {
    await saveMailSyncSettings(
      selectedMailAccount.value?.account_id ?? null,
      selectedSyncSettings.value,
      batchSizeDraft.value,
      pollIntervalDraft.value,
      windowsDraft.value,
      mailSyncActionDependencies
    )
  }

  async function toggleSelectedMailSync(enabled: boolean) {
    await toggleMailSync(
      selectedMailAccount.value?.account_id ?? null,
      selectedSyncSettings.value,
      enabled,
      mailSyncActionDependencies
    )
  }

  async function updateSelectedMailContentEgress(
    permission: keyof MailContentEgressSettings,
    enabled: boolean
  ) {
    await updateMailContentEgress(
      selectedMailAccount.value?.account_id ?? null,
      permission,
      enabled,
      contentEgressActionDependencies
    )
  }

  function selectSensitiveForwardingPolicy(policyId: string) {
    const policy = sensitiveForwardingPolicies.value.find((item) => item.policy_id === policyId)
    if (!policy) return
    selectedSensitiveForwardingPolicyId.value = policy.policy_id
    sensitiveForwardingDraft.value = sensitiveForwardingPolicyInput(policy)
  }

  function createSensitiveForwardingPolicy() {
    const account = selectedMailAccount.value
    if (!account) return
    selectedSensitiveForwardingPolicyId.value = null
    sensitiveForwardingDraft.value = newSensitiveForwardingPolicyDraft(account.account_id)
  }

  function updateSensitiveForwardingDraft(update: Partial<MailSensitiveForwardingPolicyInput>) {
    sensitiveForwardingDraft.value = { ...sensitiveForwardingDraft.value, ...update }
  }

  function updateSensitiveForwardingRecipients(value: string) {
    updateSensitiveForwardingDraft({
      fixed_recipients: value.split(',').map((recipient) => recipient.trim()).filter(Boolean),
    })
  }

  function sensitiveForwardingQuietHour(key: 'start' | 'end'): string {
    const value = sensitiveForwardingDraft.value.quiet_hours[key]
    return typeof value === 'string' ? value : ''
  }

  function updateSensitiveForwardingQuietHours(start: string, end: string) {
    updateSensitiveForwardingDraft({
      quiet_hours: start && end ? { timezone: 'UTC', start, end } : {},
    })
  }

  function sensitiveForwardingExpiryValue(): string {
    return sensitiveForwardingDraft.value.expires_at?.replace(/Z$/, '').slice(0, 16) ?? ''
  }

  function updateSensitiveForwardingExpiry(value: string) {
    if (!value) {
      updateSensitiveForwardingDraft({ expires_at: null })
      return
    }
    const parsed = new Date(`${value}:00Z`)
    if (Number.isNaN(parsed.getTime())) {
      store.setError('Sensitive forwarding expiry must be a valid UTC date and time.')
      return
    }
    updateSensitiveForwardingDraft({ expires_at: parsed.toISOString() })
  }

  async function saveSensitiveForwardingPolicy() {
    const draft = sensitiveForwardingDraft.value
    const policies = await saveSensitiveForwardingPolicyAction(
      selectedMailAccount.value?.account_id ?? null,
      draft,
      forwardingActionDependencies
    )
    if (policies) {
      const saved = policies.find((policy) => policy.policy_id === draft.policy_id) ?? policies[0]
      if (saved) {
        selectedSensitiveForwardingPolicyId.value = saved.policy_id
        sensitiveForwardingDraft.value = sensitiveForwardingPolicyInput(saved)
      }
    }
  }

  async function removeSelectedSensitiveForwardingPolicy() {
    const accountId = selectedMailAccount.value?.account_id ?? null
    const policyId = selectedSensitiveForwardingPolicyId.value
    const deleted = await deleteSensitiveForwardingPolicyAction(
      accountId,
      policyId,
      forwardingActionDependencies
    )
    if (deleted && accountId) {
      selectedSensitiveForwardingPolicyId.value = null
      sensitiveForwardingDraft.value = newSensitiveForwardingPolicyDraft(accountId)
    }
  }

  async function refreshCommandDiagnostics() {
    await commandDiagnosticsQuery.refetch()
  }

  async function retryMailProviderCommand(commandId: string) {
    await retryProviderCommand.mutateAsync(commandId)
  }

  async function saveProviderResourceMapping(
    resource: MailProviderResource,
    update: MailProviderResourceMappingUpdate
  ) {
    await saveProviderResourceMappingAction(
      selectedMailAccount.value?.account_id ?? null,
      resource,
      update,
      resourceMappingActionDependencies
    )
  }

  async function updateProviderResourceRole(
    resource: MailProviderResource,
    semanticRole: MailProviderSemanticRole | null
  ) {
    await saveProviderResourceMapping(resource, {
      semantic_role: semanticRole,
      local_folder_id: resource.local_folder_id,
    })
  }

  async function updateProviderResourceLocalFolder(
    resource: MailProviderResource,
    localFolderId: string | null
  ) {
    await saveProviderResourceMapping(resource, {
      semantic_role: resource.semantic_role,
      local_folder_id: localFolderId,
    })
  }

  return {
    batchSizeDraft,
    commandDiagnostics,
    windowsDraft,
    commandDiagnosticsStatus,
    degradationThresholdDraft,
    degradationThresholdSetting,
    telegramReadReceiptReportsSetting,
    isLoading,
    localFolders,
    localFoldersLoading: localFoldersQuery.isLoading,
    mailAccounts,
    pollIntervalDraft,
    providerResources,
    providerResourcesLoading: providerResourcesQuery.isLoading,
    providerResourcesSaving: updateProviderResourceMapping.isPending,
    refreshCommandDiagnostics,
    retryMailProviderCommand,
    saveDegradationThreshold,
    updateTelegramReadReceiptReports,
    saveSelectedMailSyncSettings,
    saveSensitiveForwardingPolicy,
    removeSelectedSensitiveForwardingPolicy,
    selectMailAccount,
    selectSensitiveForwardingPolicy,
    selectedMailAccount,
    selectedContentEgress,
    selectedSensitiveForwardingPolicyId,
    selectedSyncSettings,
    selectedSyncStatus,
    sensitiveForwardingDraft,
    sensitiveForwardingPolicies,
    sensitiveForwardingPoliciesLoading: sensitiveForwardingPoliciesQuery.isLoading,
    sensitiveForwardingSaving: upsertSensitiveForwardingPolicy.isPending,
    sensitiveForwardingDeleting: deleteSensitiveForwardingPolicy.isPending,
    commandDiagnosticsLoading: commandDiagnosticsQuery.isLoading,
    commandDiagnosticsRefreshing: commandDiagnosticsQuery.isFetching,
    commandDiagnosticsRetrying: retryProviderCommand.isPending,
    syncSaving: updateSyncSettings.isPending,
    contentEgressLoading: contentEgressQuery.isLoading,
    contentEgressSaving: updateContentEgressSettings.isPending,
    toggleSelectedMailSync,
    updateSelectedMailContentEgress,
    updateSensitiveForwardingDraft,
    updateSensitiveForwardingExpiry,
    updateSensitiveForwardingQuietHours,
    updateSensitiveForwardingRecipients,
    sensitiveForwardingExpiryValue,
    sensitiveForwardingQuietHour,
    createSensitiveForwardingPolicy,
    updateProviderResourceLocalFolder,
    updateProviderResourceRole,
    updateDegradationThreshold,
  }
}

export type CommunicationsSettingsSurface = ReturnType<typeof useCommunicationsSettingsSurface>
