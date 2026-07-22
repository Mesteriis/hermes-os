import { describe, expect, it, vi } from 'vitest'
import { downloadAiModel } from './aiModelDownloadActions'
import type { AiModelCatalogItem } from '../types/aiControlCenter'

describe('ai model download actions', () => {
  it('owns download ordering while delegating progress lifecycle', async () => {
    const dependencies = dependenciesFor()

    await downloadAiModel(model(), 'provider-1:model-1', dependencies)

    expect(dependencies.startProgress).toHaveBeenCalledWith('provider-1:model-1')
    expect(dependencies.download).toHaveBeenCalledWith({ provider_id: 'provider-1', model_key: 'model-1' })
    expect(dependencies.finishProgress).toHaveBeenCalledWith('provider-1:model-1')
    expect(dependencies.refreshOverview).toHaveBeenCalledOnce()
  })

  it('does not start a duplicate download', async () => {
    const dependencies = dependenciesFor()
    dependencies.isDownloading.mockReturnValue(true)

    const result = await downloadAiModel(model(), 'provider-1:model-1', dependencies)

    expect(result).toBeNull()
    expect(dependencies.startProgress).not.toHaveBeenCalled()
    expect(dependencies.download).not.toHaveBeenCalled()
  })
})

function dependenciesFor() {
  return {
    isDownloading: vi.fn().mockReturnValue(false),
    startProgress: vi.fn(),
    finishProgress: vi.fn(),
    clearProgress: vi.fn(),
    download: vi.fn().mockResolvedValue(model()),
    refreshOverview: vi.fn().mockResolvedValue(undefined),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    t: (key: string) => key
  }
}

function model(): AiModelCatalogItem {
  return {
    provider_id: 'provider-1', model_key: 'model-1', display_name: 'Model',
    category: 'chat', privacy: 'local', capabilities: ['chat'], is_available: true,
    metadata: {}, created_at: '', updated_at: ''
  }
}
