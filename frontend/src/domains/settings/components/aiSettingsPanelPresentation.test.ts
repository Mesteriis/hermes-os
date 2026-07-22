import { describe, expect, it } from 'vitest'
import {
  buildAiProviderListGroups,
  buildAiSelectedProviderRows,
  countProviderRoutes
} from './aiSettingsPanelPresentation'
import { countAvailableModels } from './aiModelCatalogPresentation'
import type {
  AiModelCatalogItem,
  AiModelRoute,
  AiProviderAccount,
  AiProviderPreset
} from '../types/aiControlCenter'

describe('ai settings panel presentation', () => {
  it('groups connected providers and unconfigured presets without duplicates', () => {
    const provider = account('provider-1', 'built_in', 'ollama')
    const preset = { ...presetFor('ollama'), provider_kind: 'built_in' }
    const groups = buildAiProviderListGroups(
      [provider], [preset], [model('provider-1', true)],
      (candidate) => candidate.provider_key === 'ollama' ? provider : null,
      (key) => key
    )

    expect(groups.find((group) => group.id === 'local')?.items).toHaveLength(1)
    expect(groups.find((group) => group.id === 'local')?.items[0]?.provider).toBe(provider)
  })

  it('builds selected details and counts as pure view data', () => {
    const provider = account('provider-1', 'api', 'openai')
    const routes: AiModelRoute[] = [
      { capability_slot: 'chat', provider_id: 'provider-1', model_key: 'model-1', created_at: '', updated_at: '' }
    ]
    expect(buildAiSelectedProviderRows(provider, () => 'https://example.test', (key) => key)).toHaveLength(5)
    expect(countAvailableModels([model('provider-1', true), model('provider-1', false)])).toBe(1)
    expect(countProviderRoutes('provider-1', routes)).toBe(1)
  })
})

function account(id: string, kind: string, key: string): AiProviderAccount {
  return {
    provider_id: id, provider_kind: kind, provider_key: key, display_name: key,
    status: 'ready', consent_state: 'granted', config: {}, capabilities: [],
    created_at: '', updated_at: ''
  }
}

function presetFor(key: string): AiProviderPreset {
  return { provider_kind: 'cli', provider_key: key, display_name: key, privacy: 'local', capabilities: [] }
}

function model(providerId: string, isAvailable: boolean): AiModelCatalogItem {
  return {
    provider_id: providerId, model_key: 'model-1', display_name: 'Model', category: 'chat',
    privacy: 'local', capabilities: ['chat'], is_available: isAvailable,
    metadata: {}, created_at: '', updated_at: ''
  }
}
