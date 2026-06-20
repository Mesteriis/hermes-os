import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

function queryClientForChat(chat: Record<string, unknown>) {
  const chatsKey = ['communications', 'telegram', 'chats', 'account-1', 50]
  const chatDetailKey = ['communications', 'telegram', 'chat-detail', 'tgchat-1']
  const setQueryData = vi.fn((queryKey, updater) => {
    if (typeof updater !== 'function') return updater
    if (JSON.stringify(queryKey) === JSON.stringify(chatsKey)) return updater([chat])
    if (JSON.stringify(queryKey) === JSON.stringify(chatDetailKey)) return updater(chat)
    return updater(undefined)
  })
  return {
    invalidateQueries: vi.fn(),
    getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
      const key = JSON.stringify(queryKey)
      if (key === JSON.stringify(['communications', 'telegram', 'chats'])) return [[chatsKey, [chat]]]
      if (key === JSON.stringify(['communications', 'telegram', 'chat-detail'])) return [[chatDetailKey, chat]]
      return []
    }),
    setQueryData,
  }
}

describe('telegram provider chat.updated cache patching', () => {
  it('patches cached chat snapshots for provider notification settings updates', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { is_muted: false },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = { ...chat, metadata: { ...chat.metadata, is_muted: true } }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-mute',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_notification_settings_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.is_muted).toBe(true)
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.is_muted).toBe(true)
  })

  it('patches cached chat snapshots for provider chat position updates', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { is_archived: false, is_pinned: false },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = {
      ...chat,
      metadata: { ...chat.metadata, is_archived: true, is_pinned: true, provider_folder_id: 7 },
    }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-position',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_chat_position_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.is_archived).toBe(true)
    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.is_pinned).toBe(true)
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.provider_folder_id).toBe(7)
  })

  it('patches cached chat snapshots for provider folder label updates', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { folder_labels: ['Unknown folder 7'], folder_name: 'Unknown folder 7' },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = {
      ...chat,
      metadata: {
        ...chat.metadata,
        folder_labels: ['Projects'],
        folder_name: 'Projects',
        provider_folder_id: 7,
      },
    }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-folders',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_chat_folder_labels_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.folder_labels).toEqual([
      'Projects',
    ])
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.folder_name).toBe('Projects')
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.provider_folder_id).toBe(7)
  })

  it('replaces cached chat snapshots when provider folder labels fall back to unknown', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { folder_labels: ['Projects'], folder_name: 'Projects', provider_folder_id: 7 },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = {
      ...chat,
      metadata: {
        folder_labels: ['Unknown folder 7'],
        folder_name: 'Unknown folder 7',
        provider_folder_id: 7,
      },
    }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-folders-fallback',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_chat_folder_labels_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.folder_labels).toEqual([
      'Unknown folder 7',
    ])
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.folder_name).toBe(
      'Unknown folder 7'
    )
  })
})
