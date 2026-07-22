import { describe, expect, it } from 'vitest'
import { isSupportedLocale, languageLocaleOptions } from './languageSettingsPresentation'

describe('language settings presentation', () => {
  it('exposes the supported locale choices', () => {
    expect(languageLocaleOptions.map((option) => option.value)).toEqual(['en', 'ru'])
    expect(isSupportedLocale('en')).toBe(true)
    expect(isSupportedLocale('de')).toBe(false)
  })
})
