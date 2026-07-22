import { describe, expect, it } from 'vitest'
import { buildAiApiProviderCreateRequest } from './aiProviderRequests'

describe('ai provider requests', () => {
  it('builds the OpenAI-compatible provider contract', () => {
    expect(buildAiApiProviderCreateRequest({
      providerKey: 'openai', displayName: 'OpenAI', baseUrl: 'https://api.openai.com',
      token: 'secret', remoteContextConsent: true
    })).toEqual({
      provider_kind: 'api', provider_key: 'openai', display_name: 'OpenAI',
      base_url: 'https://api.openai.com',
      capabilities: ['chat', 'reasoning', 'summarization', 'embeddings'],
      enabled: true, remote_context_consent: true, api_key: 'secret'
    })
  })
})
