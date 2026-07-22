import type { RealtimeStatusTone } from '../../../shared/stores/realtimeStatus'
import type { SettingsSection } from '../stores/settings'

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

export function findSelectedSettingsTreeItem(
  groups: SettingsTreeGroup[],
  selectedSection: SettingsSection
): SettingsTreeItem | null {
  for (const group of groups) {
    const item = group.items.find((candidate) => candidate.id === selectedSection)
    if (item) return item
  }
  return null
}

interface SettingsPageTranslator {
  (key: string): string
}

export interface SettingsTreePresentationInput {
  integrationCount: number
  communicationsAccountCount: number
  applicationSettingsCount: number
  backgroundJobCount: number
  backgroundJobsLoading: boolean
  traceSpanCount: number
  traceLogsLoading: boolean
  maintenanceTotalSizeLabel: string
  maintenanceLoading: boolean
  aiProviderCount: number
  signalSourceCount: number
  signalHubLoading: boolean
}

export function buildSettingsTreeGroups(
  input: SettingsTreePresentationInput,
  t: SettingsPageTranslator
): SettingsTreeGroup[] {
  return [
    {
      label: 'Workspace',
      items: [
        {
          id: 'accounts',
          label: 'Accounts',
          description: 'Provider identities and service capabilities',
          icon: 'tabler:id',
          meta: String(input.integrationCount)
        },
        {
          id: 'communications',
          label: 'Communications',
          description: 'Mail sync reliability and provider polling settings',
          icon: 'tabler:mail-cog',
          meta: String(input.communicationsAccountCount)
        },
        {
          id: 'application',
          label: 'Application',
          description: 'Runtime flags and declared workspace preferences',
          icon: 'tabler:adjustments-horizontal',
          meta: String(input.applicationSettingsCount)
        },
        {
          id: 'background-jobs',
          label: 'Background Jobs',
          description: 'Schedulers, workers and projection consumers',
          icon: 'tabler:clock-cog',
          meta: input.backgroundJobsLoading ? t('Loading...') : String(input.backgroundJobCount)
        },
        {
          id: 'logs-traces',
          label: 'Logs & Traces',
          description: 'Event log spans and causal trace graph',
          icon: 'tabler:timeline-event',
          meta: input.traceLogsLoading ? t('Loading...') : String(input.traceSpanCount)
        },
        {
          id: 'maintenance',
          label: 'Maintenance',
          description: 'Cleanup, backups and local storage sizes',
          icon: 'tabler:tool',
          meta: input.maintenanceLoading ? t('Loading...') : input.maintenanceTotalSizeLabel
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
          meta: String(input.aiProviderCount)
        },
        {
          id: 'signal-hub',
          label: 'Signal Hub',
          description: 'Observed signals, profiles and replay operations',
          icon: 'tabler:database-import',
          meta: input.signalHubLoading ? t('Loading...') : String(input.signalSourceCount)
        }
      ]
    }
  ]
}

export interface SettingsOverviewPresentationInput {
  realtimeStatusLabel: string
  realtimeStatusTone: RealtimeStatusTone
  realtimeHasError: boolean
  integrationCount: number
  applicationSettingsCount: number
  applicationSettingsLoading: boolean
  aiProviderCount: number
  aiLoading: boolean
}

export function buildSettingsOverviewCards(
  input: SettingsOverviewPresentationInput,
  t: SettingsPageTranslator
): SettingsOverviewCard[] {
  return [
    {
      id: 'realtime',
      icon: input.realtimeStatusTone === 'success' ? 'tabler:cloud-check' : 'tabler:cloud-exclamation',
      label: 'Realtime',
      value: t(input.realtimeStatusLabel),
      detail: input.realtimeHasError
        ? t('Realtime connection is retrying. Diagnostics keep the transport details.')
        : t('Replay cursor and UI cache updates are monitored by the local runtime.'),
      tone: input.realtimeStatusTone
    },
    {
      id: 'sources',
      icon: 'tabler:plug-connected',
      label: 'Sources',
      value: String(input.integrationCount),
      detail: t('Provider accounts connected to the local workspace'),
      tone: input.integrationCount > 0 ? 'success' : 'neutral'
    },
    {
      id: 'registry',
      icon: 'tabler:list-check',
      label: 'Settings registry',
      value: input.applicationSettingsLoading ? t('Loading...') : String(input.applicationSettingsCount),
      detail: t('Declared application settings available for review'),
      tone: input.applicationSettingsCount > 0 ? 'success' : 'neutral'
    },
    {
      id: 'ai',
      icon: 'tabler:sparkles',
      label: 'AI providers',
      value: input.aiLoading ? t('Loading...') : String(input.aiProviderCount),
      detail: t('Provider accounts, model inventory and routes are owned by AI Hub'),
      tone: input.aiProviderCount > 0 ? 'success' : 'neutral'
    }
  ]
}
