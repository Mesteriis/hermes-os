import type {
  MaintenanceActionDescriptor,
  MaintenanceActionRequest,
  MaintenanceActionResponse
} from '../types/maintenance'

export function canRunMaintenanceAction(
  action: MaintenanceActionDescriptor | null,
  confirmation: string,
  isBusy: boolean
): boolean {
  if (!action?.enabled || isBusy) return false
  if (!action.requires_confirmation) return true
  return confirmation === action.confirmation_phrase
}

interface RunMaintenanceActionDependencies {
  runAction: (variables: {
    actionId: string
    request: MaintenanceActionRequest
  }) => Promise<MaintenanceActionResponse>
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  clearConfirmation: () => void
}

export async function runSelectedMaintenanceAction(
  action: MaintenanceActionDescriptor | null,
  confirmation: string,
  isAllowed: boolean,
  dependencies: RunMaintenanceActionDependencies
): Promise<void> {
  if (!action || !isAllowed) return
  try {
    const result = await dependencies.runAction({
      actionId: action.id,
      request: {
        confirmation: action.requires_confirmation ? confirmation : undefined
      }
    })
    dependencies.setActionMessage(result.message)
    dependencies.clearConfirmation()
  } catch (error) {
    dependencies.setError(error instanceof Error ? error.message : 'Maintenance action failed')
  }
}
