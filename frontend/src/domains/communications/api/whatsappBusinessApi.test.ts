import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import {
  addWhatsappBusinessReaction,
  archiveWhatsappBusinessConversation,
  deleteWhatsappBusinessMessage,
  editWhatsappBusinessMessage,
  fetchWhatsappBusinessReactions,
  fetchWhatsappWebBusinessConversationDetail,
  fetchWhatsappWebBusinessConversationMembers,
  fetchWhatsappWebBusinessConversations,
  fetchWhatsappWebBusinessMessages,
  fetchWhatsappWebBusinessPinnedMessages,
  forwardWhatsappBusinessMessage,
  markWhatsappBusinessConversationRead,
  markWhatsappBusinessConversationUnread,
  muteWhatsappBusinessConversation,
  pinWhatsappBusinessConversation,
  pinWhatsappBusinessMessage,
  replyToWhatsappBusinessMessage,
  removeWhatsappBusinessReaction,
  sendWhatsappBusinessMessage,
  searchWhatsappWebBusinessMedia,
  searchWhatsappWebBusinessMessages,
  unarchiveWhatsappBusinessConversation,
  unmuteWhatsappBusinessConversation,
  unpinWhatsappBusinessConversation,
} from './whatsappBusinessApi'

describe('WhatsApp business API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('reads projected whatsapp conversations from the provider-neutral route', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          items: [
            {
              conversation_id: 'wa-chat-1',
              account_id: 'whatsapp-account-1',
              provider_chat_id: 'wa-chat-1',
              title: 'Family',
              last_message_at: '2026-06-20T11:00:00Z',
              metadata: { channel_kind: 'whatsapp_web' },
              created_at: '2026-06-20T11:00:00Z',
              updated_at: '2026-06-20T11:00:01Z',
            },
            {
              conversation_id: 'tg-chat-1',
              account_id: 'telegram-account-1',
              provider_chat_id: 'tg-chat-1',
              title: 'Telegram',
              last_message_at: '2026-06-20T11:00:00Z',
              metadata: { channel_kind: 'telegram_user' },
              created_at: '2026-06-20T11:00:00Z',
              updated_at: '2026-06-20T11:00:01Z',
            },
          ],
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchWhatsappWebBusinessConversations('whatsapp-account-1', 10)

    expect(response.items).toEqual([
      expect.objectContaining({
        conversation_id: 'wa-chat-1',
        provider_chat_id: 'wa-chat-1',
      }),
    ])
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/conversations?')
    expect(url).toContain('account_id=whatsapp-account-1')
    expect(url).toContain('channel_kind=whatsapp_web')
  })

  it('uses provider-neutral whatsapp conversation detail and member routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            item: {
              conversation_id: 'wa-chat-1',
              account_id: 'whatsapp-account-1',
              provider_chat_id: 'wa-chat-1',
              title: 'Family',
              last_message_at: '2026-06-20T11:00:00Z',
              metadata: { channel_kind: 'whatsapp_web' },
              created_at: '2026-06-20T11:00:00Z',
              updated_at: '2026-06-20T11:00:01Z',
            },
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } }
        )
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [],
            next_cursor: null,
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } }
        )
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappWebBusinessConversationDetail('wa-chat-1')
    await fetchWhatsappWebBusinessConversationMembers('wa-chat-1', 25, 'bea', 'admin', '50')

    const [detailUrl] = fetchMock.mock.calls[0]
    expect(detailUrl).toContain('/api/v1/communications/conversations/wa-chat-1')

    const [membersUrl] = fetchMock.mock.calls[1]
    expect(membersUrl).toContain('/api/v1/communications/conversations/wa-chat-1/members?')
    expect(membersUrl).toContain('limit=25')
    expect(membersUrl).toContain('query=bea')
    expect(membersUrl).toContain('role=admin')
    expect(membersUrl).toContain('cursor=50')
  })

  it('adapts canonical Communication messages to WhatsApp message DTOs', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          items: [
            {
              message_id: 'wa-msg-1',
              raw_record_id: 'wa-raw-1',
              account_id: 'whatsapp-account-1',
              provider_record_id: 'provider-wa-1',
              subject: 'Family',
              sender: 'whatsapp:+100000000',
              recipients: [],
              body_text_preview: 'hello from whatsapp',
              occurred_at: '2026-06-20T11:00:00Z',
              projected_at: '2026-06-20T11:00:01Z',
              channel_kind: 'whatsapp_web',
              conversation_id: 'wa-chat-1',
              sender_display_name: 'Bea',
              delivery_state: 'received',
              workflow_state: 'new',
              importance_score: null,
              ai_category: null,
              ai_summary: null,
              ai_summary_generated_at: null,
              message_metadata: { source: 'fixture' },
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

    const response = await fetchWhatsappWebBusinessMessages('whatsapp-account-1', 'wa-chat-1', 10)

    expect(response.items).toEqual([
      {
        message_id: 'wa-msg-1',
        raw_record_id: 'wa-raw-1',
        account_id: 'whatsapp-account-1',
        provider_message_id: 'provider-wa-1',
        provider_chat_id: 'wa-chat-1',
        chat_title: 'Family',
        sender: 'whatsapp:+100000000',
        sender_display_name: 'Bea',
        text: 'hello from whatsapp',
        occurred_at: '2026-06-20T11:00:00Z',
        projected_at: '2026-06-20T11:00:01Z',
        channel_kind: 'whatsapp_web',
        delivery_state: 'received',
        metadata: { source: 'fixture' },
      },
    ])
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages?')
    expect(url).toContain('channel_kind=whatsapp_web')
    expect(url).toContain('conversation_id=wa-chat-1')
  })

  it('uses provider-neutral whatsapp message search with channel scoping', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          query: 'hello',
          items: [],
          total: 0,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchWhatsappWebBusinessMessages({
      q: 'hello',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      limit: 20,
    })

    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/search/messages?')
    expect(url).toContain('channel_kind=whatsapp_web')
    expect(url).toContain('account_id=whatsapp-account-1')
    expect(url).toContain('provider_chat_id=wa-chat-1')
  })

  it('uses provider-neutral whatsapp media search and pinned message routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await searchWhatsappWebBusinessMedia({
      q: 'invoice',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      kind: 'document',
      limit: 10,
    })
    await fetchWhatsappWebBusinessPinnedMessages({
      conversation_id: 'wa-chat-1',
      limit: 10,
    })

    const [mediaUrl] = fetchMock.mock.calls[0]
    expect(mediaUrl).toContain('/api/v1/communications/search/media?')
    expect(mediaUrl).toContain('channel_kind=whatsapp_web')
    expect(mediaUrl).toContain('kind=document')

    const [pinnedUrl] = fetchMock.mock.calls[1]
    expect(pinnedUrl).toContain('/api/v1/communications/conversations/wa-chat-1/pinned-messages?')
    expect(pinnedUrl).toContain('limit=10')
  })

  it('uses provider-neutral whatsapp message command routes', async () => {
    const fetchMock = vi.fn()
    for (let index = 0; index < 7; index += 1) {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ status: 'queued', pinned: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    }
    vi.stubGlobal('fetch', fetchMock)

    await sendWhatsappBusinessMessage({
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      text: 'hello from panel',
    })
    await replyToWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      text: 'reply text',
    })
    await forwardWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      provider_chat_id: 'wa-chat-2',
    })
    await editWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      new_text: 'edited text',
    })
    await deleteWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      reason_class: 'deleted_by_owner',
      actor_class: 'owner',
      is_provider_delete: false,
    })
    await pinWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
    })
    await pinWhatsappBusinessConversation({
      conversation_id: 'wa-chat-1',
    })

    const [sendUrl, sendInit] = fetchMock.mock.calls[0]
    expect(sendUrl).toContain('/api/v1/communications/conversations/wa-chat-1/messages')
    expect(sendInit.method).toBe('POST')
    expect(JSON.parse(sendInit.body as string)).toEqual({
      account_id: 'whatsapp-account-1',
      text: 'hello from panel',
    })

    const [replyUrl, replyInit] = fetchMock.mock.calls[1]
    expect(replyUrl).toContain('/api/v1/communications/messages/wa-msg-1/reply')
    expect(replyInit.method).toBe('POST')
    expect(JSON.parse(replyInit.body as string)).toEqual({
      text: 'reply text',
    })

    const [forwardUrl, forwardInit] = fetchMock.mock.calls[2]
    expect(forwardUrl).toContain('/api/v1/communications/messages/wa-msg-1/forward')
    expect(forwardInit.method).toBe('POST')
    expect(JSON.parse(forwardInit.body as string)).toEqual({
      conversation_id: 'wa-chat-2',
    })

    const [editUrl, editInit] = fetchMock.mock.calls[3]
    expect(editUrl).toContain('/api/v1/communications/messages/wa-msg-1')
    expect(editInit.method).toBe('PATCH')
    expect(JSON.parse(editInit.body as string)).toMatchObject({
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      new_text: 'edited text',
    })

    const [deleteUrl, deleteInit] = fetchMock.mock.calls[4]
    expect(deleteUrl).toContain('/api/v1/communications/messages/wa-msg-1')
    expect(deleteInit.method).toBe('DELETE')
    expect(JSON.parse(deleteInit.body as string)).toMatchObject({
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      reason_class: 'deleted_by_owner',
      actor_class: 'owner',
      is_provider_delete: false,
    })

    const [pinMessageUrl, pinMessageInit] = fetchMock.mock.calls[5]
    expect(pinMessageUrl).toContain('/api/v1/communications/messages/wa-msg-1/pin')
    expect(pinMessageInit.method).toBe('POST')

    const [pinConversationUrl, pinConversationInit] = fetchMock.mock.calls[6]
    expect(pinConversationUrl).toContain('/api/v1/communications/conversations/wa-chat-1/pin')
    expect(pinConversationInit.method).toBe('POST')
  })

  it('uses provider-neutral whatsapp conversation unpin route', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ status: 'queued', active: false }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await unpinWhatsappBusinessConversation({
      conversation_id: 'wa-chat-1',
    })

    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/conversations/wa-chat-1/unpin')
    expect(init.method).toBe('POST')
  })

  it('uses provider-neutral whatsapp conversation lifecycle routes', async () => {
    const fetchMock = vi.fn()
    for (let index = 0; index < 6; index += 1) {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ status: 'queued', active: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    }
    vi.stubGlobal('fetch', fetchMock)

    await archiveWhatsappBusinessConversation({ conversation_id: 'wa-chat-1' })
    await unarchiveWhatsappBusinessConversation({ conversation_id: 'wa-chat-1' })
    await muteWhatsappBusinessConversation({ conversation_id: 'wa-chat-1' })
    await unmuteWhatsappBusinessConversation({ conversation_id: 'wa-chat-1' })
    await markWhatsappBusinessConversationRead({ conversation_id: 'wa-chat-1' })
    await markWhatsappBusinessConversationUnread({ conversation_id: 'wa-chat-1' })

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/communications/conversations/wa-chat-1/archive')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/communications/conversations/wa-chat-1/unarchive')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/communications/conversations/wa-chat-1/mute')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/communications/conversations/wa-chat-1/unmute')
    expect(fetchMock.mock.calls[4][0]).toContain('/api/v1/communications/conversations/wa-chat-1/read')
    expect(fetchMock.mock.calls[5][0]).toContain('/api/v1/communications/conversations/wa-chat-1/unread')
  })

  it('uses provider-neutral whatsapp reaction routes', async () => {
    const fetchMock = vi.fn()
    for (let index = 0; index < 3; index += 1) {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'wa-msg-1', reactions: [], summary: { message_id: 'wa-msg-1', total_reactions: 0, active_reactions: 0, reactions: [] }, status: 'queued' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    }
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappBusinessReactions('wa-msg-1')
    await addWhatsappBusinessReaction('wa-msg-1', {
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      reaction_emoji: '👍',
      sender_id: 'whatsapp:+100000000',
      sender_display_name: 'Bea',
    })
    await removeWhatsappBusinessReaction('wa-msg-1', {
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      reaction_emoji: '👍',
      sender_id: 'whatsapp:+100000000',
      sender_display_name: 'Bea',
    })

    const [listUrl, listInit] = fetchMock.mock.calls[0]
    expect(listUrl).toContain('/api/v1/communications/messages/wa-msg-1/reactions')
    expect(listInit.method).toBe('GET')

    const [addUrl, addInit] = fetchMock.mock.calls[1]
    expect(addUrl).toContain('/api/v1/communications/messages/wa-msg-1/reactions')
    expect(addInit.method).toBe('POST')
    expect(JSON.parse(addInit.body as string)).toEqual({
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      reaction_emoji: '👍',
      sender_id: 'whatsapp:+100000000',
      sender_display_name: 'Bea',
    })

    const [removeUrl, removeInit] = fetchMock.mock.calls[2]
    expect(removeUrl).toContain('/api/v1/communications/messages/wa-msg-1/reactions?')
    expect(removeUrl).toContain('account_id=whatsapp-account-1')
    expect(removeUrl).toContain('provider_chat_id=wa-chat-1')
    expect(removeUrl).toContain('provider_message_id=provider-wa-1')
    expect(removeUrl).toContain('reaction_emoji=%F0%9F%91%8D')
    expect(removeUrl).toContain('sender_id=whatsapp%3A%2B100000000')
    expect(removeInit.method).toBe('DELETE')
  })
})
