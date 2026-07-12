import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import {
  addTelegramBusinessReaction,
  fetchTelegramBusinessChatFolders,
  fetchTelegramBusinessMessages,
  forwardTelegramBusinessMessage,
  pinTelegramBusinessMessage,
  replyToTelegramBusinessMessage,
  removeTelegramBusinessReaction,
  searchTelegramBusinessTopics,
  sendTelegramBusinessMessage,
} from './telegramBusinessApi'

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

  it('loads Telegram conversation folders through the canonical Communications route', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramBusinessChatFolders(' telegram-account-1 ')

    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/conversation-folders?account_id=telegram-account-1')
    expect(init.method).toBe('GET')
  })

  it('adapts canonical Communication messages to Telegram message DTOs', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          items: [
            {
              message_id: 'msg-1',
              raw_record_id: 'raw-1',
              account_id: 'telegram-account-1',
              provider_record_id: 'provider-message-1',
              subject: 'General',
              sender: 'telegram:user:42',
              recipients: [],
              body_text_preview: 'hello from projection',
              occurred_at: '2026-06-20T10:00:00Z',
              projected_at: '2026-06-20T10:00:01Z',
              channel_kind: 'telegram_user',
              conversation_id: 'chat-1',
              sender_display_name: 'Ada',
              delivery_state: 'received',
              workflow_state: 'new',
              importance_score: null,
              ai_category: null,
              ai_summary: null,
              ai_summary_generated_at: null,
              message_metadata: { is_pinned: true },
              attachment_count: 0,
              local_state: 'active',
              local_state_changed_at: null,
            },
          ],
          next_cursor: null,
          has_more: false,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchTelegramBusinessMessages('telegram-account-1', 'chat-1', 25)

    expect(response.items).toEqual([
      {
        message_id: 'msg-1',
        raw_record_id: 'raw-1',
        account_id: 'telegram-account-1',
        provider_message_id: 'provider-message-1',
        provider_chat_id: 'chat-1',
        chat_title: 'General',
        sender: 'telegram:user:42',
        sender_display_name: 'Ada',
        text: 'hello from projection',
        occurred_at: '2026-06-20T10:00:00Z',
        projected_at: '2026-06-20T10:00:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: { is_pinned: true },
      },
    ])
    expect(response).toMatchObject({ next_cursor: null, has_more: false })
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages?')
    expect(url).toContain('channel_kind=telegram')
    expect(url).toContain('conversation_id=chat-1')
  })

  it('uses the provider-neutral pin response shape', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ message_id: 'msg-1', pinned: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await pinTelegramBusinessMessage({ message_id: 'msg-1' })

    expect(response).toEqual({ message_id: 'msg-1', pinned: true })
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages/msg-1/pin')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({})
  })

  it('uses the provider-neutral command response shape for message writes', async () => {
    const commandResponse = {
      message_id: 'msg-1',
      raw_record_id: 'raw-1',
      conversation_id: 'chat-1',
      provider_chat_id: 'chat-1',
      provider_message_id: null,
      channel_kind: 'telegram_user',
      status: 'queued',
      command_id: 'command-1',
      provider: 'telegram',
    }
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify(commandResponse), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ ...commandResponse, provider_message_id: 'provider-reply-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ ...commandResponse, command_id: 'command-forward-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await expect(sendTelegramBusinessMessage({
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      text: 'hello',
    })).resolves.toEqual(commandResponse)
    await expect(replyToTelegramBusinessMessage({
      message_id: 'msg-1',
      text: 'reply',
    })).resolves.toMatchObject({ provider_message_id: 'provider-reply-1' })
    await expect(forwardTelegramBusinessMessage({
      message_id: 'msg-1',
      provider_chat_id: 'chat-2',
    })).resolves.toMatchObject({ command_id: 'command-forward-1' })

    const [sendUrl, sendInit] = fetchMock.mock.calls[0]
    expect(sendUrl).toContain('/api/v1/communications/conversations/chat-1/messages')
    expect(JSON.parse(sendInit.body as string)).toEqual({ account_id: 'account-1', text: 'hello' })

    const [replyUrl, replyInit] = fetchMock.mock.calls[1]
    expect(replyUrl).toContain('/api/v1/communications/messages/msg-1/reply')
    expect(JSON.parse(replyInit.body as string)).toEqual({ text: 'reply' })

    const [forwardUrl, forwardInit] = fetchMock.mock.calls[2]
    expect(forwardUrl).toContain('/api/v1/communications/messages/msg-1/forward')
    expect(JSON.parse(forwardInit.body as string)).toEqual({ conversation_id: 'chat-2' })
  })

  it('lets the Telegram server derive the authenticated reaction sender', async () => {
    const reactionResponse = {
      reaction_id: 'reaction-1',
      message_id: 'msg-1',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-message-1',
      reaction_emoji: '👍',
      is_active: true,
      status: 'queued',
      timestamp: '2026-07-12T10:00:00Z',
    }
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(JSON.stringify(reactionResponse), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ ...reactionResponse, is_active: false }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
    vi.stubGlobal('fetch', fetchMock)

    const request = {
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-1',
      provider_message_id: 'provider-message-1',
      reaction_emoji: '👍',
    }
    await addTelegramBusinessReaction('msg-1', request)
    await removeTelegramBusinessReaction('msg-1', request)

    const [, addInit] = fetchMock.mock.calls[0]
    expect(JSON.parse(addInit.body as string)).toEqual(request)
    const [removeUrl] = fetchMock.mock.calls[1]
    expect(removeUrl).not.toContain('sender_id=')
  })
})
