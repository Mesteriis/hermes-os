import type { AiProviderAuthStartRequest, AiProviderPreset } from '../types/aiControlCenter'

const AI_PROVIDER_CALLBACK_PATH = '/api/v1/ai/provider-auth/callback'

export function buildAiProviderCallbackUrl(apiBaseUrl: string): string {
  return `${apiBaseUrl.replace(/\/+$/, '')}${AI_PROVIDER_CALLBACK_PATH}`
}

export function buildAiProviderAuthStartRequest(
  preset: AiProviderPreset,
  callbackUrl: string
): AiProviderAuthStartRequest {
  return {
    provider_kind: preset.provider_kind,
    provider_key: preset.provider_key,
    display_name: preset.display_name,
    callback_url: callbackUrl
  }
}
