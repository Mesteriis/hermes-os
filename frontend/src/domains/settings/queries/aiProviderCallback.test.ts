import { describe, expect, it } from 'vitest'
import { parseAiProviderCallbackMessage } from './aiProviderCallback'

describe('ai provider callback parser', () => {
  it('accepts the provider-connected protocol message', () => {
    expect(parseAiProviderCallbackMessage({
      type: 'hermes:ai-provider-connected', providerId: 'provider-1'
    })).toEqual({ providerId: 'provider-1' })
  })

  it('preserves a valid message when provider id is absent', () => {
    expect(parseAiProviderCallbackMessage({ type: 'hermes:ai-provider-connected' }))
      .toEqual({ providerId: null })
    expect(parseAiProviderCallbackMessage({ type: 'other' })).toBeNull()
    expect(parseAiProviderCallbackMessage(null)).toBeNull()
  })
})
