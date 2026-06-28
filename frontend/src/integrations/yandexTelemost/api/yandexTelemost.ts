import { invoke } from '@tauri-apps/api/core'
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  YandexTelemostAccountListResponse,
  YandexTelemostAccountSetupRequest,
  YandexTelemostAccountSetupResponse,
  YandexTelemostCapabilitiesResponse,
  YandexTelemostCohostPage,
  YandexTelemostCompanionManifest,
  YandexTelemostCompanionOpenRequest,
  YandexTelemostConferenceCreateRequest,
  YandexTelemostConferenceOperationResponse,
  YandexTelemostConferenceUpdateRequest,
  YandexTelemostConferenceWebviewManifest,
  YandexTelemostRecordingIntentResponse,
  YandexTelemostRecordingBridgeRequest,
  YandexTelemostRecordingBridgeResponse,
  YandexTelemostRecordingSession,
  YandexTelemostRecordingStopReceipt,
  YandexTelemostRuntimeStatus,
  YandexTelemostWebviewManifestRequest,
} from '../types/yandexTelemost'

export async function fetchYandexTelemostCapabilities(): Promise<YandexTelemostCapabilitiesResponse> {
  return ApiClient.instance.get<YandexTelemostCapabilitiesResponse>(
    '/api/v1/integrations/yandex-telemost/capabilities',
    'Yandex Telemost capabilities request failed'
  )
}

export async function fetchYandexTelemostAccounts(
  includeRemoved = false
): Promise<YandexTelemostAccountListResponse> {
  const params = new URLSearchParams()
  if (includeRemoved) params.set('include_removed', 'true')
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<YandexTelemostAccountListResponse>(
    `/api/v1/integrations/yandex-telemost/accounts${suffix}`,
    'Yandex Telemost accounts request failed'
  )
}

export async function setupYandexTelemostAccount(
  request: YandexTelemostAccountSetupRequest
): Promise<YandexTelemostAccountSetupResponse> {
  return ApiClient.instance.post<YandexTelemostAccountSetupResponse>(
    '/api/v1/integrations/yandex-telemost/accounts',
    request,
    'Yandex Telemost account setup failed'
  )
}

export async function fetchYandexTelemostRuntimeStatus(
  accountId: string
): Promise<YandexTelemostRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<YandexTelemostRuntimeStatus>(
    `/api/v1/integrations/yandex-telemost/runtime/status?${params.toString()}`,
    'Yandex Telemost runtime status request failed'
  )
}

export async function createYandexTelemostConference(
  request: YandexTelemostConferenceCreateRequest
): Promise<YandexTelemostConferenceOperationResponse> {
  const { account_id, ...body } = request
  return ApiClient.instance.post<YandexTelemostConferenceOperationResponse>(
    '/api/v1/integrations/yandex-telemost/conferences',
    { account_id, body },
    'Yandex Telemost conference creation failed'
  )
}

export async function readYandexTelemostConference(
  accountId: string,
  conferenceId: string
): Promise<YandexTelemostConferenceOperationResponse> {
  return ApiClient.instance.get<YandexTelemostConferenceOperationResponse>(
    `/api/v1/integrations/yandex-telemost/conferences/${encodeURIComponent(accountId.trim())}/${encodeURIComponent(conferenceId.trim())}`,
    'Yandex Telemost conference read failed'
  )
}

export async function updateYandexTelemostConference(
  accountId: string,
  conferenceId: string,
  request: YandexTelemostConferenceUpdateRequest
): Promise<YandexTelemostConferenceOperationResponse> {
  return ApiClient.instance.patch<YandexTelemostConferenceOperationResponse>(
    `/api/v1/integrations/yandex-telemost/conferences/${encodeURIComponent(accountId.trim())}/${encodeURIComponent(conferenceId.trim())}`,
    request,
    'Yandex Telemost conference update failed'
  )
}

export async function fetchYandexTelemostCohosts(
  accountId: string,
  conferenceId: string,
  offset?: number | null,
  limit?: number | null
): Promise<YandexTelemostCohostPage> {
  const params = new URLSearchParams()
  if (typeof offset === 'number') params.set('offset', String(offset))
  if (limit) params.set('limit', String(limit))
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<YandexTelemostCohostPage>(
    `/api/v1/integrations/yandex-telemost/conferences/${encodeURIComponent(accountId.trim())}/${encodeURIComponent(conferenceId.trim())}/cohosts${suffix}`,
    'Yandex Telemost cohosts request failed'
  )
}

export async function fetchYandexTelemostWebviewManifest(
  request: YandexTelemostWebviewManifestRequest
): Promise<YandexTelemostConferenceWebviewManifest> {
  return ApiClient.instance.post<YandexTelemostConferenceWebviewManifest>(
    '/api/v1/integrations/yandex-telemost/webview/manifest',
    request,
    'Yandex Telemost webview manifest request failed'
  )
}

export async function fetchYandexTelemostRecordingIntent(
  request: YandexTelemostWebviewManifestRequest
): Promise<YandexTelemostRecordingIntentResponse> {
  return ApiClient.instance.post<YandexTelemostRecordingIntentResponse>(
    '/api/v1/integrations/yandex-telemost/recording/intent',
    request,
    'Yandex Telemost recording intent request failed'
  )
}

export async function openYandexTelemostCompanion(
  request: YandexTelemostCompanionOpenRequest
): Promise<YandexTelemostCompanionManifest> {
  return invoke<YandexTelemostCompanionManifest>('open_yandex_telemost_companion', { request })
}

export async function prepareYandexTelemostAudioDevice(request: {
  device_name?: string | null
}): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>('yandex_telemost_prepare_audio_device', { request })
}

export async function startYandexTelemostRecording(request: {
  account_id: string
  join_url: string
  conference_id?: string | null
  window_label?: string | null
  audio_input?: string | null
  consent_attested: boolean
}): Promise<YandexTelemostRecordingSession> {
  return invoke<YandexTelemostRecordingSession>('yandex_telemost_recording_start', { request })
}

export async function stopYandexTelemostRecording(
  recordingSessionId: string
): Promise<YandexTelemostRecordingStopReceipt> {
  return invoke<YandexTelemostRecordingStopReceipt>('yandex_telemost_recording_stop', {
    request: { recording_session_id: recordingSessionId },
  })
}

export async function completeYandexTelemostRecording(
  request: YandexTelemostRecordingBridgeRequest
): Promise<YandexTelemostRecordingBridgeResponse> {
  return ApiClient.instance.post<YandexTelemostRecordingBridgeResponse>(
    '/api/v1/integrations/yandex-telemost/runtime-bridge/recordings',
    request,
    'Yandex Telemost recording completion bridge failed'
  )
}
