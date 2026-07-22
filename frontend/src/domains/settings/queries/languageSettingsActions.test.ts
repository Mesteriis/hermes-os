import { describe, expect, it, vi } from 'vitest'
import { saveLocaleWithRollback } from './languageSettingsActions'

describe('language settings actions', () => {
  it('rolls back the locale when persistence fails', async () => {
    const setLocale = vi.fn()
    const reportError = vi.fn()

    await saveLocaleWithRollback('ru', 'en', {
      setLocale,
      saveLocale: vi.fn().mockRejectedValue(new Error('save failed')),
      reportError
    })

    expect(setLocale).toHaveBeenNthCalledWith(1, 'ru')
    expect(setLocale).toHaveBeenNthCalledWith(2, 'en')
    expect(reportError).toHaveBeenCalledWith(expect.any(Error))
  })
})
