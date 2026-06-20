import { describe, expect, it } from 'vitest'
import type { TelegramAttachmentHint } from '../types/telegram'
import { telegramDownloadQueueItems, telegramDownloadQueueTitle } from './telegramDownloadQueue'

function attachment(overrides: Partial<TelegramAttachmentHint> = {}): TelegramAttachmentHint {
  return {
    id: 'att-1',
    kind: 'document',
    fileName: 'invoice.pdf',
    mimeType: 'application/pdf',
    sizeBytes: 2048,
    tdlibFileId: 42,
    providerAttachmentId: 'attachment-42',
    downloadState: 'remote',
    localPath: null,
    expectedSizeBytes: null,
    downloadedSizeBytes: null,
    isDownloadingActive: false,
    isDownloadingCompleted: false,
    lastError: null,
    messageId: 'msg-1',
    providerMessageId: 'provider-msg-1',
    ...overrides,
  }
}

describe('telegram download queue', () => {
  it('keeps only in-progress and failed current-chat downloads in stable order', () => {
    const items = telegramDownloadQueueItems(
      [
        attachment({ id: 'att-progress', downloadState: 'downloading', isDownloadingActive: true }),
        attachment({ id: 'att-ready', downloadState: 'downloaded', localPath: '/tmp/invoice.pdf' }),
      ],
      [
        attachment({ id: 'att-failed', downloadState: 'failed', lastError: 'tdlib timeout' }),
        attachment({ id: 'att-progress', downloadState: 'downloading', isDownloadingActive: true }),
      ],
      5
    )

    expect(items.map((item) => item.id)).toEqual(['att-progress', 'att-failed'])
  })

  it('derives a visible title from filename, provider attachment id or tdlib file id', () => {
    expect(telegramDownloadQueueTitle(attachment())).toBe('invoice.pdf')
    expect(
      telegramDownloadQueueTitle(
        attachment({
          fileName: '   ',
          providerAttachmentId: 'attachment-fallback',
        })
      )
    ).toBe('attachment-fallback')
    expect(
      telegramDownloadQueueTitle(
        attachment({
          fileName: '   ',
          providerAttachmentId: '   ',
          tdlibFileId: 51,
        })
      )
    ).toBe('TDLib file 51')
  })
})
