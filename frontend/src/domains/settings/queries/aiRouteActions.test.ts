import { describe, expect, it, vi } from 'vitest'
import { updateAiRouteSelection } from './aiRouteActions'

describe('ai route actions', () => {
  it('clears an empty route selection', async () => {
    const dependencies = dependenciesFor()

    await updateAiRouteSelection('chat', '', dependencies)

    expect(dependencies.deleteRoute).toHaveBeenCalledWith('chat')
    expect(dependencies.updateRoute).not.toHaveBeenCalled()
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('AI model route cleared')
  })

  it('updates a parsed route selection and refreshes overview', async () => {
    const dependencies = dependenciesFor()
    const value = `${encodeURIComponent('provider-1')}|${encodeURIComponent('model-1')}`

    await updateAiRouteSelection('chat', value, dependencies)

    expect(dependencies.updateRoute).toHaveBeenCalledWith({
      slot: 'chat', request: { provider_id: 'provider-1', model_key: 'model-1' }
    })
    expect(dependencies.refreshOverview).toHaveBeenCalledOnce()
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('AI model route updated')
  })
})

function dependenciesFor() {
  return {
    updateRoute: vi.fn().mockResolvedValue(undefined),
    deleteRoute: vi.fn().mockResolvedValue(undefined),
    refreshOverview: vi.fn().mockResolvedValue(undefined),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    t: (key: string) => key
  }
}
