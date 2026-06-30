import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('TelegramCapabilityMatrix boundary', () => {
  it('removes the capability matrix Vue layer while preserving account capability queries in TS', () => {
    const querySource = readFileSync(new URL('../queries/useTelegramQuery.ts', import.meta.url), 'utf8')
    const apiSource = readFileSync(new URL('../api/telegram.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./TelegramCapabilityMatrix.vue', import.meta.url))).toBe(false)
    expect(querySource).toContain('useTelegramAccountCapabilitiesQuery')
    expect(querySource).toContain('fetchTelegramAccountCapabilities')
    expect(querySource).toContain('telegramQueryKeys.accountCapabilities')
    expect(apiSource).toContain('fetchTelegramAccountCapabilities')
    expect(querySource).not.toContain('fetch(')
  })
})
