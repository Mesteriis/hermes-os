import { describe, expect, it, vi } from 'vitest'
import { removeSignalHubProfile, saveSignalHubProfile } from './signalHubProfileActions'

describe('signal hub profile actions', () => {
  it('updates an existing profile instead of creating a duplicate', async () => {
    const dependencies = dependenciesFor()
    const request = { display_name: 'Updated', description: 'Description', source_policies: [] }

    await saveSignalHubProfile('profile-1', 'ignored', request, dependencies)

    expect(dependencies.update).toHaveBeenCalledWith({ profileCode: 'profile-1', request })
    expect(dependencies.create).not.toHaveBeenCalled()
  })

  it('resets the editor only after profile removal succeeds', async () => {
    const dependencies = dependenciesFor()

    await removeSignalHubProfile('profile-1', dependencies)

    expect(dependencies.remove).toHaveBeenCalledWith('profile-1')
    expect(dependencies.resetEditor).toHaveBeenCalledOnce()
  })
})

function dependenciesFor() {
  return {
    update: vi.fn().mockResolvedValue(undefined),
    create: vi.fn().mockResolvedValue(undefined),
    remove: vi.fn().mockResolvedValue(undefined),
    resetEditor: vi.fn()
  }
}
