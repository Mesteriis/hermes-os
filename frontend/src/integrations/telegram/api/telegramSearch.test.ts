import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchTelegramPinnedMessages,
  searchTelegramChats,
  searchTelegramProviderMessages,
  searchTelegramMedia,
  searchTelegramMessages,
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

  it('builds Telegram dialog search requests with scoped query params', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ query: 'alpha', items: [], total: 0 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchTelegramChats({
      q: 'alpha',
      account_id: 'telegram-account-1',
      limit: 15,
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/telegram/chats/search?')
    expect(url).toContain('q=alpha')
    expect(url).toContain('account_id=telegram-account-1')
    expect(url).toContain('limit=15')
    expect(init.method).toBe('GET')
  })

  it('builds Telegram message search requests with scoped query params', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ query: 'project alpha', items: [], total: 0 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchTelegramMessages({
      q: 'project alpha',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      limit: 25,
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/telegram/search/messages?')
    expect(url).toContain('q=project+alpha')
    expect(url).toContain('account_id=telegram-account-1')
    expect(url).toContain('provider_chat_id=chat-42')
    expect(url).toContain('limit=25')
    expect(init.method).toBe('GET')
  })

  it('builds Telegram provider message search requests with required account scope', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ query: 'project alpha', items: [], total: 0 }), {
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
    expect(url).toContain('/api/v1/communications/telegram/search/provider')
    expect(init.method).toBe('POST')
    const body = JSON.parse(init.body as string)
    expect(body).toEqual({
      q: 'project alpha',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      limit: 25,
    })
  })

  it('builds Telegram media gallery search requests with text and kind filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        query: 'invoice',
        source: 'provider_refresh',
        provider_search_attempted: true,
        provider_search_error: null,
        items: []
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await searchTelegramMedia({
      q: 'invoice',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      kind: 'photo',
      limit: 80,
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/telegram/search/media?')
    expect(url).toContain('q=invoice')
    expect(url).toContain('account_id=telegram-account-1')
    expect(url).toContain('provider_chat_id=chat-42')
    expect(url).toContain('kind=photo')
    expect(url).toContain('limit=80')
    expect(init.method).toBe('GET')
    expect(response.source).toBe('provider_refresh')
    expect(response.provider_search_attempted).toBe(true)
  })

  it('builds Telegram pinned-message requests for a projected chat', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramPinnedMessages({
      telegram_chat_id: 'tgchat-1',
      limit: 40,
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/telegram/chats/tgchat-1/pinned-messages?')
    expect(url).toContain('limit=40')
    expect(init.method).toBe('GET')
  })
})
