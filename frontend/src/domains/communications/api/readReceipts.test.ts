import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { recordReadReceipt } from './readReceipts'

describe('communication read receipt API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('posts read receipts through the protected communications API', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          receipt_id: 'mail_read_receipt:1',
          outbox_id: 'outbox-1',
          receipt_kind: 'read'
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await recordReadReceipt({
      account_id: 'account-1',
      provider_message_id: 'provider-message-1',
      recipient: 'reader@example.com',
      read_at: '2026-06-15T10:00:00Z',
      source_kind: 'mdn'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/read-receipts')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({
      account_id: 'account-1',
      provider_message_id: 'provider-message-1',
      recipient: 'reader@example.com',
      read_at: '2026-06-15T10:00:00Z',
      source_kind: 'mdn'
    })
  })
})
