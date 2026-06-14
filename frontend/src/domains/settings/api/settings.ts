import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ProviderAccountListResponse,
  CalendarAccount
} from '../types/settings'
import type { ProviderAccount } from '../types/settings'

export {
  fetchApplicationSettings,
  saveApplicationSetting,
  FRONTEND_LAYOUT_SETTING_KEY,
  FRONTEND_SIDEBAR_SETTING_KEY,
  FRONTEND_LOCALE_SETTING_KEY,
  FRONTEND_THEME_SETTING_KEY,
  FRONTEND_UI_STATE_SETTING_KEY
} from '../../../platform/settings/applicationSettingsClient'

export async function fetchProviderAccounts(): Promise<ProviderAccountListResponse> {
  return ApiClient.instance.get<ProviderAccountListResponse>(
    '/api/v1/settings/accounts',
    'Provider accounts request failed'
  )
}

export async function fetchCalendarAccounts(): Promise<{ items: CalendarAccount[] }> {
  return ApiClient.instance.get<{ items: CalendarAccount[] }>(
    '/api/v1/settings/accounts/calendar',
    'Calendar accounts request failed'
  )
}

export async function deleteMailAccount(accountId: string): Promise<{ result: boolean; error?: string }> {
  return ApiClient.instance.delete<{ result: boolean; error?: string }>(
    `/api/v1/settings/accounts/mail/${encodeURIComponent(accountId)}`,
    'Mail account delete failed'
  )
}

export async function logoutMailAccount(accountId: string): Promise<{ result: boolean; error?: string }> {
  return ApiClient.instance.post<{ result: boolean; error?: string }>(
    `/api/v1/settings/accounts/mail/${encodeURIComponent(accountId)}/logout`,
    {},
    'Mail account logout failed'
  )
}

export async function exportMailAccountSettings(
  accountId: string
): Promise<{ result?: { exported_at: string }; error?: string }> {
  return ApiClient.instance.get<{ result?: { exported_at: string }; error?: string }>(
    `/api/v1/settings/accounts/mail/${encodeURIComponent(accountId)}/export`,
    'Mail account export failed'
  )
}

export async function importMailAccountSettings(
  request: { account_id?: string; provider_kind: string; settings: Record<string, unknown> }
): Promise<{ result?: unknown; error?: string }> {
  return ApiClient.instance.post<{ result?: unknown; error?: string }>(
    '/api/v1/settings/accounts/mail/import',
    request,
    'Mail account import failed'
  )
}
