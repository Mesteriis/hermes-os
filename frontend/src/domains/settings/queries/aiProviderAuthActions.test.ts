import { describe, expect, it, vi } from 'vitest'
import { refreshAiProviderAuthStatus, startAiProviderAuth } from './aiProviderAuthActions'
import type { AiProviderPreset } from '../types/aiControlCenter'

describe('ai provider auth actions', () => {
  it('selects a provider and stops polling when auth completes immediately', async () => {
    const dependencies = dependenciesFor({ provider: { provider_id: 'provider-1' } })

    await startAiProviderAuth(preset(), dependencies)

    expect(dependencies.setActiveAuth).toHaveBeenCalledOnce()
    expect(dependencies.setSelectedProvider).toHaveBeenCalledWith('provider-1')
    expect(dependencies.stopPolling).toHaveBeenCalledOnce()
    expect(dependencies.startPolling).not.toHaveBeenCalled()
  })

  it('starts polling while callback auth is pending', async () => {
    const dependencies = dependenciesFor({ provider: null, setup_id: 'setup-1', message: 'waiting' })

    await startAiProviderAuth(preset(), dependencies)

    expect(dependencies.startPolling).toHaveBeenCalledWith('setup-1')
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('waiting')
    expect(dependencies.stopPolling).not.toHaveBeenCalled()
  })

  it('stops polling when a status refresh returns a provider', async () => {
    const dependencies = statusDependencies({ provider: { provider_id: 'provider-2' } })

    await refreshAiProviderAuthStatus('setup-1', false, dependencies)

    expect(dependencies.setSelectedProvider).toHaveBeenCalledWith('provider-2')
    expect(dependencies.stopPolling).toHaveBeenCalledOnce()
  })
})

function dependenciesFor(overrides: Record<string, unknown>) {
  const response = {
    setup_id: 'setup-1', provider_id: 'provider-1', provider_kind: 'cli',
    provider_key: 'ollama', display_name: 'Ollama', callback_url: 'https://localhost/callback',
    login_command: null, status: 'pending', message: 'waiting', expires_at: '',
    provider: null, ...overrides
  }
  return {
    startAuth: vi.fn().mockResolvedValue(response),
    setActiveAuth: vi.fn(),
    setSelectedProvider: vi.fn(),
    stopPolling: vi.fn(),
    startPolling: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    t: (key: string) => key
  }
}

function statusDependencies(overrides: Record<string, unknown>) {
  const response = {
    setup_id: 'setup-1', provider_id: 'provider-1', provider_kind: 'cli',
    provider_key: 'ollama', display_name: 'Ollama', callback_url: 'https://localhost/callback',
    login_command: null, status: 'pending', message: 'waiting', expires_at: '',
    provider: null, ...overrides
  }
  return {
    isPending: vi.fn().mockReturnValue(false),
    fetchStatus: vi.fn().mockResolvedValue(response),
    setActiveAuth: vi.fn(),
    setSelectedProvider: vi.fn(),
    stopPolling: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    t: (key: string) => key
  }
}

function preset(): AiProviderPreset {
  return {
    provider_kind: 'cli', provider_key: 'ollama', display_name: 'Ollama',
    privacy: 'local', capabilities: []
  }
}
