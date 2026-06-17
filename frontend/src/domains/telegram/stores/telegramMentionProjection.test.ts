import { describe, expect, it } from 'vitest'
import {
  telegramMessageMentionLabel,
  telegramMessageMentionProjection,
} from './telegramMentionProjection'
import type { TelegramMessage } from '../types/telegram'

function message(metadata: Record<string, unknown>): TelegramMessage {
  return {
    message_id: 'msg-1',
    raw_record_id: 'raw-1',
    account_id: 'acct-1',
    provider_message_id: 'provider-msg-1',
    provider_chat_id: 'chat-1',
    chat_title: 'Chat',
    sender: 'sender-1',
    sender_display_name: 'Sender',
    text: 'Hello @alice',
    occurred_at: '2026-06-17T10:00:00Z',
    projected_at: '2026-06-17T10:00:00Z',
    channel_kind: 'telegram_user',
    delivery_state: 'received',
    metadata,
  }
}

describe('telegram mention projection', () => {
  it('uses projected mention handles when available', () => {
    const projection = telegramMessageMentionProjection(
      message({
        mention_count: 2,
        mentions: ['@alice', '@bob'],
        mentions_detected_by: 'text_regex',
      })
    )

    expect(projection).toEqual({
      count: 2,
      mentions: ['@alice', '@bob'],
      detected_by: 'text_regex',
    })
    expect(telegramMessageMentionLabel(message({ mentions: ['@alice', '@bob'] }))).toBe(
      '@alice, @bob'
    )
  })

  it('falls back to mention count when TDLib entities do not include handles', () => {
    expect(
      telegramMessageMentionLabel(
        message({
          mention_count: 3,
          mentions: [],
          mentions_detected_by: 'tdlib_entities',
        })
      )
    ).toBe('3 mentions')
  })

  it('caps rendered mention handles and reports overflow', () => {
    expect(
      telegramMessageMentionLabel(
        message({
          mentions: ['@a', '@b', '@c', '@d'],
        })
      )
    ).toBe('@a, @b, @c +1')
  })
})
