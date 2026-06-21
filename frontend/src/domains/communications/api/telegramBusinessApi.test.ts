import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import {
  fetchTelegramBusinessMessages,
  pinTelegramBusinessMessage,
  searchTelegramBusinessTopics,
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
})
