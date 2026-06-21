import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  addTelegramReaction,
  fetchTelegramCommands,
  fetchTelegramForwardChain,
  fetchTelegramMessageTombstones,
  fetchTelegramMessageVersions,
  fetchTelegramReactions,
  forwardTelegramMessage,
  markTelegramMessageRead,
  pinTelegramMessage,
  fetchTelegramReplyChain,
  removeTelegramReaction,
  retryTelegramCommand,
  restoreTelegramMessageVisibility,
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

  it('rejects projected reply chains from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(fetchTelegramReplyChain('msg-1')).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('rejects projected forward chains from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(fetchTelegramForwardChain('msg-1')).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('sends forward commands with a generated command id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ status: 'sent' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await forwardTelegramMessage({
      message_id: 'msg-forward-1',
      account_id: 'acct-1',
      provider_chat_id: 'target-chat',
      from_provider_chat_id: 'source-chat',
      from_provider_message_id: 'source-chat:42',
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-commands/messages/msg-forward-1/forward')
    const [, init] = fetchMock.mock.calls[0]
    const body = JSON.parse(String(init?.body))
    expect(body.command_id).toMatch(/^cmd_/)
    expect(body.provider_chat_id).toBe('target-chat')
    expect(body.from_provider_chat_id).toBe('source-chat')
    expect(body.from_provider_message_id).toBe('source-chat:42')
  })

  it('rejects message versions and tombstones from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(fetchTelegramMessageVersions('msg-1')).rejects.toThrow('moved')
    await expect(fetchTelegramMessageTombstones('msg-1')).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('fetches account command rows and rejects message reactions', async () => {
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
    await expect(fetchTelegramReactions('msg-1')).rejects.toThrow('moved')

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

  it('sends restore visibility with a generated command id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ status: 'visibility_restored' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await restoreTelegramMessageVisibility({
      message_id: 'msg-restore-1',
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-msg-1',
    })

    const [, init] = fetchMock.mock.calls[0]
    const body = JSON.parse(String(init?.body))
    expect(body.command_id).toMatch(/^cmd_/)
    expect(body.reason).toBe('manual_restore')
  })

  it('sends message pin updates with a generated command id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ status: 'pinned' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await pinTelegramMessage({
      message_id: 'msg-pin-1',
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-msg-1',
      is_pinned: true,
    })

    const [, init] = fetchMock.mock.calls[0]
    const body = JSON.parse(String(init?.body))
    expect(body.command_id).toMatch(/^cmd_/)
    expect(body.is_pinned).toBe(true)
  })

  it('posts message-level mark-read against the dedicated Telegram route', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'mark_read', status: 'read', metadata: {} }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await markTelegramMessageRead({
      message_id: 'msg-read-1',
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-commands/messages/msg-read-1/mark-read')
    const [, init] = fetchMock.mock.calls[0]
    expect(init?.method).toBe('POST')
    expect(JSON.parse(String(init?.body))).toEqual({
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
    })
  })

  it('rejects reaction add commands from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(addTelegramReaction('msg-react-1', {
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-msg-1',
      reaction_emoji: '👍',
      sender_id: 'owner-1',
    })).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('rejects reaction remove commands from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(removeTelegramReaction('msg-react-1', {
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-msg-1',
      reaction_emoji: '👍',
      sender_id: 'owner-1',
    })).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })
})
