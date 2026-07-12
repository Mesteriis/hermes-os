import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { importMailFile } from './mailImportApi'

describe('mail import API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('posts EML bytes to the bounded local import endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      message_id: 'message-1', raw_record_id: 'raw-1', attachment_count: 2
    }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
    vi.stubGlobal('fetch', fetchMock)

    await expect(importMailFile('account-1', 'eml', 'ZW1s')).resolves.toEqual({
      imported_count: 1,
      message_ids: ['message-1'],
      failed_count: 0,
      failures: []
    })

    expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/api/v1/communications/import/eml')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({
      account_id: 'account-1', eml_base64: 'ZW1s'
    })
  })

  it('posts MBOX bytes and preserves the imported message ids', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      imported_count: 2, message_ids: ['message-1', 'message-2'], failed_count: 1,
      failures: [{ message_index: 1, reason: 'invalid_message' }]
    }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
    vi.stubGlobal('fetch', fetchMock)

    await expect(importMailFile('account-1', 'mbox', 'bWJveA==')).resolves.toEqual({
      imported_count: 2,
      message_ids: ['message-1', 'message-2'],
      failed_count: 1,
      failures: [{ message_index: 1, reason: 'invalid_message' }]
    })

    expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/api/v1/communications/import/mbox')
  })
})
