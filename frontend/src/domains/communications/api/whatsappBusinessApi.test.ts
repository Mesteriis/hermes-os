import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import { fetchWhatsappWebBusinessMessages } from './whatsappBusinessApi'

describe('WhatsApp business API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
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
})
