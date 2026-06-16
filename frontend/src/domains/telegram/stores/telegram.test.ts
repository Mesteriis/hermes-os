import { describe, expect, it } from 'vitest'
import { mergeTelegramAttachmentHints } from './telegram'

describe('telegram media attachment helpers', () => {
  it('preserves query-backed TDLib and local-path metadata when merging media search hits with loaded file hints', () => {
    const merged = mergeTelegramAttachmentHints(
      [{
        message_id: 'msg-1',
        provider_message_id: '42',
        provider_chat_id: 'chat-1',
        file_name: 'invoice.pdf',
        kind: 'document',
        mime_type: 'application/pdf',
        size_bytes: 2048,
        occurred_at: '2026-06-16T10:00:00Z',
        download_state: 'downloaded',
        tdlib_file_id: 9001,
        provider_attachment_id: 'attachment-1',
        local_path: '/tmp/hermes/invoice.pdf',
      }],
      [{
        id: 'msg-1:invoice.pdf',
        kind: 'file',
        fileName: 'invoice.pdf',
        mimeType: 'application/pdf',
        sizeBytes: 2048,
        tdlibFileId: null,
        providerAttachmentId: '',
        downloadState: 'unknown',
        localPath: null,
        messageId: 'msg-1',
        providerMessageId: null,
      }]
    )

    expect(merged).toHaveLength(1)
    expect(merged[0]).toMatchObject({
      fileName: 'invoice.pdf',
      kind: 'document',
      tdlibFileId: 9001,
      providerAttachmentId: 'attachment-1',
      localPath: '/tmp/hermes/invoice.pdf',
      downloadState: 'downloaded',
      providerMessageId: '42',
    })
  })
})
