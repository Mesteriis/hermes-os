import { describe, expect, it } from 'vitest'
import { aiProviderListSelection } from './aiSettingsPanelPresentation'
import type { AiProviderPreset } from '../types/aiControlCenter'

describe('AI provider list selection', () => {
  it('normalizes provider and preset items into explicit targets', () => {
    const provider = { provider_id: 'provider-1' }
    const providerItem = {
      provider,
      preset: null,
    } satisfies Parameters<typeof aiProviderListSelection>[0]
    const preset = {
      provider_kind: 'built_in',
      provider_key: 'ollama',
      display_name: 'Ollama',
      privacy: 'local',
      capabilities: ['chat'],
    } satisfies AiProviderPreset
    const presetItem = { provider: null, preset }

    expect(aiProviderListSelection(providerItem)).toEqual({ kind: 'provider', providerId: 'provider-1' })
    expect(aiProviderListSelection(presetItem)).toEqual({ kind: 'preset', preset })
  })

  it('returns null for an incomplete item', () => {
    expect(aiProviderListSelection({ provider: null, preset: null })).toBeNull()
  })
})
