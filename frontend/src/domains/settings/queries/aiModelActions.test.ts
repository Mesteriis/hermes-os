import { describe, expect, it, vi } from 'vitest'
import { updateAiModelAvailability } from './aiModelActions'
import type { AiModelCatalogItem } from '../types/aiControlCenter'

describe('ai model actions', () => {
  it('refreshes the model overview after availability changes', async () => {
    const dependencies = {
      updateAvailability: vi.fn().mockResolvedValue(model()),
      refreshOverview: vi.fn().mockResolvedValue(undefined),
      setActionMessage: vi.fn(),
      setError: vi.fn(),
      t: (key: string) => key
    }

    await updateAiModelAvailability(model(), false, dependencies)

    expect(dependencies.updateAvailability).toHaveBeenCalledWith({
      provider_id: 'provider-1', model_key: 'model-1', is_available: false
    })
    expect(dependencies.refreshOverview).toHaveBeenCalledOnce()
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('AI model disabled')
  })
})

function model(): AiModelCatalogItem {
  return {
    provider_id: 'provider-1', model_key: 'model-1', display_name: 'Model',
    category: 'chat', privacy: 'local', capabilities: ['chat'], is_available: true,
    metadata: {}, created_at: '', updated_at: ''
  }
}
