import { useQuery } from '@tanstack/vue-query'
import {
  fetchApplicationSettings,
  fetchProviderAccounts,
  fetchCalendarAccounts
} from '../api/settings'
import type { ApplicationSetting } from '../types/settings'

export const settingsKeys = {
  all: ['settings'] as const,
  application: () => [...settingsKeys.all, 'application'] as const,
  providerAccounts: () => [...settingsKeys.all, 'provider-accounts'] as const,
  calendarAccounts: () => [...settingsKeys.all, 'calendar-accounts'] as const,
  workspace: () => [...settingsKeys.all, 'workspace'] as const
}

export function useApplicationSettingsQuery() {
  return useQuery({
    queryKey: settingsKeys.application(),
    queryFn: fetchApplicationSettings
  })
}

export function useProviderAccountsQuery() {
  return useQuery({
    queryKey: settingsKeys.providerAccounts(),
    queryFn: fetchProviderAccounts
  })
}

export function useCalendarAccountsQuery() {
  return useQuery({
    queryKey: settingsKeys.calendarAccounts(),
    queryFn: fetchCalendarAccounts
  })
}

export function useSettingsWorkspaceQuery() {
  return useQuery({
    queryKey: settingsKeys.workspace(),
    queryFn: async () => {
      const [appSettings, providerAccounts, calendarAccounts] = await Promise.all([
        fetchApplicationSettings(),
        fetchProviderAccounts(),
        fetchCalendarAccounts()
      ])
      return { appSettings, providerAccounts, calendarAccounts }
    }
  })
}

/** Find a specific setting by key from a settings list. */
export function findSetting(
  settings: ApplicationSetting[] | undefined,
  key: string
): ApplicationSetting | null {
  if (!settings) return null
  return settings.find((s) => s.setting_key === key) ?? null
}

/** Group settings by category. */
export function groupSettingsByCategory(
  settings: ApplicationSetting[] | undefined
): Record<string, ApplicationSetting[]> {
  if (!settings) return {}
  const groups: Record<string, ApplicationSetting[]> = {}
  for (const setting of settings) {
    const cat = setting.category
    if (!groups[cat]) groups[cat] = []
    groups[cat].push(setting)
  }
  return groups
}
