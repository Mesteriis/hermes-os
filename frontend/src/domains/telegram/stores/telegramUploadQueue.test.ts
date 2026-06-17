import { describe, expect, it } from 'vitest'
import type { TelegramProviderWriteCommand } from '../types/telegram'
import { telegramUploadCommandTitle, telegramUploadQueueCommands } from './telegramUploadQueue'

function command(overrides: Partial<TelegramProviderWriteCommand> = {}): TelegramProviderWriteCommand {
  return {
    command_id: 'cmd-1',
    account_id: 'account-1',
    command_kind: 'send_media',
    idempotency_key: 'idem-1',
    provider_chat_id: 'chat-1',
    provider_message_id: null,
    target_ref: {},
    payload: { filename: 'report.pdf', attachment_id: 'att-1', blob_id: 'blob-1' },
    capability_state: 'available',
    action_class: 'provider_write',
    confirmation_decision: 'not_required',
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
    updated_at: '2026-06-17T09:00:00Z',
    ...overrides
  }
}

describe('telegramUploadQueue', () => {
  it('returns only current-chat media uploads that still need user attention', () => {
    const commands = [
      command({ command_id: 'cmd-visible-1', updated_at: '2026-06-17T09:10:00Z' }),
      command({ command_id: 'cmd-completed', status: 'completed', updated_at: '2026-06-17T09:12:00Z' }),
      command({ command_id: 'cmd-other-chat', provider_chat_id: 'chat-2', updated_at: '2026-06-17T09:11:00Z' }),
      command({ command_id: 'cmd-visible-2', status: 'retrying', updated_at: '2026-06-17T09:13:00Z' }),
      command({ command_id: 'cmd-visible-3', status: 'dead_letter', updated_at: '2026-06-17T09:14:00Z' }),
      command({ command_id: 'cmd-visible-4', status: 'failed', updated_at: '2026-06-17T09:15:00Z' }),
    ]

    expect(telegramUploadQueueCommands(commands, 'chat-1', 3).map((item) => item.command_id)).toEqual([
      'cmd-visible-4',
      'cmd-visible-3',
      'cmd-visible-2'
    ])
  })

  it('builds a stable display title from filename, attachment id, blob id, then fallback', () => {
    expect(telegramUploadCommandTitle(command())).toBe('report.pdf')
    expect(telegramUploadCommandTitle(command({ payload: { attachment_id: 'att-only' } }))).toBe('att-only')
    expect(telegramUploadCommandTitle(command({ payload: { blob_id: 'blob-only' } }))).toBe('blob blob-only')
    expect(telegramUploadCommandTitle(command({ payload: {} }))).toBe('Media upload')
  })
})
