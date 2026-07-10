import { computed, ref, watch } from 'vue'
import {
  useMailSyncSettingsQuery,
  useSyncStatusesQuery,
  useUpdateMailSyncSettingsMutation,
} from '../../../shared/mailSync/runtimeQueries'
import { findSetting } from './useSettingsQuery'
import { isMailProvider } from './integrationAccountPresentation'
import { useSettingsStore } from '../stores/settings'
import type { useApplicationSettingsSurface } from './useApplicationSettingsSurface'
import type { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'

const MAIL_DEGRADATION_THRESHOLD_KEY = 'communications.mail.consecutive_failures_before_degraded'

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
  const syncSettingsQuery = useMailSyncSettingsQuery(() => selectedMailAccountId.value)
  const syncStatusesQuery = useSyncStatusesQuery()
  const updateSyncSettings = useUpdateMailSyncSettingsMutation()

  const mailAccounts = computed(() =>
    integrationsSettings.accounts.value.filter((account) => isMailProvider(account.provider_kind))
  )
  const selectedMailAccount = computed(() =>
    mailAccounts.value.find((account) => account.account_id === selectedMailAccountId.value) ?? null
  )
  const selectedSyncSettings = computed(() => syncSettingsQuery.data.value ?? null)
  const selectedSyncStatus = computed(() =>
    (syncStatusesQuery.data.value ?? []).find((status) => status.account_id === selectedMailAccountId.value) ?? null
  )
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
        settings: { ...settings, sync_enabled: enabled },
      })
      store.setActionMessage(enabled ? 'Mail sync enabled' : 'Mail sync paused')
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Mail sync settings update failed')
    }
  }

  return {
    batchSizeDraft,
    degradationThresholdDraft,
    degradationThresholdSetting,
    isLoading,
    mailAccounts,
    pollIntervalDraft,
    saveDegradationThreshold,
    saveSelectedMailSyncSettings,
    selectMailAccount,
    selectedMailAccount,
    selectedSyncSettings,
    selectedSyncStatus,
    syncSaving: updateSyncSettings.isPending,
    toggleSelectedMailSync,
    updateDegradationThreshold,
  }
}

export type CommunicationsSettingsSurface = ReturnType<typeof useCommunicationsSettingsSurface>
