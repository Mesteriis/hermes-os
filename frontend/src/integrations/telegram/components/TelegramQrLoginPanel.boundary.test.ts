import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramQrLoginPanel boundary', () => {
  it('drives QR login through query/mutation hooks rather than inline fetch', () => {
    const source = readFileSync(
      resolve('src/integrations/telegram/components/TelegramQrLoginPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useStartTelegramQrLoginMutation')
    expect(source).toContain('useTelegramQrLoginStatusQuery')
    expect(source).toContain('useCancelTelegramQrLoginMutation')
    expect(source).toContain('useSubmitTelegramQrPasswordMutation')
    expect(source).toContain("t('Start QR')")
    expect(source).toContain("t('Apply Suggested Account')")
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })
})
