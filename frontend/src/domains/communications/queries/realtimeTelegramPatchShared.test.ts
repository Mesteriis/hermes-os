import { describe, expect, it } from 'vitest'
import { telegramChatSnapshot, telegramMessageSnapshot } from './realtimeTelegramPatchShared'

describe('Telegram realtime snapshot parsing', () => {
  it('rejects unknown finite provider kinds', () => {
    expect(telegramChatSnapshot({
      telegram_chat_id: 'chat-1', account_id: 'account-1', provider_chat_id: 'provider-1',
      chat_kind: 'future_kind', title: 'Chat', sync_state: 'synced',
      created_at: '2026-07-21T00:00:00Z', updated_at: '2026-07-21T00:00:00Z'
    })).toBeNull()

    expect(telegramMessageSnapshot({
      message_id: 'message-1', account_id: 'account-1', provider_message_id: 'provider-1',
      chat_title: 'Chat', sender: 'Sender', projected_at: '2026-07-21T00:00:00Z',
      channel_kind: 'future_kind', delivery_state: 'sent'
    })).toBeNull()
  })
})
