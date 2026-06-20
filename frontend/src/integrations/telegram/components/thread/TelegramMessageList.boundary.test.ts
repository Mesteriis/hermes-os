import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramMessageList pin boundary', () => {
  it('surfaces per-message mention projection without inline fetching', () => {
    const source = readFileSync(new URL('./TelegramMessageList.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramMessageMentionProjection')
    expect(source).toContain('telegramMessageMentionLabel')
    expect(source).toContain('telegram-message-mentions')
    expect(source).not.toContain('fetch(')
  })

  it('exposes a capability-gated local pin toggle for messages', () => {
    const source = readFileSync(new URL('./TelegramMessageList.vue', import.meta.url), 'utf8')

    expect(source).toContain("togglePinMessage: [message: TelegramMessage]")
    expect(source).toContain("capability('messages.pin')")
    expect(source).toContain("emit('togglePinMessage', message)")
    expect(source).toContain("isMessagePinned(message) ? 'tabler:pinned-off' : 'tabler:pinned'")
  })

  it('keeps reaction rendering in a dedicated thread component', () => {
    const listSource = readFileSync(new URL('./TelegramMessageList.vue', import.meta.url), 'utf8')
    const reactionSource = readFileSync(new URL('./TelegramMessageReactions.vue', import.meta.url), 'utf8')

    expect(listSource).toContain("import TelegramMessageReactions from './TelegramMessageReactions.vue'")
    expect(listSource).toContain('<TelegramMessageReactions')
    expect(reactionSource).toContain("capability('reactions.add')")
    expect(reactionSource).toContain("capability('reactions.remove')")
    expect(reactionSource).toContain('message.metadata?.reaction_summary')
    expect(reactionSource).toContain("emit('addReaction', { message: props.message, emoji })")
  })

  it('uses shared Telegram attachment readiness state instead of inline download heuristics', () => {
    const source = readFileSync(new URL('./TelegramMessageList.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramAttachmentReadiness')
    expect(source).toContain('attachmentReadiness(attachment).label')
    expect(source).toContain('attachmentReadiness(attachment).detail')
    expect(source).toContain('attachmentReadiness(attachment).can_request_download')
    expect(source).not.toContain("attachment.tdlibFileId === null ? t('Download requires TDLib file metadata') : t('Download media')")
  })

  it('renders a provider-observed read-progress divider from shared thread state', () => {
    const source = readFileSync(new URL('./TelegramMessageList.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramThreadReadProgress')
    expect(source).toContain('providerReadProgress')
    expect(source).toContain("t('Read through here')")
    expect(source).toContain('telegram-read-progress-divider')
  })

  it('exposes a capability-gated message-level mark-read action without inline fetching', () => {
    const source = readFileSync(new URL('./TelegramMessageList.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramCanMarkMessageRead')
    expect(source).toContain("capability('messages.mark_read')")
    expect(source).toContain("emit('markReadMessage', message)")
    expect(source).not.toContain('fetch(')
  })
})
