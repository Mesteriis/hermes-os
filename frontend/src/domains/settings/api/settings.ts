import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CalendarAccount,
  ProviderAccount,
  ProviderAccountListResponse,
} from '../types/settings'

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

export interface ProviderAccountUpdate {
  display_name?: string
  address_book_sync_enabled?: boolean
  address_book_sync_direction?: 'read_only' | 'bidirectional'
  address_book_remote_write_enabled?: boolean
}

export async function updateProviderAccount(
  accountId: string,
  update: ProviderAccountUpdate
): Promise<ProviderAccount> {
  return ApiClient.instance.patch<ProviderAccount>(
    `/api/v1/settings/accounts/${encodeURIComponent(accountId)}`,
    update,
    'Provider account update failed'
  )
}

export async function fetchCalendarAccounts(): Promise<{ items: CalendarAccount[] }> {
  return ApiClient.instance.get<{ items: CalendarAccount[] }>(
    '/api/v1/calendar/accounts',
    'Calendar accounts request failed'
  )
}

export async function updateCalendarAccount(
  accountId: string,
  update: { account_name?: string; email?: string | null; sync_status?: string }
): Promise<CalendarAccount> {
  return ApiClient.instance.put<CalendarAccount>(
    `/api/v1/calendar/accounts/${encodeURIComponent(accountId)}`,
    update,
    'Calendar account update failed'
  )
}

export async function deleteMailAccount(
  accountId: string
): Promise<{
  account_id: string
  deleted: boolean
  unbound_secret_refs: string[]
  vault_deleted_secret_refs: string[]
  retained_secret_refs: string[]
}> {
  return ApiClient.instance.delete<{
    account_id: string
    deleted: boolean
    unbound_secret_refs: string[]
    vault_deleted_secret_refs: string[]
    retained_secret_refs: string[]
  }>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}`,
    'Mail account delete failed'
  )
}

export async function logoutMailAccount(
  accountId: string
): Promise<{ account: unknown; capabilities: unknown; sync_settings: unknown }> {
  return ApiClient.instance.post<{ account: unknown; capabilities: unknown; sync_settings: unknown }>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/logout`,
    {},
    'Mail account logout failed'
  )
}

export async function exportMailAccountSettings(
  accountId: string
): Promise<{ exported_at: string; account: unknown; capabilities: unknown; sync_settings: unknown }> {
  return ApiClient.instance.get<{ exported_at: string; account: unknown; capabilities: unknown; sync_settings: unknown }>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/export`,
    'Mail account export failed'
  )
}

export async function importMailAccountSettings(
  request: {
    account: {
      account_id: string
      provider_kind: string
      display_name: string
      external_account_id: string
      config?: Record<string, unknown>
    }
    sync_settings?: {
      sync_enabled?: boolean
      batch_size?: number
      poll_interval_seconds?: number
    }
  }
): Promise<{ account: unknown; capabilities: unknown; sync_settings: unknown }> {
  return ApiClient.instance.post<{ account: unknown; capabilities: unknown; sync_settings: unknown }>(
    '/api/v1/integrations/mail/accounts/import',
    request,
    'Mail account import failed'
  )
}

export interface AddressBookSyncRunResponse {
  status: string
  provider_entries_seen: number
  provider_entries_upserted: number
  provider_entries_skipped: number
  local_entries_seen: number
  local_entries_pushed: number
  local_entries_blocked: number
}

export async function runAddressBookSyncNow(
  accountId: string
): Promise<AddressBookSyncRunResponse> {
  return ApiClient.instance.post<AddressBookSyncRunResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/address-book-sync-now`,
    {},
    'Address book sync request failed'
  )
}
