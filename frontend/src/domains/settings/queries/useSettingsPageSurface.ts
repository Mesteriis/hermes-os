import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import { useAISettingsSurface } from './useAISettingsSurface'
import { useApplicationSettingsSurface } from './useApplicationSettingsSurface'
import { useBackgroundJobsSettingsSurface } from './useBackgroundJobsSettingsSurface'
import { useCommunicationsSettingsSurface } from './useCommunicationsSettingsSurface'
import { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'
import { useLanguageSettingsSurface } from './useLanguageSettingsSurface'
import { useMaintenanceSettingsSurface } from './useMaintenanceSettingsSurface'
import { useSignalHubSettingsSurface } from './useSignalHubSettingsSurface'
import { useTraceLogsSettingsSurface } from './useTraceLogsSettingsSurface'
import { useSettingsStore } from '../stores/settings'
import {
  buildSettingsOverviewCards,
  buildSettingsTreeGroups,
  findSelectedSettingsTreeItem,
  type SettingsOverviewCard,
  type SettingsTreeGroup,
  type SettingsTreeItem
} from './settingsPagePresentation'

export function useSettingsPageSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const realtimeStatus = useRealtimeStatusStore()
  const applicationSettings = useApplicationSettingsSurface()
  const aiSettings = useAISettingsSurface()
  const integrationsSettings = useIntegrationsSettingsSurface()
  const communicationsSettings = useCommunicationsSettingsSurface({ applicationSettings, integrationsSettings })
  const languageSettings = useLanguageSettingsSurface()
  const maintenanceSettings = useMaintenanceSettingsSurface()
  const signalHubSettings = useSignalHubSettingsSurface()
  const backgroundJobsSettings = useBackgroundJobsSettingsSurface({
    aiSettings,
    integrationsSettings,
    realtimeStatus,
    signalHubSettings
  })
  const traceLogsSettings = useTraceLogsSettingsSurface()

  const applicationSettingsCount = computed(() => applicationSettings.applicationSettings.value.length)
  const integrationCount = computed(() => integrationsSettings.accounts.value.length)
  const aiProviderCount = computed(() => aiSettings.providers.value.length)
  const signalSourceCount = computed(() => signalHubSettings.signalInventoryRows.value.length)
  const backgroundJobCount = computed(() => backgroundJobsSettings.backgroundJobRows.value.length)
  const traceSpanCount = computed(() => traceLogsSettings.traceEventRows.value.length)

  const settingsTreeGroups = computed<SettingsTreeGroup[]>(() => buildSettingsTreeGroups({
    integrationCount: integrationCount.value,
    communicationsAccountCount: communicationsSettings.mailAccounts.value.length,
    applicationSettingsCount: applicationSettingsCount.value,
    backgroundJobCount: backgroundJobCount.value,
    backgroundJobsLoading: backgroundJobsSettings.isLoading.value,
    traceSpanCount: traceSpanCount.value,
    traceLogsLoading: traceLogsSettings.isLoading.value,
    maintenanceTotalSizeLabel: maintenanceSettings.totalSizeLabel.value,
    maintenanceLoading: maintenanceSettings.isLoading.value,
    aiProviderCount: aiProviderCount.value,
    signalSourceCount: signalSourceCount.value,
    signalHubLoading: signalHubSettings.isLoading.value
  }, t))

  const selectedTreeItem = computed<SettingsTreeItem | null>(() => {
    return findSelectedSettingsTreeItem(settingsTreeGroups.value, store.selectedSection)
  })

  const settingsOverviewCards = computed<SettingsOverviewCard[]>(() => buildSettingsOverviewCards({
    realtimeStatusLabel: realtimeStatus.realtimeStatusLabel,
    realtimeStatusTone: realtimeStatus.realtimeStatusTone,
    realtimeHasError: Boolean(realtimeStatus.status.error),
    integrationCount: integrationCount.value,
    applicationSettingsCount: applicationSettingsCount.value,
    applicationSettingsLoading: applicationSettings.isLoading.value,
    aiProviderCount: aiProviderCount.value,
    aiLoading: aiSettings.isLoading.value
  }, t))

  return {
    aiSettings,
    applicationSettings,
    applicationSettingsCount,
    backgroundJobsSettings,
    communicationsSettings,
    integrationCount,
    integrationsSettings,
    languageSettings,
    maintenanceSettings,
    realtimeStatus,
    settingsOverviewCards,
    settingsTreeGroups,
    selectedTreeItem,
    signalHubSettings,
    store,
    traceLogsSettings
  }
}
