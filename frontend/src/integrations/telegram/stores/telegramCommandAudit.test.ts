import { describe, expect, it } from 'vitest'
import {
  isTelegramCommandDeadLetter,
  telegramCommandAuditState,
  telegramCommandSubject,
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

  it('shows upload progress detail for executing media commands when provider state supplies it', () => {
    const executing = command({
      command_kind: 'send_media',
      status: 'executing',
      provider_state: {
        upload_phase: 'dispatching_to_provider',
        progress_detail: 'Uploading local media to Telegram',
      },
    })

    expect(telegramCommandAuditState(executing)).toMatchObject({
      label: 'Executing',
      detail: 'Uploading local media to Telegram',
      tone: 'progress',
      is_dead_letter: false,
    })
  })

  it('formats targeted mark-read commands as readable progress instead of raw message ids', () => {
    const executing = command({
      command_kind: 'mark_read',
      status: 'executing',
      provider_message_id: 'chat-1:777',
    })
    const completed = command({
      command_kind: 'mark_read',
      status: 'completed',
      provider_message_id: 'chat-1:777',
      provider_state: {
        last_read_inbox_message_id: 'chat-1:778',
      },
    })

    expect(telegramCommandSubject(executing)).toBe('Read through chat-1:777')
    expect(telegramCommandAuditState(executing).detail).toBe('Read through chat-1:777')
    expect(telegramCommandAuditState(completed).detail).toBe('Read through chat-1:778')
  })

  it('formats mark-unread commands without leaking provider-specific placeholders', () => {
    const commandRow = command({
      command_kind: 'mark_unread',
      provider_message_id: null,
    })

    expect(telegramCommandSubject(commandRow)).toBe('Mark chat unread')
  })

  it('formats folder add/remove commands as readable chat-folder actions', () => {
    const addQueued = command({
      command_kind: 'folder_add',
      provider_message_id: null,
      payload: {
        provider_folder_id: 7,
      },
    })
    const removeCompleted = command({
      command_kind: 'folder_remove',
      provider_message_id: null,
      status: 'completed',
      payload: {
        provider_folder_id: 9,
      },
      provider_state: {
        provider_folder_id: 9,
      },
    })

    expect(telegramCommandSubject(addQueued)).toBe('Add chat to folder 7')
    expect(telegramCommandAuditState(addQueued).detail).toBe('Add to folder 7')
    expect(telegramCommandSubject(removeCompleted)).toBe('Remove chat from folder 9')
    expect(telegramCommandAuditState(removeCompleted).detail).toBe(
      'Folder 9 removal observed on provider'
    )
  })

  it('describes provider-observed mark-unread mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'mark_unread',
      provider_message_id: null,
      status: 'failed',
      last_error: 'Provider observed a different unread state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_marked_as_unread: false,
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Mark chat unread')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · chat is still read',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes edit reconciliation from provider-observed text state', () => {
    const queued = command({
      command_kind: 'edit',
      payload: {
        new_text: 'Queued provider edit body',
      },
    })
    const completed = command({
      command_kind: 'edit',
      status: 'completed',
      provider_state: {
        body_text: 'Provider observed edited body',
      },
    })

    expect(telegramCommandSubject(queued)).toBe('Edit message')
    expect(telegramCommandAuditState(queued).detail).toBe('Target text · 25 chars')
    expect(telegramCommandAuditState(completed).detail).toBe(
      'Provider text observed · 29 chars'
    )
  })

  it('describes provider-observed edit mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'edit',
      status: 'failed',
      last_error: 'Provider observed a different message body than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        expected_body_text: 'Expected provider edit body',
        observed_body_text: 'Observed provider body',
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Edit message')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · expected 27 chars, observed 22 chars',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes delete reconciliation from provider-observed tombstone state', () => {
    const queued = command({
      command_kind: 'delete',
      payload: {
        reason_class: 'deleted_by_owner',
      },
    })
    const completed = command({
      command_kind: 'delete',
      status: 'completed',
      provider_state: {
        is_deleted: true,
      },
    })

    expect(telegramCommandSubject(queued)).toBe('Delete message')
    expect(telegramCommandAuditState(queued).detail).toBe(
      'Delete requested · deleted_by_owner'
    )
    expect(telegramCommandAuditState(completed).detail).toBe('Provider delete observed')
  })

  it('describes reaction reconciliation from provider-observed chosen state', () => {
    const queued = command({
      command_kind: 'react',
      payload: {
        reaction_emoji: '👍',
      },
    })
    const completed = command({
      command_kind: 'unreact',
      status: 'completed',
      provider_state: {
        reaction_emoji: '👍',
        is_chosen: false,
      },
    })

    expect(telegramCommandSubject(queued)).toBe('Add reaction 👍')
    expect(telegramCommandAuditState(queued).detail).toBe('Add reaction 👍')
    expect(telegramCommandSubject(completed)).toBe('Remove reaction 👍')
    expect(telegramCommandAuditState(completed).detail).toBe(
      'Reaction 👍 absent on provider'
    )
  })

  it('describes provider-observed reaction mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'react',
      status: 'failed',
      last_error: 'Provider observed a different reaction state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        reaction_emoji: '👍',
        observed_is_chosen: false,
      },
      payload: {
        reaction_emoji: '👍',
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Add reaction 👍')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · reaction 👍 is still absent',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes provider-observed pin mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'unpin',
      status: 'failed',
      last_error: 'Provider observed a different pin state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_pinned: true,
      },
      payload: {
        is_pinned: false,
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Unpin message')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · message is still pinned',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('distinguishes dialog pin commands from message pin commands in user-facing subjects', () => {
    const chatPin = command({
      command_kind: 'pin',
      provider_message_id: null,
    })
    const chatUnpinMismatch = command({
      command_kind: 'unpin',
      provider_message_id: null,
      status: 'failed',
      last_error: 'Provider observed a different dialog pin state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_pinned: true,
      },
    })

    expect(telegramCommandSubject(chatPin)).toBe('Pin chat')
    expect(telegramCommandSubject(chatUnpinMismatch)).toBe('Unpin chat')
    expect(telegramCommandAuditState(chatUnpinMismatch).detail).toBe(
      'Provider mismatch · chat is still pinned'
    )
  })

  it('describes provider-observed archive mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'unarchive',
      provider_message_id: null,
      status: 'failed',
      last_error: 'Provider observed a different archive state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_archived: true,
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Unarchive chat')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · chat is still archived',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes provider-observed mute mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'unmute',
      provider_message_id: null,
      status: 'failed',
      last_error: 'Provider observed a different mute state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_muted: true,
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Unmute chat')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · chat is still muted',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes participant lifecycle reconciliation from provider roster evidence', () => {
    const joinCompleted = command({
      command_kind: 'join',
      status: 'completed',
      provider_state: {
        membership_state: 'present',
      },
    })
    const leaveCompleted = command({
      command_kind: 'leave',
      status: 'completed',
      provider_state: {
        membership_state: 'absent_exhaustive',
      },
    })

    expect(telegramCommandAuditState(joinCompleted).detail).toBe('Joined chat')
    expect(telegramCommandAuditState(leaveCompleted).detail).toBe(
      'Left chat (confirmed by full provider roster)'
    )
  })
})
