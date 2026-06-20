import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram command realtime query filters', () => {
  it('inserts command rows only into matching filtered command caches', () => {
    const matchingKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-1', 'chat-1:42', 'mark_read|mark_unread']
    const otherChatKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-2', 'chat-1:42', 'mark_read|mark_unread']
    const otherMessageKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-1', 'chat-1:99', 'mark_read|mark_unread']
    const otherKindKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-1', 'chat-1:42', 'join|leave']
    const commands: Array<Record<string, unknown>> = []

    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [
            [matchingKey, commands],
            [otherChatKey, commands],
            [otherMessageKey, commands],
            [otherKindKey, commands],
          ]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-filter-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            payload: {
              command_id: 'cmd-read-1',
              account_id: 'account-1',
              provider_chat_id: 'chat-1',
              provider_message_id: 'chat-1:42',
              command_kind: 'mark_read',
              status: 'queued',
              retry_count: 0,
              max_retries: 3,
              capability_state: 'available',
              action_class: 'provider_write',
              confirmation_decision: 'confirmed',
            },
          },
        }),
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledTimes(1)
    expect(setQueryData.mock.results[0]?.value).toHaveLength(1)
  })
})
