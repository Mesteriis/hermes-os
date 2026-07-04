import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useSettingsViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'settings',
    titleKey: 'Settings',
    descriptionKey: 'Settings and capability control surface.',
    preservedLogicKey: 'Settings logic is active',
    detailKey: 'Settings route renders the domain SettingsPage, which owns settings orchestration through useSettingsPageSurface.',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/settings/queries/useSettingsPageSurface.ts'
  })
}
