import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram realtime cache patch handling', () => {
  it('patches cached telegram message reaction summary for telegram reaction events', () => {
    const messageKey = ['telegram', 'messages', 'account-1', 'chat-1', 50]
    const messages = [
      {
        message_id: 'tg-msg-1',
        raw_record_id: 'raw-1',
        account_id: 'account-1',
        provider_message_id: 'provider-1',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: 'Hello',
        occurred_at: '2026-06-16T09:00:00Z',
        projected_at: '2026-06-16T09:00:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {}
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(messages) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[messageKey, messages]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-57',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.reaction.changed',
            subject: { id: 'tg-msg-1', kind: 'telegram_message' },
            payload: {
              reaction_emoji: '👍',
              is_active: true
            }
          }
        })
      },
      queryClient
    )

    const patchedItems = setQueryData.mock.results[0]?.value
    expect(patchedItems[0].metadata.reaction_summary.reactions[0]).toMatchObject({
      reaction_emoji: '👍',
      count: 1
    })
  })

  it('patches cached telegram lifecycle metadata for telegram message updated events', () => {
    const messageKey = ['telegram', 'messages', 'account-1', 'chat-1', 50]
    const pinnedKey = ['telegram', 'chats', 'tgchat-1', 'pinned-messages', 100]
    const messages = [
      {
        message_id: 'tg-msg-2',
        raw_record_id: 'raw-2',
        account_id: 'account-1',
        provider_message_id: 'provider-2',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-2',
        sender_display_name: 'Sender',
        text: 'Hello again',
        occurred_at: '2026-06-16T09:05:00Z',
        projected_at: '2026-06-16T09:05:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {}
      }
    ]
    const pinnedResponse = { items: [] }
    const searchKey = ['telegram', 'search', 'messages', 'hello', 'account-1', 'chat-1', 50]
    const searchResponse = { query: 'hello', items: [], total: 0 }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (typeof updater !== 'function') return updater
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) return updater(messages)
      if (JSON.stringify(queryKey) === JSON.stringify(pinnedKey)) return updater(pinnedResponse)
      if (JSON.stringify(queryKey) === JSON.stringify(searchKey)) return updater(searchResponse)
      return updater(undefined)
    })
    const updatedSnapshot = {
      ...messages[0],
      text: 'Hello again',
      metadata: { is_pinned: true, pinned: true }
    }
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['telegram', 'messages'])) {
          return [[messageKey, messages]]
        }
        if (key === JSON.stringify(['telegram', 'chats'])) {
          return [[pinnedKey, pinnedResponse]]
        }
        if (key === JSON.stringify(['telegram', 'search', 'messages'])) {
          return [[searchKey, searchResponse]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-58',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.message.updated',
            subject: { id: 'tg-msg-2', kind: 'telegram_message' },
            payload: {
              version_number: 3,
              is_pinned: true,
              telegram_chat_id: 'tgchat-1',
              message: updatedSnapshot
            }
          }
        })
      },
      queryClient
    )

    const patchedItems = setQueryData.mock.results[0]?.value
    expect(patchedItems[0].metadata.lifecycle.latest_version_number).toBe(3)
    expect(patchedItems[0].metadata.is_pinned).toBe(true)

    const patchedPinned = setQueryData.mock.results[1]?.value
    expect(patchedPinned.items[0].message_id).toBe('tg-msg-2')

    const patchedSearch = setQueryData.mock.results[2]?.value
    expect(patchedSearch.items[0].message_id).toBe('tg-msg-2')
    expect(patchedSearch.total).toBe(1)
  })

  it('upserts telegram message snapshots for telegram created events', () => {
    const messageKey = ['telegram', 'messages', 'account-1', 'chat-1', 50]
    const messages = [
      {
        message_id: 'tg-msg-1',
        raw_record_id: 'raw-1',
        account_id: 'account-1',
        provider_message_id: 'provider-1',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: 'Older message',
        occurred_at: '2026-06-16T09:00:00Z',
        projected_at: '2026-06-16T09:00:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {}
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(messages) : updater
    )
    const snapshot = {
      message_id: 'tg-msg-3',
      raw_record_id: 'raw-3',
      account_id: 'account-1',
      provider_message_id: 'provider-3',
      provider_chat_id: 'chat-1',
      chat_title: 'Chat',
      sender: 'sender-3',
      sender_display_name: 'Sender',
      text: 'Newest message',
      occurred_at: '2026-06-16T10:05:00Z',
      projected_at: '2026-06-16T10:05:01Z',
      channel_kind: 'telegram_user',
      delivery_state: 'sent',
      metadata: {}
    }
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'messages'])) {
          return [[messageKey, messages]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-60',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.message.created',
            subject: { id: 'tg-msg-3', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              message: snapshot
            }
          }
        })
      },
      queryClient
    )

    const patchedItems = setQueryData.mock.results[0]?.value
    expect(patchedItems[0].message_id).toBe('tg-msg-3')
    expect(patchedItems).toHaveLength(2)
  })

  it('patches cached telegram chats for telegram created events with chat snapshots', () => {
    const chatsKey = ['telegram', 'chats', 'account-1', 50]
    const chatDetailKey = ['telegram', 'chat-detail', 'telegram_chat:v4:abc']
    const chatList = [
      {
        telegram_chat_id: 'telegram_chat:v4:older',
        account_id: 'account-1',
        provider_chat_id: 'chat-older',
        chat_kind: 'private',
        title: 'Older Chat',
        username: null,
        sync_state: 'synced',
        last_message_at: '2026-06-16T09:00:00Z',
        metadata: { unread_count: 0, mention_count: 0 },
        created_at: '2026-06-16T08:00:00Z',
        updated_at: '2026-06-16T09:00:00Z'
      },
      {
        telegram_chat_id: 'telegram_chat:v4:abc',
        account_id: 'account-1',
        provider_chat_id: 'chat-1',
        chat_kind: 'private',
        title: 'Project Chat',
        username: null,
        sync_state: 'synced',
        last_message_at: '2026-06-16T08:00:00Z',
        metadata: { unread_count: 1, mention_count: 0 },
        created_at: '2026-06-16T07:00:00Z',
        updated_at: '2026-06-16T08:00:00Z'
      }
    ]
    const chatDetail = { ...chatList[1] }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (JSON.stringify(queryKey) === JSON.stringify(chatsKey)) {
        return typeof updater === 'function' ? updater(chatList) : updater
      }
      return typeof updater === 'function' ? updater(chatDetail) : updater
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['telegram', 'messages'])) return []
        if (key === JSON.stringify(['telegram', 'runtime'])) return []
        if (key === JSON.stringify(['telegram', 'search', 'messages'])) return []
        if (key === JSON.stringify(['telegram', 'chats'])) return [[chatsKey, chatList]]
        if (key === JSON.stringify(['telegram', 'chat-detail'])) return [[chatDetailKey, chatDetail]]
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-60a',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.message.created',
            subject: { id: 'tg-msg-3', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              telegram_chat_id: 'telegram_chat:v4:abc',
              chat: {
                telegram_chat_id: 'telegram_chat:v4:abc',
                account_id: 'account-1',
                provider_chat_id: 'chat-1',
                chat_kind: 'private',
                title: 'Project Chat',
                username: null,
                sync_state: 'synced',
                last_message_at: '2026-06-16T10:05:00Z',
                metadata: { unread_count: 2, mention_count: 1 },
                created_at: '2026-06-16T07:00:00Z',
                updated_at: '2026-06-16T10:05:01Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patchedChats = setQueryData.mock.results[0]?.value
    const patchedDetail = setQueryData.mock.results[1]?.value
    expect(patchedChats[0].telegram_chat_id).toBe('telegram_chat:v4:abc')
    expect(patchedChats[0].metadata.unread_count).toBe(2)
    expect(patchedChats[0].metadata.mention_count).toBe(1)
    expect(patchedDetail.last_message_at).toBe('2026-06-16T10:05:00Z')
    expect(patchedDetail.metadata.unread_count).toBe(2)
  })

  it('patches cached telegram runtime status for telegram sync failed events', () => {
    const runtimeKey = ['telegram', 'runtime', 'account-1']
    const runtimeStatus = {
      account_id: 'account-1',
      provider_kind: 'telegram_user',
      runtime_kind: 'fixture',
      status: 'running',
      fixture_runtime: true,
      tdjson_runtime_available: false,
      telegram_app_credentials_configured: false,
      live_send_available: false,
      last_error: null,
      updated_at: '2026-06-16T09:00:00Z'
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(runtimeStatus) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'messages'])) {
          return []
        }
        return [[runtimeKey, runtimeStatus]]
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-59',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.sync.failed',
            metadata: { account_id: 'account-1' },
            payload: {
              scope: 'history',
              status: 'failed',
              synced_count: 12,
              has_more: true,
              provider_chat_id: 'chat-1'
            }
          }
        })
      },
      queryClient
    )

    const patchedRuntime = setQueryData.mock.results[0]?.value
    expect(patchedRuntime.status).toBe('degraded')
    expect(patchedRuntime.last_sync_scope).toBe('history')
    expect(patchedRuntime.last_sync_status).toBe('failed')
    expect(patchedRuntime.last_synced_count).toBe(12)
    expect(patchedRuntime.last_sync_has_more).toBe(true)
    expect(patchedRuntime.last_sync_provider_chat_id).toBe('chat-1')
    expect(patchedRuntime.updated_at).not.toBe(runtimeStatus.updated_at)
  })

  it('patches cached telegram runtime status for telegram command status events', () => {
    const runtimeKey = ['telegram', 'runtime', 'account-1']
    const runtimeStatus = {
      account_id: 'account-1',
      provider_kind: 'telegram_user',
      runtime_kind: 'fixture',
      status: 'running',
      fixture_runtime: true,
      tdjson_runtime_available: false,
      telegram_app_credentials_configured: false,
      live_send_available: false,
      last_error: null,
      updated_at: '2026-06-16T09:00:00Z'
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(runtimeStatus) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'messages'])) return []
        return [[runtimeKey, runtimeStatus]]
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-60',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-1',
              status: 'pinned',
              provider_chat_id: 'chat-1',
              telegram_chat_id: 'telegram_chat:v4:abc',
              message_id: 'msg-1'
            }
          }
        })
      },
      queryClient
    )

    const patchedRuntime = setQueryData.mock.results[0]?.value
    expect(patchedRuntime.status).toBe('running')
    expect(patchedRuntime.last_command_id).toBe('cmd-1')
    expect(patchedRuntime.last_command_status).toBe('pinned')
    expect(patchedRuntime.last_command_provider_chat_id).toBe('chat-1')
    expect(patchedRuntime.last_command_telegram_chat_id).toBe('telegram_chat:v4:abc')
    expect(patchedRuntime.last_command_message_id).toBe('msg-1')
  })

  it('patches cached telegram message and media search results for media download events', () => {
    const messageKey = ['telegram', 'messages', 'account-1', 'chat-1', 50]
    const mediaKey = ['telegram', 'search', 'media', '', 'account-1', 'chat-1', 'all', 100]
    const messages = [
      {
        message_id: 'tg-msg-media-1',
        raw_record_id: 'raw-media-1',
        account_id: 'account-1',
        provider_message_id: 'provider-media-1',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: '',
        occurred_at: '2026-06-16T10:15:00Z',
        projected_at: '2026-06-16T10:15:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {
          attachments: [
            {
              attachment_id: 'att-1',
              attachment_type: 'photo',
              filename: 'before.jpg',
              content_type: 'image/jpeg',
              download_state: 'remote',
            },
          ],
        },
      },
    ]
    const mediaResponse = { query: '', items: [] }
    const downloadedSnapshot = {
      ...messages[0],
      metadata: {
        attachments: [
          {
            attachment_id: 'att-1',
            attachment_type: 'photo',
            filename: 'after.jpg',
            content_type: 'image/jpeg',
            download_state: 'downloaded',
            local_path: '/tmp/after.jpg',
            size: 2048,
          },
        ],
      },
    }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (typeof updater !== 'function') return updater
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) return updater(messages)
      if (JSON.stringify(queryKey) === JSON.stringify(mediaKey)) return updater(mediaResponse)
      return updater(undefined)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['telegram', 'messages'])) return [[messageKey, messages]]
        if (key === JSON.stringify(['telegram', 'search', 'media'])) {
          return [[mediaKey, mediaResponse]]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-62',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.downloaded',
            subject: { id: 'tg-msg-media-1', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-1',
              attachment_id: 'att-1',
              download_state: 'downloaded',
              local_path: '/tmp/after.jpg',
              message: downloadedSnapshot,
            },
          },
        }),
      },
      queryClient
    )

    const patchedMessages = setQueryData.mock.results[0]?.value
    expect(patchedMessages[0].metadata.attachments[0].download_state).toBe('downloaded')
    expect(patchedMessages[0].metadata.attachments[0].local_path).toBe('/tmp/after.jpg')

    const patchedMedia = setQueryData.mock.results[1]?.value
    expect(patchedMedia.items).toHaveLength(1)
    expect(patchedMedia.items[0]).toMatchObject({
      message_id: 'tg-msg-media-1',
      provider_chat_id: 'chat-1',
      file_name: 'after.jpg',
      kind: 'photo',
      mime_type: 'image/jpeg',
      download_state: 'downloaded',
    })
  })

  it('patches cached telegram chat list and detail for dialog command status events', () => {
    const chatsKey = ['telegram', 'chats', 'account-1', 50]
    const chatDetailKey = ['telegram', 'chat-detail', 'telegram_chat:v4:abc']
    const chatList = [
      {
        telegram_chat_id: 'telegram_chat:v4:abc',
        account_id: 'account-1',
        provider_chat_id: 'chat-1',
        chat_kind: 'private',
        title: 'Project Chat',
        username: null,
        sync_state: 'synced',
        last_message_at: '2026-06-16T08:00:00Z',
        metadata: { is_pinned: false, unread_count: 3 },
        created_at: '2026-06-16T07:00:00Z',
        updated_at: '2026-06-16T08:00:00Z'
      }
    ]
    const chatDetail = { ...chatList[0] }
    const setQueryData = vi.fn((queryKey, updater) => {
      const seed = JSON.stringify(queryKey) === JSON.stringify(chatsKey) ? chatList : chatDetail
      return typeof updater === 'function' ? updater(seed) : updater
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'messages'])) return []
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'runtime'])) return []
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'chats'])) {
          return [[chatsKey, chatList]]
        }
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'chat-detail'])) {
          return [[chatDetailKey, chatDetail]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-61',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-2',
              action: 'pin',
              status: 'queued',
              provider_chat_id: 'chat-1',
              telegram_chat_id: 'telegram_chat:v4:abc',
              chat: {
                telegram_chat_id: 'telegram_chat:v4:abc',
                account_id: 'account-1',
                provider_chat_id: 'chat-1',
                chat_kind: 'private',
                title: 'Project Chat',
                username: null,
                sync_state: 'synced',
                last_message_at: '2026-06-16T08:00:00Z',
                metadata: { is_pinned: true, unread_count: 3 },
                created_at: '2026-06-16T07:00:00Z',
                updated_at: '2026-06-16T09:00:00Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patchedChats = setQueryData.mock.results[0]?.value
    const patchedDetail = setQueryData.mock.results[1]?.value
    expect(patchedChats[0].metadata.is_pinned).toBe(true)
    expect(patchedChats[0].updated_at).toBe('2026-06-16T09:00:00Z')
    expect(patchedDetail.metadata.is_pinned).toBe(true)
    expect(patchedDetail.updated_at).toBe('2026-06-16T09:00:00Z')
  })
})
