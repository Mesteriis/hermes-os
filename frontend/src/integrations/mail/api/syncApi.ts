import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  MailSyncStatusListResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncRunResponse
} from '../../../shared/mailSync/types'

export async function fetchMailSyncStatus(): Promise<MailSyncStatusListResponse> {
  return ApiClient.instance.get<MailSyncStatusListResponse>(
    '/api/v1/integrations/mail/accounts/sync-status',
    'Mail sync status request failed'
  )
}

export async function fetchMailSyncSettings(accountId: string): Promise<MailSyncSettings> {
  return ApiClient.instance.get<MailSyncSettings>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-settings`,
    'Mail sync settings request failed'
  )
}

export async function updateMailSyncSettings(
  accountId: string,
  settings: MailSyncSettingsUpdate
): Promise<MailSyncSettings> {
  return ApiClient.instance.put<MailSyncSettings>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-settings`,
    settings,
    'Mail sync settings update failed'
  )
}

export async function runMailSyncNow(accountId: string): Promise<MailSyncRunResponse> {
  return ApiClient.instance.post<MailSyncRunResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-now`,
    {},
    'Mail sync request failed'
  )
}

export async function runMailFullResync(accountId: string): Promise<MailSyncRunResponse> {
  return ApiClient.instance.post<MailSyncRunResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-full-resync`,
    {},
    'Mail full resync request failed'
  )
}
