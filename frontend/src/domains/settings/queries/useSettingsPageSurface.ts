import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import { useThemeStore } from '../../../shared/stores/theme'
import { useApplicationSettingsQuery, useProviderAccountsQuery } from './useSettingsQuery'
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
  id: 'realtime' | 'appearance' | 'sources' | 'registry'
  icon: string
  label: string
  value: string
  detail: string
  tone: 'neutral' | 'success' | 'warning' | 'danger'
}

export function useSettingsPageSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const theme = useThemeStore()
  const realtimeStatus = useRealtimeStatusStore()
  const { data: appSettingsData, isLoading: isApplicationSettingsLoading } = useApplicationSettingsQuery()
  const { data: providerAccountsData, isLoading: isProviderAccountsLoading } = useProviderAccountsQuery()

  const applicationSettingsCount = computed(() => appSettingsData.value?.items.length ?? 0)
  const integrationCount = computed(() => providerAccountsData.value?.items.length ?? 0)

  const settingsTreeGroups = computed<SettingsTreeGroup[]>(() => [
    {
      label: 'Workspace',
      items: [
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
      label: 'Interface',
      items: [
        {
          id: 'appearance',
          label: 'Appearance',
          description: 'Theme, density, panel surface and preview',
          icon: 'tabler:palette'
        },
        {
          id: 'sidebar',
          label: 'Sidebar',
          description: 'Navigation groups and visible workspaces',
          icon: 'tabler:layout-sidebar'
        }
      ]
    },
    {
      label: 'Sources',
      items: [
        {
          id: 'integrations',
          label: 'Integrations',
          description: 'Provider accounts, capabilities and sync controls',
          icon: 'tabler:plug-connected',
          meta: isProviderAccountsLoading.value ? '...' : String(integrationCount.value)
        },
        {
          id: 'signal-hub',
          label: 'Signal Hub',
          description: 'Observed signals, profiles and replay operations',
          icon: 'tabler:database-import'
        }
      ]
    },
    {
      label: 'Intelligence',
      items: [
        {
          id: 'ai',
          label: 'AI Control Center',
          description: 'Local model providers, readiness and consent',
          icon: 'tabler:sparkles'
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
      id: 'appearance',
      icon: 'tabler:palette',
      label: 'Appearance',
      value: t(theme.themePersistenceLabel),
      detail: theme.themePersistenceError
        ? t(theme.themePersistenceError)
        : t('Theme is stored without private content or secrets'),
      tone: theme.themePersistenceError ? 'warning' : 'success'
    },
    {
      id: 'sources',
      icon: 'tabler:plug-connected',
      label: 'Sources',
      value: isProviderAccountsLoading.value ? t('Loading...') : String(integrationCount.value),
      detail: t('Provider accounts connected to the local workspace'),
      tone: integrationCount.value > 0 ? 'success' : 'neutral'
    },
    {
      id: 'registry',
      icon: 'tabler:list-check',
      label: 'Settings registry',
      value: isApplicationSettingsLoading.value ? t('Loading...') : String(applicationSettingsCount.value),
      detail: t('Declared application settings available for review'),
      tone: applicationSettingsCount.value > 0 ? 'success' : 'neutral'
    }
  ])

  return {
    applicationSettingsCount,
    integrationCount,
    realtimeStatus,
    settingsOverviewCards,
    settingsTreeGroups,
    selectedTreeItem,
    store,
    theme
  }
}
