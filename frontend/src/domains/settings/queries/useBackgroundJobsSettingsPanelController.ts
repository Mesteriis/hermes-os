import type { BackgroundJobsSettingsSurface } from './useBackgroundJobsSettingsSurface'

export function useBackgroundJobsSettingsPanelController(
  options: { surface: BackgroundJobsSettingsSurface },
) {
  const surface = options.surface

  function handleRefresh(): void {
    surface.handleRefresh()
  }

  function handleSelectJobFilter(tabId: string): void {
    surface.handleSelectJobFilter(tabId)
  }

  function handleOpenControl(controlSection: string): void {
    surface.handleOpenControl(controlSection)
  }

  return {
    handleRefresh,
    handleSelectJobFilter,
    handleOpenControl,
  }
}
