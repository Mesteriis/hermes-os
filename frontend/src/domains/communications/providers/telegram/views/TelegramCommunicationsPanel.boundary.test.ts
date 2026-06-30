import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('TelegramCommunicationsPanel boundary', () => {
  it('preserves provider-specific Telegram orchestration after removing the legacy Vue render layer', () => {
    const querySource = readFileSync(
      new URL('../../../queries/telegramBusinessQueries.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./TelegramCommunicationsPanel.vue', import.meta.url))).toBe(false)

    expect(querySource).toContain('useTelegramChatsQuery')
    expect(querySource).toContain('useTelegramMessagesQuery')
    expect(querySource).toContain('useTelegramMessageSearchQuery')
    expect(querySource).toContain('useSendTelegramMessageMutation')
    expect(querySource).toContain('useReplyTelegramMessageMutation')
    expect(querySource).toContain('useEditTelegramMessageMutation')
    expect(querySource).toContain('useDeleteTelegramMessageMutation')
    expect(querySource).toContain('usePinTelegramMessageMutation')
    expect(querySource).toContain('searchTelegramBusinessMessages')
    expect(querySource).toContain('fetchTelegramBusinessMessages')
    expect(querySource).toContain('telegramBusinessQueryKeys')
    expect(querySource).not.toContain("../providers/telegram/views/")
    expect(querySource).not.toContain('.vue')
  })
})
