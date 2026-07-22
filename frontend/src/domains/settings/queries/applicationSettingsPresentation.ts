import type { ApplicationSetting } from '../types/settings'

export function settingDraftValue(
  setting: ApplicationSetting,
  drafts: Record<string, string>
): string {
  const draft = drafts[setting.setting_key]
  if (draft !== undefined) return draft
  return String(setting.value ?? '')
}

export function settingControlType(setting: ApplicationSetting): string {
  const allowedValues = setting.metadata?.allowed_values
  if (Array.isArray(allowedValues) && allowedValues.length > 0) return 'select'
  if (setting.value_kind === 'boolean') return 'checkbox'
  if (setting.value_kind === 'integer') return 'number'
  return 'text'
}

export function settingAllowedValues(setting: ApplicationSetting): string[] {
  const values = setting.metadata?.allowed_values
  return Array.isArray(values) ? values.map(String) : []
}

export function settingMetadataFlag(setting: ApplicationSetting, key: string): boolean {
  return Boolean(setting.metadata?.[key])
}

export function settingMetadataText(setting: ApplicationSetting, key: string): string {
  const value = setting.metadata?.[key]
  return typeof value === 'string' ? value : ''
}

export function categoryLabel(category: string): string {
  const labels: Record<string, string> = {
    general: 'General',
    frontend: 'Interface',
    ai: 'AI',
    privacy: 'Privacy',
    notifications: 'Notifications'
  }
  return labels[category] ?? category
}
