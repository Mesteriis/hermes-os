import { describe, expect, it } from 'vitest'
import { telegramAttachmentDownloadExtras } from './telegramAttachmentDownload'

describe('telegramAttachmentDownloadExtras', () => {
  it('only exposes a provider download command for attachments with TDLib identity', () => {
    expect(telegramAttachmentDownloadExtras({
      id: 'attachment-1',
      name: 'brief.pdf',
      meta: 'document',
      icon: 'tabler:file-description',
      downloadable: true,
      providerAttachmentId: 'tdlib:document:42',
      providerMessageId: '-100:42',
      tdlibFileId: 42,
      contentType: 'application/pdf',
    })).toEqual({
      providerAttachmentId: 'tdlib:document:42',
      providerMessageId: '-100:42',
      tdlibFileId: 42,
      filename: 'brief.pdf',
      contentType: 'application/pdf',
    })

    expect(telegramAttachmentDownloadExtras({
      id: 'attachment-2',
      name: 'unavailable.pdf',
      meta: 'document',
      icon: 'tabler:file-description',
      downloadable: false,
    })).toBeNull()
  })
})
