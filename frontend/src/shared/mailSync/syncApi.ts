import { ApiClient } from '../../platform/api/ApiClient'
import type {
  MailContentEgressSettings,
  MailSensitiveForwardingPolicy,
  MailSensitiveForwardingPolicyInput,
  MailSensitiveForwardingPolicyListResponse,
  MailSyncRunResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncStatusListResponse,
} from './types'

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

export async function fetchMailContentEgressSettings(accountId: string): Promise<MailContentEgressSettings> {
  return ApiClient.instance.get<MailContentEgressSettings>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/content-egress-settings`,
    'Mail content egress settings request failed'
  )
}

export async function updateMailContentEgressSettings(
  accountId: string,
  settings: Partial<MailContentEgressSettings>
): Promise<MailContentEgressSettings> {
  return ApiClient.instance.put<MailContentEgressSettings>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/content-egress-settings`,
    settings,
    'Mail content egress settings update failed'
  )
}

export async function fetchMailSensitiveForwardingPolicies(
  accountId: string
): Promise<MailSensitiveForwardingPolicyListResponse> {
  return ApiClient.instance.get<MailSensitiveForwardingPolicyListResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sensitive-forwarding-policies`,
    'Sensitive forwarding policies request failed'
  )
}

export async function upsertMailSensitiveForwardingPolicy(
  accountId: string,
  policy: MailSensitiveForwardingPolicyInput
): Promise<MailSensitiveForwardingPolicyListResponse> {
  return ApiClient.instance.post<MailSensitiveForwardingPolicyListResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sensitive-forwarding-policies`,
    policy,
    'Sensitive forwarding policy update failed'
  )
}

export async function deleteMailSensitiveForwardingPolicy(
  accountId: string,
  policyId: string
): Promise<{ policy_id: string; deleted: boolean }> {
  return ApiClient.instance.delete<{ policy_id: string; deleted: boolean }>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sensitive-forwarding-policies/${encodeURIComponent(policyId)}`,
    'Sensitive forwarding policy deletion failed'
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
