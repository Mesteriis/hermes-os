import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ZoomAccountListResponse,
  ZoomAccountSetupResponse,
  ZoomAuthorizationResult,
  ZoomAuditEventResponse,
  ZoomCallTranscriptResponse,
  ZoomCapabilitiesResponse,
  ZoomLiveAccountSetupRequest,
  ZoomMeetingIngestResult,
  ZoomMeetingObservationRequest,
  ZoomOAuthCompleteRequest,
  ZoomOAuthStartRequest,
  ZoomOAuthStartResponse,
  ZoomProviderCallListResponse,
  ZoomRecordingIngestResult,
  ZoomRecordingImportAuditResponse,
  ZoomRecordingImportRemoveRequest,
  ZoomRecordingImportRemoveResponse,
  ZoomRetentionCleanupRequest,
  ZoomRetentionCleanupResponse,
  ZoomRecordingObservationRequest,
  ZoomRecordingSyncRequest,
  ZoomRecordingSyncResult,
  ZoomRuntimeRemoveRequest,
  ZoomRuntimeRemoveResponse,
  ZoomRuntimeStartRequest,
  ZoomRuntimeStatus,
  ZoomRuntimeStopRequest,
  ZoomServerToServerAuthorizeRequest,
  ZoomTokenMaintenanceRequest,
  ZoomTokenMaintenanceResult,
  ZoomTokenRefreshRequest,
  ZoomTokenRefreshResult,
  ZoomTranscriptFileImportRequest,
  ZoomTranscriptFileImportResult,
  ZoomTranscriptIngestResult,
  ZoomTranscriptObservationRequest,
  ZoomWebhookSubscriptionReconcileRequest,
  ZoomWebhookSubscriptionReconcileResult,
  ZoomWebhookSubscriptionRemoveRequest,
  ZoomWebhookSubscriptionRemoveResult,
  ZoomWebhookSubscriptionStatusResult,
} from '../types/zoom'

export async function fetchZoomCapabilities(): Promise<ZoomCapabilitiesResponse> {
  return ApiClient.instance.get<ZoomCapabilitiesResponse>(
    '/api/v1/integrations/zoom/capabilities',
    'Zoom capabilities request failed'
  )
}

export async function fetchZoomAccounts(includeRemoved = false): Promise<ZoomAccountListResponse> {
  const params = new URLSearchParams()
  if (includeRemoved) params.set('include_removed', 'true')
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<ZoomAccountListResponse>(
    `/api/v1/integrations/zoom/accounts${suffix}`,
    'Zoom accounts request failed'
  )
}

export async function setupZoomLiveAccount(
  request: ZoomLiveAccountSetupRequest
): Promise<ZoomAccountSetupResponse> {
  return ApiClient.instance.post<ZoomAccountSetupResponse>(
    '/api/v1/integrations/zoom/accounts',
    request,
    'Zoom account setup failed'
  )
}

export async function startZoomOAuth(
  request: ZoomOAuthStartRequest
): Promise<ZoomOAuthStartResponse> {
  return ApiClient.instance.post<ZoomOAuthStartResponse>(
    '/api/v1/integrations/zoom/oauth/start',
    request,
    'Zoom OAuth start failed'
  )
}

export async function completeZoomOAuth(
  request: ZoomOAuthCompleteRequest
): Promise<ZoomAuthorizationResult> {
  return ApiClient.instance.post<ZoomAuthorizationResult>(
    '/api/v1/integrations/zoom/oauth/complete',
    request,
    'Zoom OAuth completion failed'
  )
}

export async function authorizeZoomServerToServer(
  request: ZoomServerToServerAuthorizeRequest
): Promise<ZoomAuthorizationResult> {
  return ApiClient.instance.post<ZoomAuthorizationResult>(
    '/api/v1/integrations/zoom/oauth/server-to-server/authorize',
    request,
    'Zoom Server-to-Server authorization failed'
  )
}

export async function refreshZoomToken(
  request: ZoomTokenRefreshRequest
): Promise<ZoomTokenRefreshResult> {
  return ApiClient.instance.post<ZoomTokenRefreshResult>(
    '/api/v1/integrations/zoom/oauth/refresh',
    request,
    'Zoom token refresh failed'
  )
}

export async function maintainZoomTokens(
  request: ZoomTokenMaintenanceRequest = {}
): Promise<ZoomTokenMaintenanceResult> {
  return ApiClient.instance.post<ZoomTokenMaintenanceResult>(
    '/api/v1/integrations/zoom/oauth/maintenance',
    request,
    'Zoom token maintenance failed'
  )
}

export async function syncZoomRecordings(
  request: ZoomRecordingSyncRequest
): Promise<ZoomRecordingSyncResult> {
  return ApiClient.instance.post<ZoomRecordingSyncResult>(
    '/api/v1/integrations/zoom/provider-sync/recordings',
    request,
    'Zoom recording sync failed'
  )
}

export async function fetchZoomWebhookSubscriptionStatus(
  accountId: string,
  apiBaseUrl?: string | null
): Promise<ZoomWebhookSubscriptionStatusResult> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  if (apiBaseUrl?.trim()) params.set('api_base_url', apiBaseUrl.trim())
  return ApiClient.instance.get<ZoomWebhookSubscriptionStatusResult>(
    `/api/v1/integrations/zoom/webhook-subscriptions/status?${params.toString()}`,
    'Zoom webhook subscription status request failed'
  )
}

