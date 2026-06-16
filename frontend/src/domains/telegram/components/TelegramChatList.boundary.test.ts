import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramChatList mention boundary', () => {
  it('renders mention badges from projected chat metadata instead of inline data fetching', () => {
    const source = readFileSync(new URL('./TelegramChatList.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramChatMentionCountValue')
    expect(source).toContain('telegram-chat-mention-badge')
    expect(source).toContain("t('Unread mentions')")
    expect(source).not.toContain('fetch(')
  })
})
