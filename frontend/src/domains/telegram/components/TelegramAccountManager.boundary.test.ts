import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramAccountManager boundary', () => {
  it('uses vee-validate form state together with account lifecycle query mutations', () => {
    const source = readFileSync(new URL('./TelegramAccountManager.vue', import.meta.url), 'utf8')

    expect(source).toContain("from 'vee-validate'")
    expect(source).toContain('useTelegramAccountsQuery')
    expect(source).toContain('useSetupTelegramAccountMutation')
    expect(source).toContain('useLogoutTelegramAccountMutation')
    expect(source).toContain('useRemoveTelegramAccountMutation')
    expect(source).toContain('telegramAccountSetupSchema')
    expect(source).toContain('TelegramCapabilityMatrix')
    expect(source).toContain('TelegramQrLoginPanel')
    expect(source).toContain('setFieldValue')
    expect(source).toContain('props.selectedAccountId')
  })
})
