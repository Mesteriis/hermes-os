import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('realtime mail provider command patches', () => {
  it('patches a visible command immediately and invalidates authoritative diagnostics', () => {
    const queryKey = [
      'communications',
      'mail',
      'provider-command-diagnostics',
      'account-1',
      null
    ] as const
    const diagnostics = {
      items: [
        {
          command_id: 'command-1',
          account_id: 'account-1',
          command_kind: 'mark_read',
          message_id: 'message-1',
          status: 'queued',
          retry_count: 0,
          max_retries: 3,
          reconciliation_status: 'not_observed',
          next_attempt_at: null,
          last_attempt_at: null,
          dead_lettered_at: null,
          last_error: null,
          created_at: '2026-07-11T09:00:00Z',
          updated_at: '2026-07-11T09:00:00Z'
        }
      ],
      counts: [
        { status: 'queued', count: 1 },
        { status: 'executing', count: 0 }
      ]
    }
    const setQueryData = vi.fn((key, updater) =>
      typeof updater === 'function' ? updater(diagnostics) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[queryKey, diagnostics]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'provider-command-event-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'communication.provider_command.executing.v1',
            occurred_at: '2026-07-11T09:01:00Z',
            payload: {
              command_id: 'command-1',
              account_id: 'account-1',
              status: 'executing',
              retry_count: 1,
              max_retries: 3,
              reconciliation_status: 'not_observed',
              next_attempt_at: null,
              dead_lettered_at: null
            }
          }
        })
      },
      queryClient
    )

    const patched = setQueryData.mock.results[0]?.value
    expect(patched.items[0]).toMatchObject({
      command_id: 'command-1',
      status: 'executing',
      retry_count: 1,
      last_attempt_at: '2026-07-11T09:01:00Z',
      updated_at: '2026-07-11T09:01:00Z'
    })
    expect(patched.counts).toEqual([
      { status: 'queued', count: 0 },
      { status: 'executing', count: 1 }
    ])
    expect(queryClient.invalidateQueries).toHaveBeenCalledOnce()
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'mail', 'provider-command-diagnostics']
    })
  })
})
