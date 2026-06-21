import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram realtime topic cache patch handling', () => {
  it('patches cached telegram topic lists for topic update events', () => {
    const topicsKey = ['communications', 'telegram', 'topics', 'telegram-chat-1', 100]
    const topicSearchKey = ['communications', 'telegram', 'topic-search', 'telegram-chat-1', 'release', 50]
    const existingTopic = {
      topic_id: 'telegram-topic-old',
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'account-1',
      provider_topic_id: 7,
      provider_chat_id: '-100123',
      title: 'Older topic',
      icon_emoji: null,
      is_pinned: false,
      is_closed: false,
      unread_count: 0,
      last_message_at: null,
      metadata: {},
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z'
    }
    const updatedTopic = {
      topic_id: 'telegram-topic-42',
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'account-1',
      provider_topic_id: 42,
      provider_chat_id: '-100123',
      title: 'Release notes',
      icon_emoji: '5368324170671202286',
      is_pinned: true,
      is_closed: false,
      unread_count: 0,
      last_message_at: null,
      metadata: {},
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-17T09:00:00Z'
    }
    const topicsResponse = { telegram_chat_id: 'telegram-chat-1', items: [existingTopic] }
    const searchResponse = { telegram_chat_id: 'telegram-chat-1', items: [] }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (typeof updater !== 'function') return updater
      if (JSON.stringify(queryKey) === JSON.stringify(topicsKey)) return updater(topicsResponse)
      if (JSON.stringify(queryKey) === JSON.stringify(topicSearchKey)) return updater(searchResponse)
      return updater(undefined)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['communications', 'telegram'])) {
          return [[topicsKey, topicsResponse], [topicSearchKey, searchResponse]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-topic-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.topic.updated',
            subject: { id: 'telegram-topic-42', kind: 'telegram_topic' },
            payload: {
              account_id: 'account-1',
              telegram_chat_id: 'telegram-chat-1',
              provider_chat_id: '-100123',
              provider_topic_id: 42,
              topic_id: 'telegram-topic-42',
              topic: updatedTopic
            }
          }
        })
      },
      queryClient
    )

    const patchedTopics = setQueryData.mock.results[0]?.value
    expect(patchedTopics.items[0]).toMatchObject({
      topic_id: 'telegram-topic-42',
      title: 'Release notes',
      is_pinned: true
    })

    const patchedSearch = setQueryData.mock.results[1]?.value
    expect(patchedSearch.items[0].topic_id).toBe('telegram-topic-42')
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({ queryKey: ['communications', 'telegram', 'topics'] })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({ queryKey: ['communications', 'telegram', 'topic-search'] })
  })
})
