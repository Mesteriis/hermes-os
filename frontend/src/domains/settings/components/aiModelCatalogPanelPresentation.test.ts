import { describe, expect, it } from 'vitest'
import {
  buildProviderModelGroups,
  filterProviderModels,
  findSelectedProviderModelGroup
} from './aiModelCatalogPanelPresentation'
import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'

describe('ai model catalog panel presentation', () => {
  it('groups models by provider and counts available models', () => {
    const provider = account('provider-1')
    const groups = buildProviderModelGroups(
      [provider], [model('provider-1', 'Chat', true), model('provider-1', 'Embed', false)]
    )

    expect(groups[0]?.models).toHaveLength(2)
    expect(groups[0]?.availableCount).toBe(1)
    expect(findSelectedProviderModelGroup(groups, 'missing')).toBe(groups[0])
  })

  it('filters by availability and provider/model search', () => {
    const provider = account('provider-1')
    const group = buildProviderModelGroups(
      [provider], [model('provider-1', 'Chat Model', true), model('provider-1', 'Embed Model', false)]
    )[0]

    expect(filterProviderModels(group ?? null, 'chat-model', false)).toHaveLength(1)
    expect(filterProviderModels(group ?? null, '', true)).toHaveLength(1)
  })
})

function account(providerId: string): AiProviderAccount {
  return {
    provider_id: providerId, provider_kind: 'api', provider_key: 'openai',
    display_name: 'OpenAI', status: 'ready', consent_state: 'granted', config: {},
    capabilities: [], created_at: '', updated_at: ''
  }
}

function model(providerId: string, displayName: string, isAvailable: boolean): AiModelCatalogItem {
  return {
    provider_id: providerId, model_key: displayName.toLowerCase().replaceAll(' ', '-'),
    display_name: displayName, category: 'chat', privacy: 'remote', capabilities: ['chat'],
    is_available: isAvailable, metadata: {}, created_at: '', updated_at: ''
  }
}
