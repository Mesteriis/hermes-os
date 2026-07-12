import { describe, expect, it } from 'vitest'
import {
  telegramMessengerConversation,
  telegramMessengerListItem,
} from './communicationMessengerWorkspaceModels'

describe('telegram messenger list presentation', () => {
  it('exposes provider unread and dialog-state signals to the shared list item', () => {
    const item = telegramMessengerListItem({
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'telegram-account',
      provider_chat_id: 'provider-chat-1',
      chat_kind: 'group',
      title: 'Release room',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: {
        unread_count: 4,
        provider_unread_mention_count: 2,
        is_pinned: true,
        is_muted: true,
      },
      created_at: '2026-07-12T10:00:00Z',
      updated_at: '2026-07-12T10:00:00Z',
    }, 'telegram-chat-1')

    expect(item).toMatchObject({
      unreadCount: 4,
      mentionCount: 2,
      pinned: true,
      muted: true,
      workflowState: 'muted',
      selected: true,
    })
  })

  it('renders an archived provider dialog as archived even when it is muted', () => {
    const item = telegramMessengerListItem({
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'telegram-account',
      provider_chat_id: 'provider-chat-1',
      chat_kind: 'private',
      title: 'Saved messages',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { is_archived: true, is_muted: true },
      created_at: '2026-07-12T10:00:00Z',
      updated_at: '2026-07-12T10:00:00Z',
    }, '')

    expect(item.workflowState).toBe('archived')
    expect(item.muted).toBe(true)
  })

  it('hides only the unread badge when a chat display policy requests it', () => {
    const item = telegramMessengerListItem({
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'telegram-account',
      provider_chat_id: 'provider-chat-1',
      chat_kind: 'private',
      title: 'Quiet chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { unread_count: 4, hide_unread_counter: true },
      created_at: '2026-07-12T10:00:00Z',
      updated_at: '2026-07-12T10:00:00Z',
    }, '')

    expect(item.unreadCount).toBeUndefined()
    expect(item.mentionCount).toBeUndefined()
  })

  it('passes a locally loaded Telegram chat avatar to the shared list item', () => {
    const item = telegramMessengerListItem({
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'telegram-account',
      provider_chat_id: 'provider-chat-1',
      chat_kind: 'private',
      title: 'Avatar chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: {},
      created_at: '2026-07-12T10:00:00Z',
      updated_at: '2026-07-12T10:00:00Z',
    }, '', 'blob:hermes-avatar-1')

    expect(item.profile).toEqual({
      displayName: 'Avatar chat',
      src: 'blob:hermes-avatar-1',
    })
  })

  it('uses the projected last-message preview instead of sync state in the list', () => {
    const item = telegramMessengerListItem({
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'telegram-account',
      provider_chat_id: 'provider-chat-1',
      chat_kind: 'group',
      title: 'Release room',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { last_message_preview: 'The release is ready' },
      created_at: '2026-07-12T10:00:00Z',
      updated_at: '2026-07-12T10:00:00Z',
    }, '')

    expect(item.preview).toBe('The release is ready')
  })

  it('renders projected Telegram attachment metadata in the message reader', () => {
    const conversation = telegramMessengerConversation({
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'telegram-account',
      provider_chat_id: 'provider-chat-1',
      chat_kind: 'private',
      title: 'Saved messages',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: {},
      created_at: '2026-07-12T10:00:00Z',
      updated_at: '2026-07-12T10:00:00Z',
    }, [{
      message_id: 'message-1',
      raw_record_id: 'raw-1',
      account_id: 'telegram-account',
      provider_message_id: 'provider-message-1',
      provider_chat_id: 'provider-chat-1',
      chat_title: 'Saved messages',
      sender: 'user:1',
      sender_display_name: 'Owner',
      text: 'A file',
      occurred_at: '2026-07-12T10:00:00Z',
      projected_at: '2026-07-12T10:00:00Z',
      channel_kind: 'telegram_user',
      delivery_state: 'received',
      metadata: {
        reaction_summary: {
          reactions: [{ reaction_emoji: '👍', count: 3, is_chosen: true }],
        },
        attachments: [{
          attachment_id: 'tdlib:document:42',
          attachment_type: 'document',
          content_type: 'application/pdf',
          filename: 'brief.pdf',
          download_state: 'remote',
          tdlib_file_id: 42,
        }],
      },
    }], 'message-1')

    expect(conversation.messages[0]?.attachments).toEqual([{
      id: 'tdlib:document:42',
      name: 'brief.pdf',
      meta: 'document · application/pdf · remote',
      icon: 'tabler:file-description',
      downloadable: true,
      providerMessageId: 'provider-message-1',
      tdlibFileId: 42,
      contentType: 'application/pdf',
    }])
    expect(conversation.messages[0]?.reactions).toEqual([{
      emoji: '👍',
      count: 3,
      active: true,
    }])
  })

  it('keeps Telegram outbound delivery state instead of claiming it was delivered', () => {
    const conversation = telegramMessengerConversation({
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'telegram-account',
      provider_chat_id: 'provider-chat-1',
      chat_kind: 'private',
      title: 'Saved messages',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: {},
      created_at: '2026-07-12T10:00:00Z',
      updated_at: '2026-07-12T10:00:00Z',
    }, [{
      message_id: 'message-sent',
      raw_record_id: 'raw-sent',
      account_id: 'telegram-account',
      provider_message_id: 'provider-message-sent',
      provider_chat_id: 'provider-chat-1',
      chat_title: 'Saved messages',
      sender: 'user:1',
      sender_display_name: 'Owner',
      text: 'Sent text',
      occurred_at: '2026-07-12T10:00:00Z',
      projected_at: '2026-07-12T10:00:00Z',
      channel_kind: 'telegram_user',
      delivery_state: 'sent',
      metadata: {},
    }, {
      message_id: 'message-blocked',
      raw_record_id: 'raw-blocked',
      account_id: 'telegram-account',
      provider_message_id: 'provider-message-blocked',
      provider_chat_id: 'provider-chat-1',
      chat_title: 'Saved messages',
      sender: 'user:1',
      sender_display_name: 'Owner',
      text: 'Blocked text',
      occurred_at: '2026-07-12T10:01:00Z',
      projected_at: '2026-07-12T10:01:00Z',
      channel_kind: 'telegram_user',
      delivery_state: 'send_blocked',
      metadata: {},
    }])

    expect(conversation.messages).toMatchObject([
      { direction: 'outbound', deliveryStatus: 'sent', pending: false },
      { direction: 'outbound', deliveryStatus: 'failed', deliveryStatusLabel: 'Blocked', pending: false },
    ])
  })
})
