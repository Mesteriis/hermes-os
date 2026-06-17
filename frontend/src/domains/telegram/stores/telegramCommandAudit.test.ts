import { describe, expect, it } from 'vitest'
import {
  isTelegramCommandDeadLetter,
  telegramCommandAuditState,
  telegramCommandRetrySummary,
} from './telegramCommandAudit'
import type { TelegramProviderWriteCommand } from '../types/telegram'

function command(overrides: Partial<TelegramProviderWriteCommand>): TelegramProviderWriteCommand {
  return {
    command_id: 'cmd-1',
    account_id: 'acct-1',
    command_kind: 'edit',
    idempotency_key: 'idem-1',
    provider_chat_id: 'chat-1',
    provider_message_id: 'msg-1',
    target_ref: {},
    payload: {},
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

describe('telegram command audit projection', () => {
  it('summarizes retry budget without exposing provider internals', () => {
    expect(telegramCommandRetrySummary(command({ retry_count: 1, max_retries: 3 }))).toBe(
      '1/3 retries used'
    )
    expect(telegramCommandRetrySummary(command({ retry_count: 9, max_retries: 3 }))).toBe(
      '3/3 retries used'
    )
    expect(telegramCommandRetrySummary(command({ max_retries: 0 }))).toBe('No retry budget')
  })

  it('marks failed commands with exhausted retry budget as dead-lettered', () => {
    const failed = command({
      status: 'failed',
      retry_count: 3,
      max_retries: 3,
      last_error: 'TDLib request failed',
    })

    expect(isTelegramCommandDeadLetter(failed)).toBe(true)
    expect(telegramCommandAuditState(failed)).toEqual({
      label: 'Dead-lettered',
      detail: 'TDLib request failed',
      tone: 'danger',
      is_dead_letter: true,
    })
  })

  it('keeps retryable failures separate from dead-lettered failures', () => {
    const failed = command({
      status: 'failed',
      retry_count: 1,
      max_retries: 3,
      last_error: 'Transient provider failure',
    })

    expect(isTelegramCommandDeadLetter(failed)).toBe(false)
    expect(telegramCommandAuditState(failed)).toMatchObject({
      label: 'Failed',
      detail: 'Transient provider failure',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('treats explicit durable dead-letter status as final until manual retry', () => {
    const failed = command({
      status: 'dead_letter',
      retry_count: 1,
      max_retries: 3,
      dead_lettered_at: '2026-06-17T10:01:00Z',
      last_error: 'Unsupported command kind',
    })

    expect(isTelegramCommandDeadLetter(failed)).toBe(true)
    expect(telegramCommandAuditState(failed)).toMatchObject({
      label: 'Dead-lettered',
      detail: 'Unsupported command kind',
      tone: 'danger',
      is_dead_letter: true,
    })
  })
})
