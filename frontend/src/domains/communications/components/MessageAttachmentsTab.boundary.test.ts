import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageAttachmentsTab boundary', () => {
  it('uses TanStack Query archive inspection wiring without direct component fetch', () => {
    const source = readFileSync(
      new URL('./MessageAttachmentsTab.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('useAttachmentArchiveInspectionQuery')
    expect(source).toContain('useAttachmentPreviewQuery')
    expect(source).toContain('useTranslateAttachmentMutation')
    expect(source).toContain('attachmentTranslationTarget')
    expect(source).toContain('attachmentTranslationResult')
    expect(source).toContain('attachmentTranslationError')
    expect(source).toContain('translateSelectedAttachment')
    expect(source).toContain('source_text: preview.text')
    expect(source).toContain('attachment-translation-panel')
    expect(source).toContain('Attachment translation')
    expect(source).toContain('isInspectableArchiveAttachment')
    expect(source).toContain('isPreviewableImageAttachment')
    expect(source).toContain('isPreviewableTextAttachment')
    expect(source).toContain('Inspect archive')
    expect(source).toContain('Attachment preview')
    expect(source).toContain('attachment-preview-image')
    expect(source).toContain('attachmentPreview.data_url')
    expect(source).not.toContain('../api/communications')
    expect(source).not.toContain('fetch(')
  })
})
