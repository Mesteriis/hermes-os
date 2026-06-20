import { describe, expect, it } from 'vitest'
import type { TelegramProviderWriteCommand } from '../types/telegram'
import {
  telegramParticipantLifecycleCommands,
  telegramParticipantLifecycleTitle,
} from './telegramParticipantLifecycle'

function command(overrides: Partial<TelegramProviderWriteCommand>): TelegramProviderWriteCommand {
  return {
    command_id: 'cmd-1',
    account_id: 'acct-1',
    command_kind: 'join',
    idempotency_key: 'idem-1',
    provider_chat_id: 'chat-1',
    provider_message_id: null,
    target_ref: {},
    payload: {},
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
    happened_at: '2026-06-17T10:00:00Z',
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
    created_at: '2026-06-17T10:00:00Z',
    updated_at: '2026-06-17T10:00:00Z',
    ...overrides,
  }
}

describe('telegram participant lifecycle command helpers', () => {
  it('keeps only current-chat join and leave commands in recency order', () => {
    const items = telegramParticipantLifecycleCommands(
      [
        command({ command_id: 'join-old', updated_at: '2026-06-17T10:00:00Z' }),
        command({ command_id: 'leave-new', command_kind: 'leave', updated_at: '2026-06-17T10:01:00Z' }),
        command({ command_id: 'media', command_kind: 'send_media', updated_at: '2026-06-17T10:02:00Z' }),
        command({ command_id: 'other-chat', provider_chat_id: 'chat-2', updated_at: '2026-06-17T10:03:00Z' }),
      ],
      'chat-1',
      3
    )

    expect(items.map((item) => item.command_id)).toEqual(['leave-new', 'join-old'])
  })

  it('maps lifecycle command kinds to stable UI titles', () => {
    expect(telegramParticipantLifecycleTitle(command({ command_kind: 'join' }))).toBe('Join chat')
    expect(telegramParticipantLifecycleTitle(command({ command_kind: 'leave' }))).toBe('Leave chat')
  })
})
