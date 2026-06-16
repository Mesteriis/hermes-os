import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramComposer dry-run boundary', () => {
  it('embeds the dry-run panel and exposes selected chat scope props for it', () => {
    const source = readFileSync(new URL('./TelegramComposer.vue', import.meta.url), 'utf8')

    expect(source).toContain('TelegramSendDryRunPanel')
    expect(source).toContain('selectedAccountId: string | null')
    expect(source).toContain('selectedProviderChatId: string | null')
    expect(source).toContain(':accountId="selectedAccountId"')
    expect(source).toContain(':providerChatId="selectedProviderChatId"')
  })
})
