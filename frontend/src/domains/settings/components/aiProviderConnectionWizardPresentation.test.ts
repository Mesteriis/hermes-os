import { describe, expect, it } from 'vitest'
import {
  connectedProviderModels,
  findConnectedProvider,
  wizardNextLabel,
} from './aiProviderConnectionWizardPresentation'
import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'

describe('ai provider connection wizard presentation', () => {
  it('selects the connected provider and only its models', () => {
    const first = provider('first')
    const second = provider('second')
    const models = [model('first', 'one'), model('second', 'two')]

    expect(findConnectedProvider([first, second], 'second')).toBe(second)
    expect(findConnectedProvider([first], 'missing')).toBeNull()
    expect(connectedProviderModels(models, 'first').map((item) => item.model_key)).toEqual(['one'])
    expect(connectedProviderModels(models, null)).toEqual([])
  })

  it('maps wizard steps to translated next labels', () => {
    const translate = (key: string) => `translated:${key}`

    expect(wizardNextLabel(1, translate)).toBe('translated:Подключить')
    expect(wizardNextLabel(2, translate)).toBe('translated:Проверить')
    expect(wizardNextLabel(3, translate)).toBe('translated:Готово')
  })
})

function provider(providerId: string): AiProviderAccount {
  return {
    provider_id: providerId, provider_kind: 'api', provider_key: providerId,
    display_name: providerId, status: 'ready', consent_state: 'granted', config: {},
    capabilities: [], created_at: '', updated_at: ''
  }
}

function model(providerId: string, modelKey: string): AiModelCatalogItem {
  return {
    provider_id: providerId, model_key: modelKey, display_name: modelKey,
    category: 'chat', privacy: 'remote', capabilities: ['chat'], is_available: true,
    metadata: {}, created_at: '', updated_at: ''
  }
}
