import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import { searchTelegramBusinessTopics } from './telegramBusinessApi'

describe('telegram business API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('searches projected topics through Communications routes', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchTelegramBusinessTopics('chat-42 ', '  architecture docs ', 25)

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/topics/search?')
    expect(url).toContain('q=architecture+docs')
    expect(url).toContain('telegram_chat_id=chat-42')
    expect(url).toContain('limit=25')
  })
})
