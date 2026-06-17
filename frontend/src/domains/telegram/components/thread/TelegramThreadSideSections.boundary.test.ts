import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramThreadSideSections media boundary', () => {
  it('renders a dedicated voice tab surface with inline playback and query-backed file download fallback', () => {
    const source = readFileSync(new URL('./TelegramThreadSideSections.vue', import.meta.url), 'utf8')

    expect(source).toContain("activeThreadTab === 'voice'")
    expect(source).toContain('voiceHints: TelegramAttachmentHint[]')
    expect(source).toContain('mergeTelegramAttachmentHints(props.mediaGalleryItems, props.fileHints)')
    expect(source).toContain('telegramMediaAlbumGroupsForMessages(props.chronologicalMessages)')
    expect(source).toContain('telegram-media-album-card')
    expect(source).toContain('telegramTopicStateLabel')
    expect(source).toContain('telegramTopicProviderLabel')
    expect(source).toContain('TelegramAttachmentSearchPanel')
    expect(source).toContain('accountId: string | null | undefined')
    expect(source).toContain('<TelegramAttachmentSearchPanel :accountId="accountId" />')
    expect(source).toContain("t('Media album')")
    expect(source).toContain('attachmentIcon(attachment)')
    expect(source).toContain("attachment.kind === 'sticker'")
    expect(source).toContain("attachment.kind === 'animation' || attachment.kind === 'video_note'")
    expect(source).toContain('telegram-voice-card__player')
    expect(source).toContain("emit('downloadMedia', attachment, messageForAttachment(attachment) ?? undefined)")
    expect(source).toContain("t('Voice playback is available after local download.')")
    expect(source).toContain("emit('downloadMedia', voice, messageForAttachment(voice) ?? undefined)")
    expect(source).toContain("@downloadMedia=\"(attachment) => emit('downloadMedia', attachment, messageForAttachment(attachment) ?? undefined)\"")
    expect(source).not.toContain('fetch(')
  })
})
