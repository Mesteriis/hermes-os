import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import {
  deleteMailAccount,
  exportMailAccountSettings,
  fetchApplicationSettings,
  fetchProviderAccounts,
  fetchCalendarAccounts,
  logoutMailAccount,
  runAddressBookSyncNow,
  type ProviderAccountUpdate,
  updateCalendarAccount,
  updateProviderAccount,
} from '../api/settings'
import {
  saveApplicationSetting,
  type ApplicationSetting,
  type ApplicationSettingValue
} from '../../../platform/settings/applicationSettingsClient'
import {
  FRONTEND_LOCALE_SETTING_KEY,
  FRONTEND_SIDEBAR_SETTING_KEY,
} from '../types/settings'

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

export function useSaveApplicationSettingMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ settingKey, value }: { settingKey: string; value: ApplicationSettingValue }) =>
      saveApplicationSetting(settingKey, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.application() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}

export function useSaveFrontendLocaleMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (value: string) => saveApplicationSetting(FRONTEND_LOCALE_SETTING_KEY, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.application() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}

export function useSaveFrontendSidebarMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (value: ApplicationSettingValue) => saveApplicationSetting(FRONTEND_SIDEBAR_SETTING_KEY, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.application() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}

export function useExportMailAccountSettingsMutation() {
  return useMutation({
    mutationFn: (accountId: string) => exportMailAccountSettings(accountId),
  })
}

export function useUpdateProviderAccountMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      accountId,
      update
    }: {
      accountId: string
      update: ProviderAccountUpdate
    }) => updateProviderAccount(accountId, update),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.providerAccounts() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}

export function useLogoutMailAccountMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (accountId: string) => logoutMailAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.providerAccounts() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}

export function useDeleteMailAccountMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (accountId: string) => deleteMailAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.providerAccounts() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}

export function useUpdateCalendarAccountMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      accountId,
      update
    }: {
      accountId: string
      update: { account_name?: string; email?: string | null; sync_status?: string }
    }) => updateCalendarAccount(accountId, update),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.calendarAccounts() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}

export function useRunAddressBookSyncNowMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (accountId: string) => runAddressBookSyncNow(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.providerAccounts() })
      queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
    },
  })
}
