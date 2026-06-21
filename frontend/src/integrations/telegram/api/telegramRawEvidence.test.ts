import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import { fetchTelegramRawMessageEvidence } from './telegramRawEvidence'

describe('telegram raw evidence API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('fetches sanitized raw source evidence by message id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ raw_record: { raw_record_id: 'raw-1' } }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramRawMessageEvidence('msg/raw 1')

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/integrations/telegram/provider-messages/msg%2Fraw%201/raw-evidence')
    expect(init.method).toBe('GET')
  })
})
