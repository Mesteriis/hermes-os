import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramThreadHeader mention boundary', () => {
  it('surfaces unread and mention summary chips from projected chat metadata', () => {
    const source = readFileSync(new URL('./TelegramThreadHeader.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramChatMentionCountValue')
    expect(source).toContain('telegram-thread-stats')
    expect(source).toContain("t('mentions')")
    expect(source).toContain("t('unread')")
    expect(source).not.toContain('fetch(')
  })
})
