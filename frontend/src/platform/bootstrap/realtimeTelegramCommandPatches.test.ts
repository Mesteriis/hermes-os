import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram command realtime cache patch handling', () => {
  it('patches cached telegram command rows for provider reconciliation events', () => {
    const commandsKey = ['telegram', 'commands', 'account-1']
    const commands = [
      {
        command_id: 'cmd-reconciled-1',
        account_id: 'account-1',
        command_kind: 'edit',
        idempotency_key: 'idem-1',
        provider_chat_id: 'chat-1',
        provider_message_id: 'provider-msg-1',
        target_ref: {},
        payload: {},
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'not_required',
        status: 'executing',
        retry_count: 1,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-17T09:00:00Z',
        next_attempt_at: null,
        last_attempt_at: '2026-06-17T09:00:00Z',
        locked_at: null,
        locked_by: null,
        provider_observed_at: null,
        provider_state: {},
        reconciliation_status: 'awaiting_provider',
        reconciled_at: null,
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-17T09:00:00Z',
        updated_at: '2026-06-17T09:00:00Z'
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-reconciled',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.reconciled',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-reconciled-1',
              status: 'completed',
              provider_chat_id: 'chat-1',
              message_id: 'provider-msg-1',
              provider_observed_at: '2026-06-17T09:00:05Z',
              reconciliation_status: 'observed',
              reconciled_at: '2026-06-17T09:00:05Z'
            }
          }
        })
      },
      queryClient
    )

    const patchedCommands = setQueryData.mock.results[0]?.value
    expect(patchedCommands[0].status).toBe('completed')
    expect(patchedCommands[0].reconciliation_status).toBe('observed')
    expect(patchedCommands[0].provider_observed_at).toBe('2026-06-17T09:00:05Z')
    expect(patchedCommands[0].reconciled_at).toBe('2026-06-17T09:00:05Z')
  })
})
