import { describe, expect, it, vi } from 'vitest'
import { saveSidebarSettings } from './sidebarSettingsActions'

describe('sidebar settings actions', () => {
  it('applies the saved settings and emits success', async () => {
    const dependencies = dependenciesFor()

    await saveSidebarSettings({ groups: [] }, dependencies)

    expect(dependencies.saveSidebarSettings).toHaveBeenCalledWith({ groups: [] })
    expect(dependencies.applySidebarSettings).toHaveBeenCalledOnce()
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Sidebar saved')
    expect(dependencies.setSaving).toHaveBeenLastCalledWith(false)
  })

  it('reports failure and always clears saving state', async () => {
    const dependencies = dependenciesFor()
    dependencies.saveSidebarSettings.mockRejectedValue(new Error('save failed'))

    await saveSidebarSettings({ groups: [] }, dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith('save failed')
    expect(dependencies.applySidebarSettings).not.toHaveBeenCalled()
    expect(dependencies.setSaving).toHaveBeenLastCalledWith(false)
  })
})

function dependenciesFor() {
  return {
    saveSidebarSettings: vi.fn().mockResolvedValue(undefined),
    applySidebarSettings: vi.fn(),
    setSaving: vi.fn(),
    clearError: vi.fn(),
    setError: vi.fn(),
    setActionMessage: vi.fn(),
    savingMessage: 'Sidebar saved',
    errorMessage: (error: unknown) => error instanceof Error ? error.message : 'save failed'
  }
}
