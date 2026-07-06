import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import { useAISettingsSurface } from './useAISettingsSurface'
import { useApplicationSettingsSurface } from './useApplicationSettingsSurface'
import { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'
import { useLanguageSettingsSurface } from './useLanguageSettingsSurface'
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

  const applicationSettingsCount = computed(() => applicationSettings.applicationSettings.value.length)
  const integrationCount = computed(() => integrationsSettings.accounts.value.length)
  const aiProviderCount = computed(() => aiSettings.providers.value.length)

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
          id: 'language',
          label: 'Language',
          description: 'Interface language and locale preference',
          icon: 'tabler:language'
        }
      ]
    },
    {
      label: 'Intelligence',
      items: [
        {
          id: 'ai',
          label: 'AI Control Center',
          description: 'Providers, model catalog and action routing',
          icon: 'tabler:sparkles',
          meta: String(aiProviderCount.value)
        },
        {
          id: 'signal-hub',
          label: 'Signal Hub',
          description: 'Observed signals, profiles and replay operations',
          icon: 'tabler:database-import'
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
      detail: t('Provider accounts, model inventory and routes are owned by AI Control Center'),
      tone: aiProviderCount.value > 0 ? 'success' : 'neutral'
    }
  ])

  return {
    aiSettings,
    applicationSettings,
    applicationSettingsCount,
    integrationCount,
    integrationsSettings,
    languageSettings,
    realtimeStatus,
    settingsOverviewCards,
    settingsTreeGroups,
    selectedTreeItem,
    store,
  }
}
