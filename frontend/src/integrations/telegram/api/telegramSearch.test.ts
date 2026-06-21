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

  it('rejects projected dialog search from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(searchTelegramChats({
      q: 'alpha',
      account_id: 'telegram-account-1',
      limit: 15,
    })).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('rejects projected message search from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(searchTelegramMessages({
      q: 'project alpha',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      limit: 25,
    })).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
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
    expect(url).toContain('/api/v1/integrations/telegram/provider-search/provider')
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
    expect(url).toContain('/api/v1/integrations/telegram/provider-search/media?')
    expect(url).toContain('q=invoice')
    expect(url).toContain('account_id=telegram-account-1')
    expect(url).toContain('provider_chat_id=chat-42')
    expect(url).toContain('kind=photo')
    expect(url).toContain('limit=80')
    expect(init.method).toBe('GET')
    expect(response.source).toBe('provider_refresh')
    expect(response.provider_search_attempted).toBe(true)
  })

  it('rejects projected pinned messages from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(fetchTelegramPinnedMessages({
      telegram_chat_id: 'tgchat-1',
      limit: 40,
    })).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })
})
