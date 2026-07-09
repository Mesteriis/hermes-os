import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  AiModelAvailabilityUpdateRequest,
  AiHubUsageStatsResponse,
  AiModelCatalogItem,
  AiModelDownloadRequest,
  AiModelRoute,
  AiModelRouteUpdateRequest,
  AiProviderAuthStartRequest,
  AiProviderAuthStartResponse,
  AiProviderAuthStatusResponse,
  AiProviderAccount,
  AiProviderCommandResponse,
  AiProviderConsentRequest,
  AiProviderCreateRequest,
  AiProviderPatchRequest,
  AiSettingsOverviewResponse,
} from '../types/aiControlCenter'

export async function fetchAiSettingsOverview(): Promise<AiSettingsOverviewResponse> {
  return ApiClient.instance.get<AiSettingsOverviewResponse>(
    '/api/v1/ai/settings/overview',
    'AI settings overview request failed'
  )
}

export async function fetchAiHubUsageStats(
  windowHours = 24
): Promise<AiHubUsageStatsResponse> {
  return ApiClient.instance.get<AiHubUsageStatsResponse>(
    `/api/v1/ai/usage/stats?window_hours=${encodeURIComponent(String(windowHours))}`,
    'AI Hub usage stats request failed'
  )
}

export async function fetchAiModels(): Promise<{ items: AiModelCatalogItem[] }> {
  return ApiClient.instance.get<{ items: AiModelCatalogItem[] }>(
    '/api/v1/ai/models',
    'AI models request failed'
  )
}

export async function updateAiModelAvailability(
  request: AiModelAvailabilityUpdateRequest
): Promise<AiModelCatalogItem> {
  return ApiClient.instance.patch<AiModelCatalogItem>(
    '/api/v1/ai/models/availability',
    request,
    'AI model availability update failed'
  )
}

export async function downloadAiModel(
  request: AiModelDownloadRequest
): Promise<AiModelCatalogItem> {
  return ApiClient.instance.post<AiModelCatalogItem>(
    '/api/v1/ai/model-downloads',
    request,
    'AI model download failed'
  )
}

export async function createAiProvider(
  request: AiProviderCreateRequest
): Promise<AiProviderAccount> {
  return ApiClient.instance.post<AiProviderAccount>(
    '/api/v1/ai/providers',
    request,
    'AI provider create failed'
  )
}

export async function updateAiProvider(
  providerId: string,
  request: AiProviderPatchRequest
): Promise<AiProviderAccount> {
  return ApiClient.instance.patch<AiProviderAccount>(
    `/api/v1/ai/providers/${encodeURIComponent(providerId)}`,
    request,
    'AI provider update failed'
  )
}

export async function testAiProvider(providerId: string): Promise<AiProviderCommandResponse> {
  return ApiClient.instance.post<AiProviderCommandResponse>(
    `/api/v1/ai/providers/${encodeURIComponent(providerId)}/test`,
    {},
    'AI provider test failed'
  )
}

export async function syncAiProviderModels(providerId: string): Promise<AiProviderCommandResponse> {
  return ApiClient.instance.post<AiProviderCommandResponse>(
    `/api/v1/ai/providers/${encodeURIComponent(providerId)}/sync-models`,
    {},
    'AI provider model sync failed'
  )
}

export async function updateAiProviderConsent(
  providerId: string,
  request: AiProviderConsentRequest
): Promise<AiProviderAccount> {
  return ApiClient.instance.post<AiProviderAccount>(
    `/api/v1/ai/providers/${encodeURIComponent(providerId)}/consent`,
    request,
    'AI provider consent update failed'
  )
}

export async function startAiProviderAuth(
  request: AiProviderAuthStartRequest
): Promise<AiProviderAuthStartResponse> {
  return ApiClient.instance.post<AiProviderAuthStartResponse>(
    '/api/v1/ai/provider-auth/start',
    request,
    'AI provider callback start failed'
  )
}

export async function fetchAiProviderAuthStatus(
  setupId: string
): Promise<AiProviderAuthStatusResponse> {
  return ApiClient.instance.get<AiProviderAuthStatusResponse>(
    `/api/v1/ai/provider-auth/${encodeURIComponent(setupId)}`,
    'AI provider callback status failed'
  )
}

export async function updateAiModelRoute(
  slot: string,
  request: AiModelRouteUpdateRequest
): Promise<AiModelRoute> {
  return ApiClient.instance.put<AiModelRoute>(
    `/api/v1/ai/model-routes/${encodeURIComponent(slot)}`,
    request,
    'AI model route update failed'
  )
}

export async function deleteAiModelRoute(slot: string): Promise<void> {
  return ApiClient.instance.delete<void>(
    `/api/v1/ai/model-routes/${encodeURIComponent(slot)}`,
    'AI model route delete failed'
  )
}
