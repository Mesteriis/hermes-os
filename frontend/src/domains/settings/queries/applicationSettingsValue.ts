import type { ApplicationSetting } from '../types/settings'

export function coerceApplicationSettingValue(
  draft: string,
  kind: ApplicationSetting['value_kind']
): ApplicationSetting['value'] {
  switch (kind) {
    case 'boolean':
      return draft === 'true'
    case 'integer':
      return parseInt(draft, 10) || 0
    case 'json':
      try {
        return JSON.parse(draft)
      } catch {
        return draft
      }
    default:
      return draft
  }
}