export async function reconcileZoomWebhookSubscription(
  request: ZoomWebhookSubscriptionReconcileRequest
): Promise<ZoomWebhookSubscriptionReconcileResult> {
  return ApiClient.instance.post<ZoomWebhookSubscriptionReconcileResult>(
    '/api/v1/integrations/zoom/webhook-subscriptions/reconcile',
    request,
    'Zoom webhook subscription reconcile failed'
  )
}

export async function removeZoomWebhookSubscription(
  request: ZoomWebhookSubscriptionRemoveRequest
): Promise<ZoomWebhookSubscriptionRemoveResult> {
  return ApiClient.instance.post<ZoomWebhookSubscriptionRemoveResult>(
    '/api/v1/integrations/zoom/webhook-subscriptions/remove',
    request,
    'Zoom webhook subscription removal failed'
  )
}

export async function fetchZoomRuntimeStatus(accountId: string): Promise<ZoomRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<ZoomRuntimeStatus>(
    `/api/v1/integrations/zoom/runtime/status?${params.toString()}`,
    'Zoom runtime status request failed'
  )
}

export async function fetchZoomRecordingImports(
  accountId: string,
  limit = 20
): Promise<ZoomRecordingImportAuditResponse> {
  const params = new URLSearchParams({ limit: String(limit) })
  return ApiClient.instance.get<ZoomRecordingImportAuditResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/recording-imports?${params.toString()}`,
    'Zoom recording imports request failed'
  )
}

export async function removeZoomRecordingImport(
  accountId: string,
  attachmentId: string,
  request: ZoomRecordingImportRemoveRequest = {}
): Promise<ZoomRecordingImportRemoveResponse> {
  return ApiClient.instance.post<ZoomRecordingImportRemoveResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/recording-imports/${encodeURIComponent(attachmentId.trim())}/remove`,
    request,
    'Zoom recording import removal failed'
  )
}

export async function fetchZoomAuditEvents(
  accountId: string,
  limit = 25
): Promise<ZoomAuditEventResponse> {
  const params = new URLSearchParams({ limit: String(limit) })
  return ApiClient.instance.get<ZoomAuditEventResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/audit-events?${params.toString()}`,
    'Zoom audit events request failed'
  )
}

export async function cleanupZoomRetention(
  accountId: string,
  request: ZoomRetentionCleanupRequest = {}
): Promise<ZoomRetentionCleanupResponse> {
  return ApiClient.instance.post<ZoomRetentionCleanupResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/retention/prune`,
    request,
    'Zoom retention cleanup failed'
  )
}

export async function fetchZoomProviderCalls(
  accountId?: string,
  limit = 20
): Promise<ZoomProviderCallListResponse> {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  params.set('provider', 'zoom')
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  return ApiClient.instance.get<ZoomProviderCallListResponse>(
    `/api/v1/calls?${params.toString()}`,
    'Zoom provider calls request failed'
  )
}

export async function fetchZoomCallTranscript(callId: string): Promise<ZoomCallTranscriptResponse> {
  return ApiClient.instance.get<ZoomCallTranscriptResponse>(
    `/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
    'Zoom call transcript request failed'
  )
}

export async function startZoomRuntime(
  request: ZoomRuntimeStartRequest
): Promise<ZoomRuntimeStatus> {
  return ApiClient.instance.post<ZoomRuntimeStatus>(
    '/api/v1/integrations/zoom/runtime/start',
    request,
    'Zoom runtime start failed'
  )
}

export async function stopZoomRuntime(request: ZoomRuntimeStopRequest): Promise<ZoomRuntimeStatus> {
  return ApiClient.instance.post<ZoomRuntimeStatus>(
    '/api/v1/integrations/zoom/runtime/stop',
    request,
    'Zoom runtime stop failed'
  )
}

export async function removeZoomRuntime(
  request: ZoomRuntimeRemoveRequest
): Promise<ZoomRuntimeRemoveResponse> {
  return ApiClient.instance.post<ZoomRuntimeRemoveResponse>(
    '/api/v1/integrations/zoom/runtime/remove',
    request,
    'Zoom runtime remove failed'
  )
}

export async function bridgeZoomMeeting(
  request: ZoomMeetingObservationRequest
): Promise<ZoomMeetingIngestResult> {
  return ApiClient.instance.post<ZoomMeetingIngestResult>(
    '/api/v1/integrations/zoom/runtime-bridge/meetings',
    request,
    'Zoom meeting bridge ingest failed'
  )
}

export async function bridgeZoomRecording(
  request: ZoomRecordingObservationRequest
): Promise<ZoomRecordingIngestResult> {
  return ApiClient.instance.post<ZoomRecordingIngestResult>(
    '/api/v1/integrations/zoom/runtime-bridge/recordings',
    request,
    'Zoom recording bridge ingest failed'
  )
}

export async function bridgeZoomTranscript(
  request: ZoomTranscriptObservationRequest
): Promise<ZoomTranscriptIngestResult> {
  return ApiClient.instance.post<ZoomTranscriptIngestResult>(
    '/api/v1/integrations/zoom/runtime-bridge/transcripts',
    request,
    'Zoom transcript bridge ingest failed'
  )
}

export async function importZoomTranscriptFile(
  request: ZoomTranscriptFileImportRequest
): Promise<ZoomTranscriptFileImportResult> {
  return ApiClient.instance.post<ZoomTranscriptFileImportResult>(
    '/api/v1/integrations/zoom/runtime-bridge/transcript-files',
    request,
    'Zoom transcript file import failed'
  )
}
