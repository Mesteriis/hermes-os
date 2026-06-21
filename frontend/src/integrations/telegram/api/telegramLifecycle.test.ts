import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchTelegramCommands,
  retryTelegramCommand,
} from './telegramLifecycle'

describe('telegram lifecycle reference API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('fetches account command rows', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramCommands('acct-1', 25, {
      providerChatId: 'chat-42',
      providerMessageId: 'chat-42:77',
      commandKinds: ['mark_read', 'mark_unread'],
    })

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/commands?account_id=acct-1&limit=25')
    expect(fetchMock.mock.calls[0][0]).toContain('provider_chat_id=chat-42')
    expect(fetchMock.mock.calls[0][0]).toContain('provider_message_id=chat-42%3A77')
    expect(fetchMock.mock.calls[0][0]).toContain('command_kinds=mark_read%2Cmark_unread')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('sends manual retry through the provider command outbox endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ command_id: 'cmd-retry-1', status: 'retrying' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await retryTelegramCommand('cmd-retry-1')

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/commands/cmd-retry-1/retry')
    const [, init] = fetchMock.mock.calls[0]
    expect(init?.method).toBe('POST')
  })

})
