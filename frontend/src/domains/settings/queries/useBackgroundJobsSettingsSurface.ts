import { computed, ref } from 'vue'
import { useSyncStatusesQuery } from '../../../shared/mailSync/runtimeQueries'
import { useSettingsStore, type SettingsSection } from '../stores/settings'
import {
  buildBackgroundJobRows,
  buildBackgroundJobSummaryTiles,
  buildBackgroundJobTabs,
  buildMailSyncStatusRows,
  filterBackgroundJobRows,
  type BackgroundJobFilter
} from '../components/backgroundJobsPresentation'
import type { AISettingsSurface } from './useAISettingsSurface'
import type { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'
import type { useSignalHubSettingsSurface } from './useSignalHubSettingsSurface'
import type { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'

type IntegrationsSettingsSurface = ReturnType<typeof useIntegrationsSettingsSurface>
type SignalHubSettingsSurface = ReturnType<typeof useSignalHubSettingsSurface>
type RealtimeStatusSurface = ReturnType<typeof useRealtimeStatusStore>

export interface BackgroundJobsSettingsDependencies {
  aiSettings: AISettingsSurface
  integrationsSettings: IntegrationsSettingsSurface
  realtimeStatus: RealtimeStatusSurface
  signalHubSettings: SignalHubSettingsSurface
}

export function useBackgroundJobsSettingsSurface({
  aiSettings,
  integrationsSettings,
  realtimeStatus,
  signalHubSettings
}: BackgroundJobsSettingsDependencies) {
  const store = useSettingsStore()
  const syncStatusesQuery = useSyncStatusesQuery()
  const activeJobFilter = ref<BackgroundJobFilter>('all')

  const mailStatuses = computed(() => syncStatusesQuery.data.value ?? [])
  const mailStatusesError = computed(() => {
    if (!syncStatusesQuery.isError.value) return null
    return syncStatusesQuery.error.value instanceof Error
      ? syncStatusesQuery.error.value.message
      : 'Mail sync status request failed'
  })
  const backgroundJobRows = computed(() =>
    buildBackgroundJobRows({
      aiBusy: aiSettings.isBusy.value,
      aiModelCount: aiSettings.models.value.length,
      aiProviderCount: aiSettings.providers.value.length,
      healthItems: signalHubSettings.healthItems.value,
      integrationAccountCount: integrationsSettings.accounts.value.length,
      mailStatuses: mailStatuses.value,
      mailStatusesError: mailStatusesError.value,
      mailStatusesLoading: syncStatusesQuery.isLoading.value,
      realtimeStatus: realtimeStatus.status,
      realtimeStatusLabel: realtimeStatus.realtimeStatusLabel,
      realtimeStatusTone: realtimeStatus.realtimeStatusTone,
      replayPendingCount: signalHubSettings.replayPendingCount.value,
      runtimeStates: signalHubSettings.runtimeStates.value,
      signalSourceCount: signalHubSettings.signalInventoryRows.value.length
    })
  )
  const backgroundJobTabs = computed(() => buildBackgroundJobTabs(backgroundJobRows.value))
  const filteredBackgroundJobRows = computed(() =>
    filterBackgroundJobRows(backgroundJobRows.value, activeJobFilter.value)
  )
  const summaryTiles = computed(() => buildBackgroundJobSummaryTiles(backgroundJobRows.value))
  const mailStatusRows = computed(() => buildMailSyncStatusRows(mailStatuses.value))
  const isLoading = computed(
    () => syncStatusesQuery.isLoading.value || signalHubSettings.isLoading.value || aiSettings.isLoading.value
  )

  function handleSelectJobFilter(filter: BackgroundJobFilter) {
    activeJobFilter.value = filter
  }

  function handleRefresh() {
    void syncStatusesQuery.refetch()
  }

  function handleOpenControl(section: SettingsSection) {
    store.selectSection(section)
  }

  return {
    activeJobFilter,
    backgroundJobRows,
    backgroundJobTabs,
    filteredBackgroundJobRows,
    handleOpenControl,
    handleRefresh,
    handleSelectJobFilter,
    isLoading,
    mailStatusRows,
    summaryTiles
  }
}

export type BackgroundJobsSettingsSurface = ReturnType<typeof useBackgroundJobsSettingsSurface>
