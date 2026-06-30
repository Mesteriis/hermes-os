import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('TelegramAccountManager boundary', () => {
  it('removes the account manager Vue layer while preserving account lifecycle mutations in TS', () => {
    const querySource = readFileSync(new URL('../queries/useTelegramQuery.ts', import.meta.url), 'utf8')
    const mutationsSource = readFileSync(new URL('../queries/useTelegramMutations.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./TelegramAccountManager.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./TelegramQrLoginPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../queries/useTelegramQrLoginQuery.ts', import.meta.url))).toBe(false)

    expect(querySource).toContain('useTelegramAccountsQuery')
    expect(mutationsSource).toContain('useLogoutTelegramAccountMutation')
    expect(mutationsSource).toContain('useRemoveTelegramAccountMutation')
    expect(mutationsSource).not.toContain('useStartTelegramQrLoginMutation')
    expect(mutationsSource).not.toContain('useSubmitTelegramQrPasswordMutation')
    expect(mutationsSource).not.toContain('useCancelTelegramQrLoginMutation')
  })
})
