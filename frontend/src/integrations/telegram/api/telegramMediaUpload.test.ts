import { afterEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  importCommunicationAttachment,
  uploadTelegramMedia
} from './telegramMediaUpload'

describe('telegramMediaUpload api', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('imports local attachments through the Communication import endpoint', async () => {
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachment_id: 'att-import:1',
          blob_id: 'blob:1',
          content_type: 'text/plain',
          size_bytes: 5,
          sha256: 'sha256:abc',
          scan_status: 'not_scanned',
          storage_kind: 'local_fs',
          storage_path: 'sha256/abc'
        }),
        { status: 200 }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await importCommunicationAttachment({
      account_id: 'telegram-1',
      channel_kind: 'telegram',
      filename: 'note.txt',
      content_type: 'text/plain',
      content_base64: 'aGVsbG8='
    })

    expect(response.attachment_id).toBe('att-import:1')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/attachments/import')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toMatchObject({
      account_id: 'telegram-1',
      content_base64: 'aGVsbG8='
    })
  })

  it('queues Telegram media upload through the provider command endpoint', async () => {
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          command_id: 'tcmd-media-1',
          account_id: 'telegram-1',
          provider_chat_id: '123',
          attachment_id: 'att-import:1',
          blob_id: 'blob:1',
          media_type: 'document',
          status: 'queued',
          reconciliation_status: 'not_observed'
        }),
        { status: 200 }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await uploadTelegramMedia({
      account_id: 'telegram-1',
      provider_chat_id: '123',
      attachment_id: 'att-import:1',
      media_type: 'document'
    })

    expect(response.status).toBe('queued')
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/telegram/media/upload')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toMatchObject({
      attachment_id: 'att-import:1',
      media_type: 'document'
    })
  })
})
