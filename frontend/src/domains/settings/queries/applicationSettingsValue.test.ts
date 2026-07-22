import { describe, expect, it } from 'vitest'
import { coerceApplicationSettingValue } from './applicationSettingsValue'

describe('application settings value coercion', () => {
  it('coerces supported setting value kinds', () => {
    expect(coerceApplicationSettingValue('true', 'boolean')).toBe(true)
    expect(coerceApplicationSettingValue('12', 'integer')).toBe(12)
    expect(coerceApplicationSettingValue('{"enabled":true}', 'json')).toEqual({ enabled: true })
    expect(coerceApplicationSettingValue('safe', 'string')).toBe('safe')
  })

  it('keeps invalid JSON as the entered draft', () => {
    expect(coerceApplicationSettingValue('{invalid', 'json')).toBe('{invalid')
  })
})
