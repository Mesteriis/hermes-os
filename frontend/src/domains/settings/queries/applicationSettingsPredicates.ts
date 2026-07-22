import type { ApplicationSetting } from '../types/settings'

export function isPublicApplicationSetting(setting: ApplicationSetting): boolean {
  return (
    setting.category !== 'ai' &&
    setting.category !== 'communications' &&
    !setting.setting_key.startsWith('ai.')
  )
}

export function settingHasChanged(
  setting: ApplicationSetting,
  drafts: Record<string, string>
): boolean {
  const draft = drafts[setting.setting_key]
  if (draft === undefined) return false
  return draft !== String(setting.value)
}
