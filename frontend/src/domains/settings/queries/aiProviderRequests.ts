import type { AiProviderCreateRequest } from '../types/aiControlCenter'

export function buildAiApiProviderCreateRequest(input: {
  providerKey: string
  displayName: string
  baseUrl: string
  token: string
  remoteContextConsent: boolean
}): AiProviderCreateRequest {
  return {
    provider_kind: 'api',
    provider_key: input.providerKey,
    display_name: input.displayName,
    base_url: input.baseUrl,
    capabilities: ['chat', 'reasoning', 'summarization', 'embeddings'],
    enabled: true,
    remote_context_consent: input.remoteContextConsent,
    api_key: input.token
  }
}
