import type { BackgroundBrightness, PanelBlur, PanelOpacity } from '../../../platform/theme/settings'

export function pickAllowedThemeNumber<T extends BackgroundBrightness | PanelOpacity | PanelBlur>(
  value: number,
  allowed: readonly T[]
): T | null {
  return allowed.includes(value as T) ? (value as T) : null
}
