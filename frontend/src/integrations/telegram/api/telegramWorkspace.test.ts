import { describe, expect, it } from 'vitest'
import { telegramOldestTdlibMessageId } from './telegramWorkspace'
import type { TelegramMessage } from '../types/telegram'

function telegramMessage(providerMessageId: string): TelegramMessage {
  return {
    message_id: `msg-${providerMessageId}`,
    raw_record_id: `raw-${providerMessageId}`,
    account_id: 'account-1',
    provider_message_id: providerMessageId,
    provider_chat_id: 'chat-1',
    chat_title: 'Chat',
    sender: 'sender-1',
    sender_display_name: 'Sender',
    text: 'message',
    occurred_at: '2026-06-16T10:00:00Z',
    projected_at: '2026-06-16T10:00:01Z',
    channel_kind: 'telegram_user',
    delivery_state: 'received',
    metadata: {},
  }
}

describe('telegram workspace API helpers', () => {
  it('finds the oldest TDLib numeric message id from projected provider ids', () => {
    expect(telegramOldestTdlibMessageId([
      telegramMessage('chat-1:42'),
      telegramMessage('chat-1:7'),
      telegramMessage('non-numeric'),
    ])).toBe(7)
  })
})
