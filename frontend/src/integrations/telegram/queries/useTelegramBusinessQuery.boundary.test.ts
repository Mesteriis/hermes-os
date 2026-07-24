import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Telegram business-query boundary', () => {
  it('keeps provider-specific Telegram orchestration in the integration unit', () => {
    const querySource = readFileSync(
      new URL('./useTelegramBusinessQuery.ts', import.meta.url),
      'utf8'
    )
    const mutationSource = readFileSync(
      new URL('./useTelegramBusinessMutations.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('../../../../domains/communications/queries/telegramBusinessQueries.ts', import.meta.url))).toBe(false)

    expect(querySource).toContain('useTelegramChatsQuery')
    expect(querySource).toContain('useTelegramMessagesQuery')
    expect(querySource).toContain('useTelegramMessageSearchQuery')
    expect(mutationSource).toContain('useSendTelegramMessageMutation')
    expect(mutationSource).toContain('useReplyTelegramMessageMutation')
    expect(mutationSource).toContain('useEditTelegramMessageMutation')
    expect(mutationSource).toContain('useDeleteTelegramMessageMutation')
    expect(mutationSource).toContain('usePinTelegramMessageMutation')
    expect(querySource).toContain('searchTelegramBusinessMessages')
    expect(querySource).toContain('fetchTelegramBusinessMessages')
    expect(querySource).toContain('telegramBusinessQueryKeys')
    expect(querySource).not.toContain("domains/communications")
    expect(mutationSource).not.toContain("domains/communications")
    expect(querySource).not.toContain('.vue')
  })
})
