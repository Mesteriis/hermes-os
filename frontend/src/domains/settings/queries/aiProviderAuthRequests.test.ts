import { describe, expect, it } from 'vitest'
import {
  buildAiProviderAuthStartRequest,
  buildAiProviderCallbackUrl
} from './aiProviderAuthRequests'
import type { AiProviderPreset } from '../types/aiControlCenter'

describe('ai provider auth requests', () => {
  it('normalizes API base URL before adding callback path', () => {
    expect(buildAiProviderCallbackUrl('https://localhost:3000///'))
      .toBe('https://localhost:3000/api/v1/ai/provider-auth/callback')
  })

  it('maps provider preset into auth start contract', () => {
    const preset: AiProviderPreset = {
      provider_kind: 'cli', provider_key: 'ollama', display_name: 'Ollama',
      privacy: 'local', capabilities: []
    }
    expect(buildAiProviderAuthStartRequest(preset, 'https://localhost/callback')).toEqual({
      provider_kind: 'cli', provider_key: 'ollama', display_name: 'Ollama',
      callback_url: 'https://localhost/callback'
    })
  })
})
