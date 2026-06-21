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

  it('delegates projected raw evidence to the shared Communication API wrapper', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          raw_record: {
            raw_record_id: 'raw-1',
            provider_kind: 'telegram',
            provider_account_id: 'acct-1',
            provider_message_id: 'provider-msg-1',
            source_uri: null,
            occurred_at: '2026-01-01T00:00:00Z',
            ingested_at: '2026-01-01T00:00:00Z',
            payload: {},
            headers: {},
            provenance: {},
          },
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchTelegramRawMessageEvidence('msg/raw 1')

    expect(response.raw_record.raw_record_id).toBe('raw-1')
    expect(fetchMock).toHaveBeenCalledOnce()
  })
})
