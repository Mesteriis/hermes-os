import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramMediaViewer boundary', () => {
  it('maps projected Telegram sticker, animation and video note attachments to preview categories', () => {
    const source = readFileSync(new URL('./TelegramMediaViewer.vue', import.meta.url), 'utf8')

    expect(source).toContain('useTelegramAttachmentPreviewQuery')
    expect(source).toContain("attachmentPreview?.preview_kind === 'text'")
    expect(source).toContain("attachmentPreview?.preview_kind === 'image'")
    expect(source).toContain("kind === 'photo' || kind === 'sticker'")
    expect(source).toContain("kind === 'video' || kind === 'animation' || kind === 'video_note'")
    expect(source).toContain('telegramAttachmentReadiness')
    expect(source).toContain('downloadMedia: [attachment: TelegramAttachmentHint]')
    expect(source).toContain("attachmentReadiness?.can_request_download")
    expect(source).toContain("@click=\"emit('downloadMedia', attachment)\"")
    expect(source).not.toContain('fetch(')
  })
})
