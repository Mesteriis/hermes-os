import type { MaintenanceSettingsSurface } from './useMaintenanceSettingsSurface'

export function useMaintenanceSettingsPanelController(options: { surface: MaintenanceSettingsSurface }) {
  const surface = options.surface

  function eventValue(event: Event): string {
    return event.target instanceof HTMLInputElement ? event.target.value : ''
  }

  function handleRefresh() {
    surface.handleRefresh()
  }

  function handleConfirmationInput(event: Event): void {
    surface.handleConfirmationInput(eventValue(event))
  }

  return {
    handleRefresh,
    handleSelectAction: surface.handleSelectAction,
    handleConfirmationInput,
    handleRunSelectedAction: surface.handleRunSelectedAction,
  }
}
