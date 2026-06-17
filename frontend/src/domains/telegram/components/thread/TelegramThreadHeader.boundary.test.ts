import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramThreadHeader mention boundary', () => {
  it('surfaces saved-message, unread, mention and selected-chat realtime summary chips from projected metadata', () => {
    const source = readFileSync(new URL('./TelegramThreadHeader.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramChatMentionCountValue')
    expect(source).toContain('telegramChatTypingLabel')
    expect(source).toContain('telegramChatIsSavedMessages')
    expect(source).toContain("t('Saved Messages')")
    expect(source).toContain('telegram-thread-stats')
    expect(source).toContain("t('mentions')")
    expect(source).toContain("t('unread')")
    expect(source).toContain('syncStateMatchesChat')
    expect(source).toContain('commandStateMatchesChat')
    expect(source).toContain('telegram-thread-stat-sync')
    expect(source).toContain('telegram-thread-stat-command')
    expect(source).toContain('telegram-thread-stat-typing')
    expect(source).not.toContain('fetch(')
  })
})
