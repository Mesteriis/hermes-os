import { describe, expect, it } from 'vitest'
import { pickAllowedThemeNumber } from './appearanceSettingsPredicates'

describe('appearance settings predicates', () => {
  it('accepts values from the theme allowlist', () => {
    expect(pickAllowedThemeNumber(80, [40, 80, 100] as const)).toBe(80)
  })

  it('rejects values outside the theme allowlist', () => {
    expect(pickAllowedThemeNumber(70, [40, 80, 100] as const)).toBeNull()
  })
})
