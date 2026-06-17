import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramChatList mention boundary', () => {
  it('renders mention and saved-message badges from projected chat metadata instead of inline data fetching', () => {
    const source = readFileSync(new URL('./TelegramChatList.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramChatMentionCountValue')
    expect(source).toContain('telegramChatIsSavedMessages')
    expect(source).toContain('telegramChatTypingLabel')
    expect(source).toContain(':class="{ typing: telegramChatTypingLabel')
    expect(source).toContain('tabler:bookmark')
    expect(source).toContain('telegram-chat-mention-badge')
    expect(source).toContain("t('Unread mentions')")
    expect(source).not.toContain('fetch(')
  })
})
