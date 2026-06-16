import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramMessageList pin boundary', () => {
  it('exposes a capability-gated local pin toggle for messages', () => {
    const source = readFileSync(new URL('./TelegramMessageList.vue', import.meta.url), 'utf8')

    expect(source).toContain("togglePinMessage: [message: TelegramMessage]")
    expect(source).toContain("capability('messages.pin')")
    expect(source).toContain("emit('togglePinMessage', message)")
    expect(source).toContain("isMessagePinned(message) ? 'tabler:pinned-off' : 'tabler:pinned'")
  })
})
