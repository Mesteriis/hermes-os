import { describe, expect, it } from 'vitest'
import {
  telegramCanMarkMessageRead,
  telegramLatestReadableProviderMessageId,
  telegramProviderMessageNumericId,
  telegramThreadReadProgress
} from './telegramReadProgress'

describe('telegram read progress helpers', () => {
  it('parses numeric TDLib message ids from provider message identifiers', () => {
    expect(telegramProviderMessageNumericId('chat-1:42')).toBe(42)
    expect(telegramProviderMessageNumericId('42')).toBe(42)
    expect(telegramProviderMessageNumericId('chat-1:not-a-number')).toBeNull()
  })

  it('marks a boundary after the last visible provider-read message when unread messages follow', () => {
    const progress = telegramThreadReadProgress(
      {
        telegram_chat_id: 'tgchat-1',
        account_id: 'account-1',
        provider_chat_id: 'chat-1',
        chat_kind: 'private',
        title: 'Chat',
        username: null,
        sync_state: 'synced',
        last_message_at: null,
        metadata: { last_read_inbox_provider_message_id: 'chat-1:11' },
        created_at: '2026-06-17T09:00:00Z',
        updated_at: '2026-06-17T09:00:00Z',
      },
      [
        {
          message_id: 'msg-10',
          raw_record_id: 'raw-10',
          account_id: 'account-1',
          provider_message_id: 'chat-1:10',
          provider_chat_id: 'chat-1',
          chat_title: 'Chat',
          sender: 'user-1',
          sender_display_name: 'User',
          text: 'older',
          occurred_at: '2026-06-17T09:00:10Z',
          projected_at: '2026-06-17T09:00:11Z',
          channel_kind: 'telegram_user',
          delivery_state: 'received',
          metadata: {},
        },
        {
          message_id: 'msg-11',
          raw_record_id: 'raw-11',
          account_id: 'account-1',
          provider_message_id: 'chat-1:11',
          provider_chat_id: 'chat-1',
          chat_title: 'Chat',
          sender: 'user-1',
          sender_display_name: 'User',
          text: 'last read',
          occurred_at: '2026-06-17T09:00:12Z',
          projected_at: '2026-06-17T09:00:13Z',
          channel_kind: 'telegram_user',
          delivery_state: 'received',
          metadata: {},
        },
        {
          message_id: 'msg-12',
          raw_record_id: 'raw-12',
          account_id: 'account-1',
          provider_message_id: 'chat-1:12',
          provider_chat_id: 'chat-1',
          chat_title: 'Chat',
          sender: 'user-1',
          sender_display_name: 'User',
          text: 'unread',
          occurred_at: '2026-06-17T09:00:14Z',
          projected_at: '2026-06-17T09:00:15Z',
          channel_kind: 'telegram_user',
          delivery_state: 'received',
          metadata: {},
        },
      ]
    )

    expect(progress.lastReadMessageId).toBe('msg-11')
    expect(progress.boundaryAfterMessageId).toBe('msg-11')
    expect(progress.hasUnreadAfterBoundary).toBe(true)
  })

  it('does not invent a divider when the visible thread is fully read', () => {
    const progress = telegramThreadReadProgress(
      {
        telegram_chat_id: 'tgchat-1',
        account_id: 'account-1',
        provider_chat_id: 'chat-1',
        chat_kind: 'private',
        title: 'Chat',
        username: null,
        sync_state: 'synced',
        last_message_at: null,
        metadata: { last_read_inbox_provider_message_id: 'chat-1:99' },
        created_at: '2026-06-17T09:00:00Z',
        updated_at: '2026-06-17T09:00:00Z',
      },
      [
        {
          message_id: 'msg-10',
          raw_record_id: 'raw-10',
          account_id: 'account-1',
          provider_message_id: 'chat-1:10',
          provider_chat_id: 'chat-1',
          chat_title: 'Chat',
          sender: 'user-1',
          sender_display_name: 'User',
          text: 'older',
          occurred_at: '2026-06-17T09:00:10Z',
          projected_at: '2026-06-17T09:00:11Z',
          channel_kind: 'telegram_user',
          delivery_state: 'received',
          metadata: {},
        },
      ]
    )

    expect(progress.lastReadMessageId).toBe('msg-10')
    expect(progress.boundaryAfterMessageId).toBeNull()
    expect(progress.hasUnreadAfterBoundary).toBe(false)
  })

  it('selects the latest visible incoming provider message id for mark-read requests', () => {
    const latest = telegramLatestReadableProviderMessageId(
      {
        telegram_chat_id: 'tgchat-1',
        account_id: 'account-1',
        provider_chat_id: 'chat-1',
        chat_kind: 'private',
        title: 'Chat',
        username: null,
        sync_state: 'synced',
        last_message_at: null,
        metadata: {},
        created_at: '2026-06-17T09:00:00Z',
        updated_at: '2026-06-17T09:00:00Z',
      },
      [
        {
          message_id: 'msg-10',
          raw_record_id: 'raw-10',
          account_id: 'account-1',
          provider_message_id: 'chat-1:10',
          provider_chat_id: 'chat-1',
          chat_title: 'Chat',
          sender: 'self',
          sender_display_name: 'Self',
          text: 'outbound',
          occurred_at: '2026-06-17T09:00:10Z',
          projected_at: '2026-06-17T09:00:11Z',
          channel_kind: 'telegram_user',
          delivery_state: 'sent',
          metadata: {},
        },
        {
          message_id: 'msg-11',
          raw_record_id: 'raw-11',
          account_id: 'account-1',
          provider_message_id: 'chat-1:11',
          provider_chat_id: 'chat-1',
          chat_title: 'Chat',
          sender: 'user-1',
          sender_display_name: 'User',
          text: 'incoming',
          occurred_at: '2026-06-17T09:00:12Z',
          projected_at: '2026-06-17T09:00:13Z',
          channel_kind: 'telegram_user',
          delivery_state: 'received',
          metadata: {},
        },
      ]
    )

    expect(latest).toBe('chat-1:11')
  })

  it('only exposes message-level mark-read for the latest visible incoming unread message', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private' as const,
      title: 'Chat',
      username: null,
      sync_state: 'synced' as const,
      last_message_at: null,
      metadata: { unread_count: 2 },
      created_at: '2026-06-17T09:00:00Z',
      updated_at: '2026-06-17T09:00:00Z',
    }
    const messages = [
      {
        message_id: 'msg-10',
        raw_record_id: 'raw-10',
        account_id: 'account-1',
        provider_message_id: 'chat-1:10',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'user-1',
        sender_display_name: 'User',
        text: 'older incoming',
        occurred_at: '2026-06-17T09:00:10Z',
        projected_at: '2026-06-17T09:00:11Z',
        channel_kind: 'telegram_user' as const,
        delivery_state: 'received',
        metadata: {},
      },
      {
        message_id: 'msg-11',
        raw_record_id: 'raw-11',
        account_id: 'account-1',
        provider_message_id: 'chat-1:11',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'self',
        sender_display_name: 'Self',
        text: 'outbound',
        occurred_at: '2026-06-17T09:00:12Z',
        projected_at: '2026-06-17T09:00:13Z',
        channel_kind: 'telegram_user' as const,
        delivery_state: 'sent',
        metadata: {},
      },
      {
        message_id: 'msg-12',
        raw_record_id: 'raw-12',
        account_id: 'account-1',
        provider_message_id: 'chat-1:12',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'user-2',
        sender_display_name: 'User Two',
        text: 'latest incoming',
        occurred_at: '2026-06-17T09:00:14Z',
        projected_at: '2026-06-17T09:00:15Z',
        channel_kind: 'telegram_user' as const,
        delivery_state: 'received',
        metadata: {},
      },
    ]

    expect(telegramCanMarkMessageRead(chat, messages, messages[0])).toBe(false)
    expect(telegramCanMarkMessageRead(chat, messages, messages[1])).toBe(false)
    expect(telegramCanMarkMessageRead(chat, messages, messages[2])).toBe(true)
  })
})
