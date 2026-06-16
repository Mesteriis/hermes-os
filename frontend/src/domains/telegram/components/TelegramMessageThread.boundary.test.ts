import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramMessageThread thread surfaces boundary', () => {
  it('uses query-backed pinned messages, exposes a dedicated voice tab, and forwards media search openings', () => {
    const source = readFileSync(new URL('./TelegramMessageThread.vue', import.meta.url), 'utf8')

    expect(source).toContain('useTelegramPinnedMessagesQuery')
    expect(source).toContain('telegramVoiceAttachmentHintsForMessages')
    expect(source).toContain("{ id: 'voice', label: t('Voice'), count: voiceHints.value.length }")
    expect(source).toContain(':voiceHints="voiceHints"')
    expect(source).toContain('focusedTelegramMessage')
    expect(source).toContain('pinnedMessagesQuery.data.value?.items')
    expect(source).toContain('threadSearchQuery.value = props.focusedTelegramMessage.provider_message_id')
    expect(source).toContain('openSearchMedia: [item: TelegramMediaItem]')
    expect(source).toContain("@openMedia=\"(item) => emit('openSearchMedia', item)\"")
    expect(source).toContain("@togglePinMessage=\"(message) => emit('togglePinMessage', message)\"")
    expect(source).toContain("@openMessage=\"(message) => emit('openSearchMessage', message)\"")
  })
})
