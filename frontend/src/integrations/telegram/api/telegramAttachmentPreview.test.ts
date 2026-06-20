import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { previewTelegramAttachment } from './telegramAttachmentPreview'

describe('telegram attachment preview API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('uses the shared Communication attachment preview endpoint for projected Telegram attachments', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachment_id: 'telegram-attachment:1',
          message_id: 'telegram-message:1',
          filename: 'note.txt',
          content_type: 'text/plain',
          scan_status: 'not_scanned',
          preview_kind: 'text',
          text: 'hello',
          data_url: null,
          truncated: false,
          byte_count: 5,
          max_preview_bytes: 65536,
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const preview = await previewTelegramAttachment('telegram-attachment:1')

    expect(preview.preview_kind).toBe('text')
    expect(preview.text).toBe('hello')
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/attachments/telegram-attachment%3A1/preview')
    expect(init.method).toBe('GET')
  })
})
