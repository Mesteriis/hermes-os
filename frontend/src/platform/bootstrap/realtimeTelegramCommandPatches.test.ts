import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'
import { TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE } from '../../integrations/telegram/queries/telegramRuntimePanelActions'

describe('telegram command realtime cache patch handling', () => {
  it('patches cached telegram command rows for retry scheduling and dead-letter fields', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1']
    const commands = [
      {
        command_id: 'cmd-retry-1',
        account_id: 'account-1',
        command_kind: 'send_media',
        idempotency_key: 'idem-retry-1',
        provider_chat_id: 'chat-1',
        provider_message_id: null,
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
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-retrying',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-retry-1',
              status: 'retrying',
              retry_count: 2,
              last_error: 'temporary tdlib failure',
              next_attempt_at: '2026-06-17T09:01:30Z'
            }
          }
        })
      },
      queryClient
    )

    const retryingCommands = setQueryData.mock.results[0]?.value
    expect(retryingCommands[0]).toMatchObject({
      status: 'retrying',
      retry_count: 2,
      last_error: 'temporary tdlib failure',
      next_attempt_at: '2026-06-17T09:01:30Z'
    })

    handleRealtimeEvent(
      {
        id: 'tg-command-dead-letter',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-retry-1',
              status: 'dead_letter',
              retry_count: 3,
              last_error: 'permanent tdlib failure',
              dead_lettered_at: '2026-06-17T09:02:00Z'
            }
          }
        })
      },
      queryClient
    )

    const deadLetteredCommands = setQueryData.mock.results[1]?.value
    expect(deadLetteredCommands[0]).toMatchObject({
      status: 'dead_letter',
      retry_count: 3,
      last_error: 'permanent tdlib failure',
      dead_lettered_at: '2026-06-17T09:02:00Z'
    })
  })

  it('patches cached telegram command rows for provider reconciliation events', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1']
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
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
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
              retry_count: 1,
              provider_chat_id: 'chat-1',
              message_id: 'provider-msg-1',
              provider_observed_at: '2026-06-17T09:00:05Z',
              provider_state: {
                source_event: 'tdlib.updateMessageContent'
              },
              result_payload: {
                projection_message_id: 'msg-1'
              },
              reconciliation_status: 'observed',
              reconciled_at: '2026-06-17T09:00:05Z',
              completed_at: '2026-06-17T09:00:05Z'
            }
          }
        })
      },
      queryClient
    )

    const patchedCommands = setQueryData.mock.results[0]?.value
    expect(patchedCommands[0].status).toBe('completed')
    expect(patchedCommands[0].retry_count).toBe(1)
    expect(patchedCommands[0].reconciliation_status).toBe('observed')
    expect(patchedCommands[0].provider_observed_at).toBe('2026-06-17T09:00:05Z')
    expect(patchedCommands[0].reconciled_at).toBe('2026-06-17T09:00:05Z')
    expect(patchedCommands[0].completed_at).toBe('2026-06-17T09:00:05Z')
    expect(patchedCommands[0].provider_state).toMatchObject({
      source_event: 'tdlib.updateMessageContent'
    })
    expect(patchedCommands[0].result_payload).toMatchObject({
      projection_message_id: 'msg-1'
    })
  })

  it('patches cached telegram command rows for provider mismatch reconciliation events', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1']
    const commands = [
      {
        command_id: 'cmd-edit-mismatch-1',
        account_id: 'account-1',
        command_kind: 'edit',
        idempotency_key: 'idem-edit-mismatch-1',
        provider_chat_id: 'chat-1',
        provider_message_id: 'provider-msg-1',
        target_ref: {},
        payload: {
          new_text: 'Expected provider edit body'
        },
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
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-mismatch',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.reconciled',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-edit-mismatch-1',
              status: 'failed',
              retry_count: 1,
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-msg-1',
              provider_observed_at: '2026-06-17T09:00:05Z',
              provider_state: {
                expected_body_text: 'Expected provider edit body',
                observed_body_text: 'Observed provider body'
              },
              result_payload: {
                expected_body_text: 'Expected provider edit body',
                observed_body_text: 'Observed provider body',
                mismatch: true
              },
              last_error: 'Provider observed a different message body than requested',
              reconciliation_status: 'mismatch',
              reconciled_at: '2026-06-17T09:00:05Z',
              completed_at: null
            }
          }
        })
      },
      queryClient
    )

    const patchedCommands = setQueryData.mock.results[0]?.value
    expect(patchedCommands[0].status).toBe('failed')
    expect(patchedCommands[0].reconciliation_status).toBe('mismatch')
    expect(patchedCommands[0].last_error).toBe(
      'Provider observed a different message body than requested'
    )
    expect(patchedCommands[0].provider_state).toMatchObject({
      expected_body_text: 'Expected provider edit body',
      observed_body_text: 'Observed provider body'
    })
    expect(patchedCommands[0].result_payload).toMatchObject({
      expected_body_text: 'Expected provider edit body',
      observed_body_text: 'Observed provider body',
      mismatch: true
    })
    expect(patchedCommands[0].completed_at).toBeNull()
    expect(patchedCommands[0].reconciled_at).toBe('2026-06-17T09:00:05Z')
  })

  it('inserts a queued send_media command row when media upload starts before command query refetch', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1', TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE]
    const commands: Array<Record<string, unknown>> = []
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-upload-started',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.upload.started',
            payload: {
              command_id: 'cmd-upload-1',
              account_id: 'account-1',
              provider_chat_id: 'chat-1',
              command_kind: 'send_media',
              idempotency_key: 'idem-upload-1',
              capability_state: 'available',
              action_class: 'provider_write',
              confirmation_decision: 'confirmed',
              status: 'queued',
              retry_count: 0,
              max_retries: 3,
              reconciliation_status: 'not_observed',
              actor_id: 'hermes-frontend',
              happened_at: '2026-06-17T09:00:00Z',
              created_at: '2026-06-17T09:00:00Z',
              updated_at: '2026-06-17T09:00:00Z',
              payload: {
                attachment_id: 'att-1',
                blob_id: 'blob-1',
                media_type: 'document',
                filename: 'upload-note.txt'
              },
              target_ref: {
                provider_chat_id: 'chat-1',
                attachment_id: 'att-1'
              }
            }
          }
        })
      },
      queryClient
    )

    const insertedCommands = setQueryData.mock.results[0]?.value
    expect(insertedCommands).toHaveLength(1)
    expect(insertedCommands[0]).toMatchObject({
      command_id: 'cmd-upload-1',
      account_id: 'account-1',
      command_kind: 'send_media',
      provider_chat_id: 'chat-1',
      status: 'queued',
      max_retries: 3,
      reconciliation_status: 'not_observed'
    })
    expect(insertedCommands[0].payload).toMatchObject({
      attachment_id: 'att-1',
      blob_id: 'blob-1',
      filename: 'upload-note.txt'
    })
  })

  it('patches cached send_media command rows with upload progress provider-state detail', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1']
    const commands = [
      {
        command_id: 'cmd-upload-progress-1',
        account_id: 'account-1',
        command_kind: 'send_media',
        idempotency_key: 'idem-upload-progress-1',
        provider_chat_id: 'chat-1',
        provider_message_id: null,
        target_ref: {},
        payload: {
          attachment_id: 'att-1',
          blob_id: 'blob-1',
          filename: 'upload-note.txt'
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'queued',
        retry_count: 0,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-17T09:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: null,
        provider_state: {},
        reconciliation_status: 'not_observed',
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
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-upload-progress',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.upload.progress',
            payload: {
              command_id: 'cmd-upload-progress-1',
              account_id: 'account-1',
              provider_chat_id: 'chat-1',
              status: 'executing',
              retry_count: 1,
              max_retries: 3,
              reconciliation_status: 'not_observed',
              provider_state: {
                upload_phase: 'dispatching_to_provider',
                progress_detail: 'Uploading local media to Telegram'
              }
            }
          }
        })
      },
      queryClient
    )

    const patchedCommands = setQueryData.mock.results[0]?.value
    expect(patchedCommands[0]).toMatchObject({
      status: 'executing',
      retry_count: 1,
      reconciliation_status: 'not_observed'
    })
    expect(patchedCommands[0].provider_state).toMatchObject({
      upload_phase: 'dispatching_to_provider',
      progress_detail: 'Uploading local media to Telegram'
    })
  })

  it('inserts a queued join command row when lifecycle status arrives before command query refetch', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1', TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE]
    const commands: Array<Record<string, unknown>> = []
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-join-queued',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            payload: {
              command_id: 'cmd-join-1',
              account_id: 'account-1',
              provider_chat_id: 'chat-1',
              telegram_chat_id: 'tgchat-1',
              action: 'join',
              status: 'queued'
            }
          }
        })
      },
      queryClient
    )

    const insertedCommands = setQueryData.mock.results[0]?.value
    expect(insertedCommands).toHaveLength(1)
    expect(insertedCommands[0]).toMatchObject({
      command_id: 'cmd-join-1',
      account_id: 'account-1',
      command_kind: 'join',
      provider_chat_id: 'chat-1',
      status: 'queued'
    })
    expect(insertedCommands[0].payload).toMatchObject({
      action: 'join'
    })
  })

  it('inserts a queued edit command row when lifecycle status arrives before command query refetch', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1', TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE]
    const commands: Array<Record<string, unknown>> = []
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-edit-queued',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            payload: {
              command_id: 'cmd-edit-1',
              account_id: 'account-1',
              command_kind: 'edit',
              action: 'edit',
              provider_chat_id: 'chat-1',
              message_id: 'msg-local-1',
              provider_message_id: 'chat-1:42',
              status: 'queued',
              payload: {
                telegram_message_id: 'msg-local-1',
                new_text: 'Edited body'
              }
            }
          }
        })
      },
      queryClient
    )

    const insertedCommands = setQueryData.mock.results[0]?.value
    expect(insertedCommands).toHaveLength(1)
    expect(insertedCommands[0]).toMatchObject({
      command_id: 'cmd-edit-1',
      account_id: 'account-1',
      command_kind: 'edit',
      provider_chat_id: 'chat-1',
      provider_message_id: 'chat-1:42',
      status: 'queued'
    })
    expect(insertedCommands[0].payload).toMatchObject({
      telegram_message_id: 'msg-local-1',
      new_text: 'Edited body'
    })
  })

  it('inserts a queued delete command row with tombstone metadata before command query refetch', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1', TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE]
    const commands: Array<Record<string, unknown>> = []
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-delete-queued',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            payload: {
              command_id: 'cmd-delete-1',
              account_id: 'account-1',
              command_kind: 'delete',
              action: 'delete',
              provider_chat_id: 'chat-1',
              message_id: 'msg-local-9',
              provider_message_id: 'chat-1:99',
              status: 'queued',
              payload: {
                telegram_message_id: 'msg-local-9',
                reason_class: 'deleted_by_owner',
                tombstone_id: 'tomb-1'
              }
            }
          }
        })
      },
      queryClient
    )

    const insertedCommands = setQueryData.mock.results[0]?.value
    expect(insertedCommands).toHaveLength(1)
    expect(insertedCommands[0]).toMatchObject({
      command_id: 'cmd-delete-1',
      account_id: 'account-1',
      command_kind: 'delete',
      provider_chat_id: 'chat-1',
      provider_message_id: 'chat-1:99',
      status: 'queued'
    })
    expect(insertedCommands[0].payload).toMatchObject({
      telegram_message_id: 'msg-local-9',
      reason_class: 'deleted_by_owner',
      tombstone_id: 'tomb-1'
    })
  })
})
