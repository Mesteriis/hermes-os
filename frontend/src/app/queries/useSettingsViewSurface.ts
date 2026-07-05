import { useSettingsSurface } from '../../domains/settings/queries/useSettingsSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useSettingsViewSurface() {
  const settings = useSettingsSurface()

  return createPlannedScreenSurface({
    screenId: 'settings',
    titleKey: 'Settings',
    descriptionKey: 'Settings and capability control surface.',
    preservedLogicKey: 'Settings logic is active',
    detailKey: 'Settings route renders the domain SettingsPage, which owns settings orchestration through useSettingsPageSurface.',
    status: settings.status,
    ownerLayer: 'domain',
    surfacePath: settings.surfacePath,
    childSurfaces: settings.childSurfaces
  })
}
