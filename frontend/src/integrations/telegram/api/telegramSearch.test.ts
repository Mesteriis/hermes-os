import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  searchTelegramProviderMessages,
} from './telegramSearch'

describe('telegram search API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('builds Telegram provider search trigger requests with required account scope', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        account_id: 'telegram-account-1',
        provider_chat_id: 'chat-42',
        query: 'project alpha',
        limit: 25,
        status: 'queued',
        error: null,
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchTelegramProviderMessages({
      q: 'project alpha',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      limit: 25,
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/integrations/telegram/provider-search')
    expect(init.method).toBe('POST')
    const body = JSON.parse(init.body as string)
    expect(body).toEqual({
      q: 'project alpha',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      limit: 25,
    })
  })
})
