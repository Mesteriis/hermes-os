import { describe, expect, it, vi } from 'vitest'
import {
  canRunMaintenanceAction,
  runSelectedMaintenanceAction
} from './maintenanceSettingsActions'
import type { MaintenanceActionDescriptor } from '../types/maintenance'

describe('maintenance settings actions', () => {
  it('requires the exact confirmation phrase for guarded actions', () => {
    const action = maintenanceAction({ requires_confirmation: true, confirmation_phrase: 'CLEAN' })

    expect(canRunMaintenanceAction(action, 'clean', false)).toBe(false)
    expect(canRunMaintenanceAction(action, 'CLEAN', false)).toBe(true)
    expect(canRunMaintenanceAction(action, 'CLEAN', true)).toBe(false)
  })

  it('runs an allowed action and clears confirmation after success', async () => {
    const dependencies = dependenciesFor()

    await runSelectedMaintenanceAction(
      maintenanceAction({ requires_confirmation: false }),
      '',
      true,
      dependencies
    )

    expect(dependencies.runAction).toHaveBeenCalledWith({
      actionId: 'cleanup',
      request: { confirmation: undefined }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('completed')
    expect(dependencies.clearConfirmation).toHaveBeenCalledOnce()
  })
})

function maintenanceAction(
  overrides: Partial<MaintenanceActionDescriptor> = {}
): MaintenanceActionDescriptor {
  return {
    id: 'cleanup',
    label: 'Cleanup',
    description: 'Cleanup',
    icon: 'tabler:trash',
    destructive: true,
    enabled: true,
    requires_confirmation: false,
    confirmation_phrase: null,
    disabled_reason: null,
    ...overrides
  }
}

function dependenciesFor() {
  return {
    runAction: vi.fn().mockResolvedValue({ message: 'completed' }),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    clearConfirmation: vi.fn()
  }
}
