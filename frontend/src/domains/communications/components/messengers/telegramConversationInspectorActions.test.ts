import { describe, expect, it } from 'vitest'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import {
  buildTelegramRuntimeActionRequest,
  buildTelegramHistoryPolicyRequest,
  buildTelegramReadReceiptPolicyRequest,
  buildTelegramTopicCloseRequest,
  buildTelegramTopicCreateRequest,
  buildTelegramUnreadCounterPolicyRequest,
  selectedTelegramProviderFolderId,
} from './telegramConversationInspectorActions'

describe('telegram conversation inspector actions', () => {
  it('builds provider-scoped runtime and topic requests', () => {
    const chat = telegramChat()
    expect(buildTelegramRuntimeActionRequest(chat, 'join', { providerFolderId: 7 })).toMatchObject({
      action: 'join', accountId: 'account-1', providerChatId: 'provider-chat-1',
      telegramChatId: 'telegram-chat-1', providerFolderId: 7,
    })
    expect(buildTelegramTopicCreateRequest(chat, 'Updates', 'command-1')).toMatchObject({
      conversationId: 'telegram-chat-1', request: { command_id: 'command-1', title: 'Updates' },
    })
    expect(buildTelegramTopicCloseRequest(chat, 'topic-1', false, 'command-2').request.is_closed).toBe(true)
    expect(buildTelegramHistoryPolicyRequest(chat, true).enabled).toBe(true)
    expect(buildTelegramReadReceiptPolicyRequest(chat, false).enabled).toBe(false)
    expect(buildTelegramUnreadCounterPolicyRequest(chat, true).hideUnreadCounter).toBe(true)
  })

  it('normalizes an optional provider folder id', () => {
    expect(selectedTelegramProviderFolderId(3)).toBe(3)
    expect(selectedTelegramProviderFolderId(null)).toBeUndefined()
  })
})

function telegramChat(): TelegramChat {
  return {
    account_id: 'account-1', provider_chat_id: 'provider-chat-1', telegram_chat_id: 'telegram-chat-1',
    chat_kind: 'group', title: 'Group', username: null, metadata: {},
  } as TelegramChat
}
