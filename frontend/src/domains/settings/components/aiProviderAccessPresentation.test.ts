import { describe, expect, it } from 'vitest'
import {
  hasAiRemoteContextConsent,
  isAiProviderEnabled,
  supportsAiRemoteContext,
} from './aiProviderAccessPresentation'

describe('AI provider access presentation', () => {
  it('derives enabled state from provider status', () => {
    expect(isAiProviderEnabled({ status: 'ready' })).toBe(true)
    expect(isAiProviderEnabled({ status: 'disabled' })).toBe(false)
  })

  it('exposes remote context only for API providers', () => {
    expect(supportsAiRemoteContext({ provider_kind: 'api' })).toBe(true)
    expect(supportsAiRemoteContext({ provider_kind: 'cli' })).toBe(false)
  })

  it('derives consent from the granted state', () => {
    expect(hasAiRemoteContextConsent({ consent_state: 'granted' })).toBe(true)
    expect(hasAiRemoteContextConsent({ consent_state: 'pending' })).toBe(false)
  })
})
