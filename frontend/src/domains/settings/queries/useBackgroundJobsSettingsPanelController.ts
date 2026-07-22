import type { BackgroundJobFilter } from '../components/backgroundJobsPresentation'
import type { SettingsSection } from '../stores/settings'
import type { BackgroundJobsSettingsSurface } from './useBackgroundJobsSettingsSurface'

export function useBackgroundJobsSettingsPanelController(
  options: { surface: BackgroundJobsSettingsSurface },
) {
  const surface = options.surface

  function handleRefresh(): void {
    surface.handleRefresh()
  }

  function handleSelectJobFilter(tabId: BackgroundJobFilter): void {
    surface.handleSelectJobFilter(tabId)
  }

  function handleOpenControl(controlSection: SettingsSection): void {
    surface.handleOpenControl(controlSection)
  }

  return {
    handleRefresh,
    handleSelectJobFilter,
    handleOpenControl,
  }
}
