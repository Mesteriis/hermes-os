import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import { fetchTelegramTopicSearch } from './telegramTopics'

describe('telegram topics API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('builds Telegram topic search requests with trimmed parameters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ telegram_chat_id: 'chat-42', items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramTopicSearch('chat-42 ', '  architecture docs ', 25)

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/integrations/telegram/provider-topics/search?')
    expect(url).toContain('q=architecture+docs')
    expect(url).toContain('telegram_chat_id=chat-42')
    expect(url).toContain('limit=25')
    expect(init.method).toBe('GET')
  })
})
