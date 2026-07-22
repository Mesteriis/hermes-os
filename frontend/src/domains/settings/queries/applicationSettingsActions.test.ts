import { describe, expect, it, vi } from 'vitest'
import { saveApplicationSettingValue } from './applicationSettingsActions'

describe('application settings actions', () => {
  it('persists a value and clears saving state after success', async () => {
    const dependencies = dependenciesFor()

    await saveApplicationSettingValue('setting.key', 'Workspace', true, dependencies)

    expect(dependencies.save).toHaveBeenCalledWith({ settingKey: 'setting.key', value: true })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Saved Workspace')
    expect(dependencies.setSavingKey).toHaveBeenLastCalledWith(null)
  })

  it('reports failure and still clears saving state', async () => {
    const dependencies = dependenciesFor()
    dependencies.save.mockRejectedValue(new Error('failed'))

    await saveApplicationSettingValue('setting.key', 'Workspace', 'value', dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith('failed')
    expect(dependencies.setSavingKey).toHaveBeenLastCalledWith(null)
  })
})

function dependenciesFor() {
  return {
    save: vi.fn().mockResolvedValue(undefined),
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    setSavingKey: vi.fn()
  }
}
