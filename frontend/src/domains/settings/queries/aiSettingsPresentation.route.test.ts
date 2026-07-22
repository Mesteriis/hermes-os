import { describe, expect, it } from 'vitest'
import {
  buildAiModelRouteRows,
  modelDownloadProgressLabel,
  modelRouteUsageCount,
  providerBaseUrl
} from './aiSettingsPresentation'
import type { AiCapabilitySlot, AiModelCatalogItem, AiModelRoute, AiProviderAccount } from '../types/aiControlCenter'

describe('ai settings presentation', () => {
  it('builds route options only for usable models', () => {
    const slot = capabilitySlot('chat')
    const modelItem = model('chat-model', ['chat'])
    const rows = buildAiModelRouteRows([slot], [], [provider()], [modelItem], (key) => key)

    expect(rows[0]?.selectedModelLabel).toBe('Not routed')
    expect(rows[0]?.options).toHaveLength(1)
  })

  it('formats pure model metadata and progress labels', () => {
    const routes: AiModelRoute[] = [{
      capability_slot: 'chat', provider_id: 'provider-1', model_key: 'chat-model',
      created_at: '', updated_at: ''
    }]
    expect(modelRouteUsageCount(model('chat-model', ['chat']), routes)).toBe(1)
    expect(providerBaseUrl(provider())).toBe('https://example.test')
    expect(modelDownloadProgressLabel(72, (key) => key)).toBe('Preparing model for routing')
  })
})

function capabilitySlot(slot: string): AiCapabilitySlot {
  return { slot, label: slot, description: slot }
}

function model(modelKey: string, capabilities: string[]): AiModelCatalogItem {
  return {
    provider_id: 'provider-1', model_key: modelKey, display_name: modelKey,
    category: 'chat', privacy: 'local', capabilities, is_available: true,
    metadata: {}, created_at: '', updated_at: ''
  }
}

function provider(): AiProviderAccount {
  return {
    provider_id: 'provider-1', provider_kind: 'api', provider_key: 'provider',
    display_name: 'Provider', status: 'ready', consent_state: 'granted',
    config: { base_url: 'https://example.test' }, capabilities: [],
    created_at: '', updated_at: ''
  }
}
