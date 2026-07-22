import { describe, expect, it, vi } from 'vitest'
import { syncAiProviderModels, toggleAiProvider } from './aiProviderActions'
import type { AiProviderAccount } from '../types/aiControlCenter'

describe('ai provider actions', () => {
  it('updates provider state and emits the enabled message', async () => {
    const dependencies = {
      updateProvider: vi.fn().mockResolvedValue({ ...provider(), provider_id: 'updated' }),
      setSelectedProvider: vi.fn(),
      setActionMessage: vi.fn(),
      setError: vi.fn(),
      t: (key: string) => key
    }

    await toggleAiProvider(provider(), true, dependencies)

    expect(dependencies.updateProvider).toHaveBeenCalledWith({
      providerId: 'provider-1', request: { enabled: true }
    })
    expect(dependencies.setSelectedProvider).toHaveBeenCalledWith('updated')
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('AI provider enabled')
  })

  it('refreshes overview after model synchronization', async () => {
    const dependencies = {
      execute: vi.fn().mockResolvedValue({ message: 'synced' }),
      refreshOverview: vi.fn().mockResolvedValue(undefined),
      setActionMessage: vi.fn(),
      setError: vi.fn(),
      t: (key: string) => key
    }

    await syncAiProviderModels(provider(), dependencies)

    expect(dependencies.execute).toHaveBeenCalledWith('provider-1')
    expect(dependencies.refreshOverview).toHaveBeenCalledOnce()
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('synced')
  })
})

function provider(): AiProviderAccount {
  return {
    provider_id: 'provider-1', provider_kind: 'api', provider_key: 'provider',
    display_name: 'Provider', status: 'ready', consent_state: 'granted', config: {},
    capabilities: [], created_at: '', updated_at: ''
  }
}
