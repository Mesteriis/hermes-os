import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ThreadAttachmentInsightPanel boundaries', () => {
  it('uses attachment preview and archive inspection query hooks without direct API calls', () => {
    const source = readFileSync(new URL('./ThreadAttachmentInsightPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useAttachmentArchiveInspectionQuery')
    expect(source).toContain('useAttachmentPreviewQuery')
    expect(source).toContain('useTranslateAttachmentMutation')
    expect(source).toContain('isPreviewableAttachment')
    expect(source).toContain('isInspectableArchiveAttachment')
    expect(source).toContain('isPreviewableImageAttachment')
    expect(source).toContain('Translate preview')
    expect(source).toContain('Inspect archive')
    expect(source).toContain('Attachment preview')
    expect(source).toContain('Archive inspection')
    expect(source).toContain('Thread attachment translation')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('ApiClient')
  })
})
