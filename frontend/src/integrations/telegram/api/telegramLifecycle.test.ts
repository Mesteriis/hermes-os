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

  it('fetches projected reply chains by message id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ message_id: 'msg-1', replies: [], reply_to: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramReplyChain('msg-1')

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-messages/msg-1/reply-chain')
  })

  it('fetches projected forward chains by message id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ message_id: 'msg-1', forwards: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramForwardChain('msg-1')

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-messages/msg-1/forward-chain')
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

  it('fetches message versions and tombstones by message id', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'msg-1', versions: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'msg-1', tombstones: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramMessageVersions('msg-1')
    await fetchTelegramMessageTombstones('msg-1')

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-messages/msg-1/versions')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/telegram/provider-messages/msg-1/tombstones')
  })

  it('fetches account command rows and message reactions', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'msg-1', reactions: [], summary: { message_id: 'msg-1', total_reactions: 0, active_reactions: 0, reactions: [] } }), {
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
    await fetchTelegramReactions('msg-1')

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/commands?account_id=acct-1&limit=25')
    expect(fetchMock.mock.calls[0][0]).toContain('provider_chat_id=chat-42')
    expect(fetchMock.mock.calls[0][0]).toContain('provider_message_id=chat-42%3A77')
    expect(fetchMock.mock.calls[0][0]).toContain('command_kinds=mark_read%2Cmark_unread')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/telegram/provider-messages/msg-1/reactions')
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

  it('adds reactions with a generated command id when one is not provided', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ status: 'added' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await addTelegramReaction('msg-react-1', {
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-msg-1',
      reaction_emoji: '👍',
      sender_id: 'owner-1',
    })

    const [, init] = fetchMock.mock.calls[0]
    const body = JSON.parse(String(init?.body))
    expect(body.command_id).toMatch(/^cmd_/)
  })

  it('removes reactions with a generated command id in the query string', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ status: 'removed' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await removeTelegramReaction('msg-react-1', {
      account_id: 'acct-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-msg-1',
      reaction_emoji: '👍',
      sender_id: 'owner-1',
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(String(fetchMock.mock.calls[0][0])).toContain('command_id=cmd_')
  })
})
