import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

function applyQueryDataUpdate<TData>(
	current: TData | undefined,
	updater: TData | ((data: TData | undefined) => TData | undefined)
): TData | undefined {
	if (typeof updater !== 'function') return updater
	const applyUpdater = updater as (data: TData | undefined) => TData | undefined
	return applyUpdater(current)
}

describe('telegram media realtime cache patch handling', () => {
  it('patches cached telegram message and media search results for started and progress events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const mediaKey = ['communications', 'telegram', 'search', 'media', '', 'account-1', 'chat-1', 'all', 100]
    const message = {
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
    }
    const mediaItem = {
      message_id: 'tg-msg-media-1',
      provider_message_id: 'provider-media-1',
      provider_chat_id: 'chat-1',
      file_name: 'before.jpg',
      kind: 'photo',
      mime_type: 'image/jpeg',
      size_bytes: null,
      occurred_at: '2026-06-16T10:15:00Z',
      download_state: 'remote',
      tdlib_file_id: null,
      provider_attachment_id: 'att-1',
      local_path: null,
    }
    let currentMessages = [message]
    let currentMediaResponse = { query: '', items: [mediaItem] }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) {
        currentMessages = applyQueryDataUpdate(currentMessages, updater) ?? currentMessages
        return currentMessages
      }
      if (JSON.stringify(queryKey) === JSON.stringify(mediaKey)) {
        currentMediaResponse =
          applyQueryDataUpdate(currentMediaResponse, updater) ?? currentMediaResponse
        return currentMediaResponse
      }
      return applyQueryDataUpdate(undefined, updater)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'messages'])) return [[messageKey, currentMessages]]
        if (key === JSON.stringify(['communications', 'telegram', 'search', 'media'])) {
          return [[mediaKey, currentMediaResponse]]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-media-started',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.started',
            subject: { id: 'provider-media-1', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-1',
              provider_attachment_id: 'att-1',
              tdlib_file_id: 9001,
              download_state: 'requested',
            },
          },
        }),
      },
      queryClient
    )

    handleRealtimeEvent(
      {
        id: 'tg-media-progress',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.progress',
            subject: { id: 'provider-media-1', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-1',
              provider_attachment_id: 'att-1',
              tdlib_file_id: 9001,
              download_state: 'downloading',
              expected_size_bytes: 4096,
              downloaded_size_bytes: 1024,
              is_downloading_active: true,
              is_downloading_completed: false,
            },
          },
        }),
      },
      queryClient
    )

    expect(currentMessages[0].metadata.attachments[0]).toMatchObject({
      attachment_id: 'att-1',
      tdlib_file_id: 9001,
      download_state: 'downloading',
      expected_size_bytes: 4096,
      downloaded_size_bytes: 1024,
      is_downloading_active: true,
      is_downloading_completed: false,
    })
    expect(currentMediaResponse.items[0]).toMatchObject({
      provider_attachment_id: 'att-1',
      tdlib_file_id: 9001,
      download_state: 'downloading',
    })
  })

  it('patches cached telegram message and media search results for failed download events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const mediaKey = ['communications', 'telegram', 'search', 'media', '', 'account-1', 'chat-1', 'all', 100]
    const messages = [
      {
        message_id: 'tg-msg-media-2',
        raw_record_id: 'raw-media-2',
        account_id: 'account-1',
        provider_message_id: 'provider-media-2',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: '',
        occurred_at: '2026-06-16T10:16:00Z',
        projected_at: '2026-06-16T10:16:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {
          attachments: [
            {
              attachment_id: 'att-2',
              attachment_type: 'photo',
              filename: 'failed.jpg',
              content_type: 'image/jpeg',
              tdlib_file_id: 9002,
              download_state: 'downloading',
            },
          ],
        },
      },
    ]
    const mediaResponse = {
      query: '',
      items: [
        {
          message_id: 'tg-msg-media-2',
          provider_message_id: 'provider-media-2',
          provider_chat_id: 'chat-1',
          file_name: 'failed.jpg',
          kind: 'photo',
          mime_type: 'image/jpeg',
          size_bytes: null,
          occurred_at: '2026-06-16T10:16:00Z',
          download_state: 'downloading',
          tdlib_file_id: 9002,
          provider_attachment_id: 'att-2',
          local_path: null,
        },
      ],
    }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) {
        return applyQueryDataUpdate(messages, updater)
      }
      if (JSON.stringify(queryKey) === JSON.stringify(mediaKey)) {
        return applyQueryDataUpdate(mediaResponse, updater)
      }
      return applyQueryDataUpdate(undefined, updater)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'messages'])) return [[messageKey, messages]]
        if (key === JSON.stringify(['communications', 'telegram', 'search', 'media'])) {
          return [[mediaKey, mediaResponse]]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-media-failed',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.failed',
            subject: { id: 'provider-media-2', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-2',
              provider_attachment_id: 'att-2',
              tdlib_file_id: 9002,
              download_state: 'failed',
              error: 'tdlib timeout',
            },
          },
        }),
      },
      queryClient
    )

    const patchedMessages = setQueryData.mock.results[0]?.value
    expect(patchedMessages[0].metadata.attachments[0]).toMatchObject({
      download_state: 'failed',
      last_error: 'tdlib timeout',
    })

    const patchedMedia = setQueryData.mock.results[1]?.value
    expect(patchedMedia.items[0]).toMatchObject({
      download_state: 'failed',
      tdlib_file_id: 9002,
    })
  })

  it('patches cached telegram message and media search results for completed download events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const mediaKey = ['communications', 'telegram', 'search', 'media', '', 'account-1', 'chat-1', 'all', 100]
    const messages = [
      {
        message_id: 'tg-msg-media-3',
        raw_record_id: 'raw-media-3',
        account_id: 'account-1',
        provider_message_id: 'provider-media-3',
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
              attachment_id: 'att-3',
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
            attachment_id: 'att-3',
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
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) {
        return applyQueryDataUpdate(messages, updater)
      }
      if (JSON.stringify(queryKey) === JSON.stringify(mediaKey)) {
        return applyQueryDataUpdate(mediaResponse, updater)
      }
      return applyQueryDataUpdate(undefined, updater)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'messages'])) return [[messageKey, messages]]
        if (key === JSON.stringify(['communications', 'telegram', 'search', 'media'])) {
          return [[mediaKey, mediaResponse]]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-media-downloaded',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.downloaded',
            subject: { id: 'tg-msg-media-3', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-3',
              attachment_id: 'att-3',
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
      message_id: 'tg-msg-media-3',
      provider_chat_id: 'chat-1',
      file_name: 'after.jpg',
      kind: 'photo',
      mime_type: 'image/jpeg',
      download_state: 'downloaded',
    })
  })
})
