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
  MailSensitiveForwardingPolicy,
  MailSensitiveForwardingPolicyInput,
} from '../../../shared/mailSync/types'
import { findSetting } from './useSettingsQuery'
import { isMailProvider } from './integrationAccountPresentation'
import { useSettingsStore } from '../stores/settings'
import type { useApplicationSettingsSurface } from './useApplicationSettingsSurface'
import type { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'

const MAIL_DEGRADATION_THRESHOLD_KEY = 'communications.mail.consecutive_failures_before_degraded'

function newSensitiveForwardingPolicyDraft(deliveryAccountId: string): MailSensitiveForwardingPolicyInput {
  return {
    delivery_account_id: deliveryAccountId,
    name: 'Sensitive mail notification',
    enabled: false,
    include_message_body: false,
    include_attachments: false,
    fixed_recipients: [],
    minimum_severity: 'high',
    subject_template: 'Sensitive mail alert: {{severity}}',
    body_template: 'Hermes detected a sensitive message. Reference: {{message_id}}\n{{attachment_notice}}',
    max_sends_per_hour: 3,
    quiet_hours: {},
    expires_at: null,
  }
}

function sensitiveForwardingPolicyInput(policy: MailSensitiveForwardingPolicy): MailSensitiveForwardingPolicyInput {
  return {
    policy_id: policy.policy_id,
    delivery_account_id: policy.delivery_account_id,
    name: policy.name,
    enabled: policy.enabled,
    include_message_body: policy.include_message_body,
    include_attachments: policy.include_attachments,
    fixed_recipients: [...policy.fixed_recipients],
    minimum_severity: policy.minimum_severity,
    subject_template: policy.subject_template,
    body_template: policy.body_template,
    max_sends_per_hour: policy.max_sends_per_hour,
    quiet_hours: { ...policy.quiet_hours },
    expires_at: policy.expires_at,
  }
}

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

  async function saveSelectedMailSyncSettings() {
    const account = selectedMailAccount.value
    const settings = selectedSyncSettings.value
    if (!account || !settings) return

    const batchSize = Number.parseInt(batchSizeDraft.value, 10)
    const pollIntervalSeconds = Number.parseInt(pollIntervalDraft.value, 10)
    if (!Number.isInteger(batchSize) || batchSize < 1 || !Number.isInteger(pollIntervalSeconds) || pollIntervalSeconds < 1) {
      store.setError('Mail sync batch size and poll interval must be positive integers.')
      return
    }

    store.clearMessages()
    try {
      await updateSyncSettings.mutateAsync({
        accountId: account.account_id,
        settings: {
          sync_enabled: settings.sync_enabled,
          batch_size: batchSize,
          poll_interval_seconds: pollIntervalSeconds,
          failure_threshold: settings.failure_threshold ?? 3,
        },
      })
      store.setActionMessage('Mail sync settings saved')
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Mail sync settings update failed')
    }
  }

  async function toggleSelectedMailSync(enabled: boolean) {
    const account = selectedMailAccount.value
    const settings = selectedSyncSettings.value
    if (!account || !settings) return

    store.clearMessages()
    try {
      await updateSyncSettings.mutateAsync({
        accountId: account.account_id,
        settings: {
          ...settings,
          sync_enabled: enabled,
          failure_threshold: settings.failure_threshold ?? 3,
        },
      })
      store.setActionMessage(enabled ? 'Mail sync enabled' : 'Mail sync paused')
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Mail sync settings update failed')
    }
  }

  async function updateSelectedMailContentEgress(
    permission: keyof MailContentEgressSettings,
    enabled: boolean
  ) {
    const account = selectedMailAccount.value
    if (!account) return

    store.clearMessages()
    try {
      await updateContentEgressSettings.mutateAsync({
        accountId: account.account_id,
        settings: { [permission]: enabled },
      })
      store.setActionMessage('Mail content access preference saved')
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Mail content access preference update failed')
    }
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
    const account = selectedMailAccount.value
    const draft = sensitiveForwardingDraft.value
    if (!account) return
    if (!draft.delivery_account_id || !draft.name.trim() || draft.fixed_recipients.length === 0) {
      store.setError('Sensitive forwarding requires a delivery account, name and fixed recipients.')
      return
    }
    if (!Number.isInteger(draft.max_sends_per_hour) || draft.max_sends_per_hour < 1) {
      store.setError('Sensitive forwarding rate limit must be a positive integer.')
      return
    }

    store.clearMessages()
    try {
      const policies = await upsertSensitiveForwardingPolicy.mutateAsync({
        accountId: account.account_id,
        policy: draft,
      })
      const saved = policies.find((policy) => policy.policy_id === draft.policy_id) ?? policies[0]
      if (saved) {
        selectedSensitiveForwardingPolicyId.value = saved.policy_id
        sensitiveForwardingDraft.value = sensitiveForwardingPolicyInput(saved)
      }
      store.setActionMessage('Sensitive forwarding policy saved')
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Sensitive forwarding policy update failed')
    }
  }

  async function removeSelectedSensitiveForwardingPolicy() {
    const account = selectedMailAccount.value
    const policyId = selectedSensitiveForwardingPolicyId.value
    if (!account || !policyId) return

    store.clearMessages()
    try {
      await deleteSensitiveForwardingPolicy.mutateAsync({ accountId: account.account_id, policyId })
      selectedSensitiveForwardingPolicyId.value = null
      sensitiveForwardingDraft.value = newSensitiveForwardingPolicyDraft(account.account_id)
      store.setActionMessage('Sensitive forwarding policy deleted')
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Sensitive forwarding policy deletion failed')
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
    const account = selectedMailAccount.value
    if (!account || !resource.writable) return

    store.clearMessages()
    try {
      await updateProviderResourceMapping.mutateAsync({
        accountId: account.account_id,
        mappingId: resource.mapping_id,
        update,
      })
      store.setActionMessage('Mail provider mapping saved')
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Mail provider mapping update failed')
    }
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
    commandDiagnosticsStatus,
    degradationThresholdDraft,
    degradationThresholdSetting,
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
