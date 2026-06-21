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

  it('delegates projected topic search to the shared Communication API wrapper', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramTopicSearch('chat-42 ', '  architecture docs ', 25)

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    const requestUrl = new URL(String(url))
    expect(requestUrl.searchParams.get('q')).toBe('architecture docs')
    expect(requestUrl.searchParams.get('telegram_chat_id')).toBe('chat-42')
    expect(requestUrl.searchParams.get('limit')).toBe('25')
    expect(requestUrl.pathname).not.toContain('/integrations/telegram')
  })
})
