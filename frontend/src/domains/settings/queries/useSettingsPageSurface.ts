import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import { useAISettingsSurface } from './useAISettingsSurface'
import { useApplicationSettingsSurface } from './useApplicationSettingsSurface'
import { useBackgroundJobsSettingsSurface } from './useBackgroundJobsSettingsSurface'
import { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'
import { useLanguageSettingsSurface } from './useLanguageSettingsSurface'
import { useMaintenanceSettingsSurface } from './useMaintenanceSettingsSurface'
import { useSignalHubSettingsSurface } from './useSignalHubSettingsSurface'
import { useTraceLogsSettingsSurface } from './useTraceLogsSettingsSurface'
import { useSettingsStore, type SettingsSection } from '../stores/settings'

export type SettingsTreeItem = {
  id: SettingsSection
  label: string
  description: string
  icon: string
  meta?: string
}

export type SettingsTreeGroup = {
  label: string
  items: SettingsTreeItem[]
}

export type SettingsOverviewCard = {
  id: 'realtime' | 'sources' | 'registry' | 'ai'
  icon: string
  label: string
  value: string
  detail: string
  tone: 'neutral' | 'success' | 'warning' | 'danger'
}

export function useSettingsPageSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const realtimeStatus = useRealtimeStatusStore()
  const applicationSettings = useApplicationSettingsSurface()
  const aiSettings = useAISettingsSurface()
  const integrationsSettings = useIntegrationsSettingsSurface()
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

  const settingsTreeGroups = computed<SettingsTreeGroup[]>(() => [
    {
      label: 'Workspace',
      items: [
        {
          id: 'accounts',
          label: 'Accounts',
          description: 'Provider identities and service capabilities',
          icon: 'tabler:id',
          meta: String(integrationCount.value)
        },
        {
          id: 'application',
          label: 'Application',
          description: 'Runtime flags and declared workspace preferences',
          icon: 'tabler:adjustments-horizontal',
          meta: String(applicationSettingsCount.value)
        },
        {
          id: 'background-jobs',
          label: 'Background Jobs',
          description: 'Schedulers, workers and projection consumers',
          icon: 'tabler:clock-cog',
          meta: backgroundJobsSettings.isLoading.value ? t('Loading...') : String(backgroundJobCount.value)
        },
        {
          id: 'logs-traces',
          label: 'Logs & Traces',
          description: 'Event log spans and causal trace graph',
          icon: 'tabler:timeline-event',
          meta: traceLogsSettings.isLoading.value ? t('Loading...') : String(traceSpanCount.value)
        },
        {
          id: 'maintenance',
          label: 'Maintenance',
          description: 'Cleanup, backups and local storage sizes',
          icon: 'tabler:tool',
          meta: maintenanceSettings.isLoading.value ? t('Loading...') : maintenanceSettings.totalSizeLabel.value
        },
        {
          id: 'language',
          label: 'Language',
          description: 'Interface language and locale preference',
          icon: 'tabler:language'
        }
      ]
    },
    {
      label: 'Hub',
      items: [
        {
          id: 'ai',
          label: 'AI Hub',
          description: 'Providers, downloads, model catalog and action routing',
          icon: 'tabler:sparkles',
          meta: String(aiProviderCount.value)
        },
        {
          id: 'signal-hub',
          label: 'Signal Hub',
          description: 'Observed signals, profiles and replay operations',
          icon: 'tabler:database-import',
          meta: signalHubSettings.isLoading.value ? t('Loading...') : String(signalSourceCount.value)
        }
      ]
    }
  ])

  const selectedTreeItem = computed<SettingsTreeItem | null>(() => {
    for (const group of settingsTreeGroups.value) {
      const item = group.items.find((candidate) => candidate.id === store.selectedSection)
      if (item) return item
    }
    return null
  })

  const settingsOverviewCards = computed<SettingsOverviewCard[]>(() => [
    {
      id: 'realtime',
      icon: realtimeStatus.realtimeStatusTone === 'success' ? 'tabler:cloud-check' : 'tabler:cloud-exclamation',
      label: 'Realtime',
      value: t(realtimeStatus.realtimeStatusLabel),
      detail: realtimeStatus.status.error
        ? t('Realtime connection is retrying. Diagnostics keep the transport details.')
        : t('Replay cursor and UI cache updates are monitored by the local runtime.'),
      tone: realtimeStatus.realtimeStatusTone
    },
    {
      id: 'sources',
      icon: 'tabler:plug-connected',
      label: 'Sources',
      value: String(integrationCount.value),
      detail: t('Provider accounts connected to the local workspace'),
      tone: integrationCount.value > 0 ? 'success' : 'neutral'
    },
    {
      id: 'registry',
      icon: 'tabler:list-check',
      label: 'Settings registry',
      value: applicationSettings.isLoading.value ? t('Loading...') : String(applicationSettingsCount.value),
      detail: t('Declared application settings available for review'),
      tone: applicationSettingsCount.value > 0 ? 'success' : 'neutral'
    },
    {
      id: 'ai',
      icon: 'tabler:sparkles',
      label: 'AI providers',
      value: aiSettings.isLoading.value ? t('Loading...') : String(aiProviderCount.value),
      detail: t('Provider accounts, model inventory and routes are owned by AI Hub'),
      tone: aiProviderCount.value > 0 ? 'success' : 'neutral'
    }
  ])

  return {
    aiSettings,
    applicationSettings,
    applicationSettingsCount,
    backgroundJobsSettings,
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
