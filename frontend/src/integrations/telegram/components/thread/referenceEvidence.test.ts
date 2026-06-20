import { describe, expect, it } from 'vitest'
import {
  previousTelegramVersionBody,
  summarizeTelegramCommandEvidence,
  summarizeTelegramTombstoneState,
  summarizeTelegramVersionDelta,
} from './referenceEvidence'

describe('telegram reference evidence helpers', () => {
  it('summarizes version deltas against the next stored version body', () => {
    const versions = [
      {
        version_id: 'v2',
        message_id: 'm1',
        account_id: 'a1',
        provider_message_id: 'pm1',
        provider_chat_id: 'pc1',
        version_number: 2,
        body_text: 'updated body text',
        edit_timestamp: '2026-06-16T10:00:00Z',
        source_event: null,
        raw_diff_payload: {},
        provenance: {},
        created_at: '2026-06-16T10:00:00Z',
      },
      {
        version_id: 'v1',
        message_id: 'm1',
        account_id: 'a1',
        provider_message_id: 'pm1',
        provider_chat_id: 'pc1',
        version_number: 1,
        body_text: 'old text',
        edit_timestamp: '2026-06-16T09:00:00Z',
        source_event: null,
        raw_diff_payload: {},
        provenance: {},
        created_at: '2026-06-16T09:00:00Z',
      },
    ]

    const previousBody = previousTelegramVersionBody(versions, 0)
    expect(previousBody).toBe('old text')
    expect(summarizeTelegramVersionDelta(versions[0], previousBody)).toBe('8 -> 17 chars')
    expect(summarizeTelegramVersionDelta(versions[1], previousTelegramVersionBody(versions, 1))).toBe('Initial captured body · 8 chars')
  })

  it('summarizes tombstone visibility and command evidence states', () => {
    expect(
      summarizeTelegramTombstoneState({
        tombstone_id: 't1',
        message_id: 'm1',
        account_id: 'a1',
        provider_message_id: 'pm1',
        provider_chat_id: 'pc1',
        reason_class: 'deleted_by_owner',
        actor_class: 'owner',
        observed_at: '2026-06-16T10:00:00Z',
        source_event: null,
        is_provider_delete: true,
        is_local_visible: true,
        metadata: {},
        provenance: {},
        created_at: '2026-06-16T10:00:00Z',
      })
    ).toBe('Provider delete observed · locally restored')

    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c1',
        account_id: 'a1',
        command_kind: 'edit',
        idempotency_key: 'idem-1',
        provider_chat_id: 'pc1',
        provider_message_id: 'pm1',
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
        happened_at: '2026-06-16T10:00:00Z',
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
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:00Z',
      })
    ).toBe('Queued · 0/3 retries used')
  })

  it('summarizes provider-observed command evidence instead of raw capability metadata', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c2',
        account_id: 'a1',
        command_kind: 'delete',
        idempotency_key: 'idem-2',
        provider_chat_id: 'pc1',
        provider_message_id: 'pm1',
        target_ref: {},
        payload: {
          reason_class: 'deleted_by_owner',
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'completed',
        retry_count: 1,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          is_deleted: true,
        },
        reconciliation_status: 'observed',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: '2026-06-16T10:00:05Z',
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Completed · Provider delete observed')
  })

  it('summarizes folder command evidence with readable folder-specific detail', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c-folder',
        account_id: 'a1',
        command_kind: 'folder_add',
        idempotency_key: 'idem-folder',
        provider_chat_id: 'pc1',
        provider_message_id: null,
        target_ref: {},
        payload: {
          provider_folder_id: 7,
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'completed',
        retry_count: 0,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          provider_folder_id: 7,
        },
        reconciliation_status: 'observed',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: '2026-06-16T10:00:05Z',
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Completed · Folder 7 observed on provider')
  })

  it('summarizes mismatched edit reconciliation outcomes for reference surfaces', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c3',
        account_id: 'a1',
        command_kind: 'edit',
        idempotency_key: 'idem-3',
        provider_chat_id: 'pc1',
        provider_message_id: 'pm1',
        target_ref: {},
        payload: {
          new_text: 'Expected provider edit body',
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'failed',
        retry_count: 1,
        max_retries: 3,
        last_error: 'Provider observed a different message body than requested',
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          expected_body_text: 'Expected provider edit body',
          observed_body_text: 'Observed provider body',
        },
        reconciliation_status: 'mismatch',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Failed · Provider mismatch · expected 27 chars, observed 22 chars')
  })

  it('summarizes mismatched reaction reconciliation outcomes for reference surfaces', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c4',
        account_id: 'a1',
        command_kind: 'react',
        idempotency_key: 'idem-4',
        provider_chat_id: 'pc1',
        provider_message_id: 'pm1',
        target_ref: {},
        payload: {
          reaction_emoji: '👍',
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'failed',
        retry_count: 1,
        max_retries: 3,
        last_error: 'Provider observed a different reaction state than requested',
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          reaction_emoji: '👍',
          observed_is_chosen: false,
        },
        reconciliation_status: 'mismatch',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Failed · Provider mismatch · reaction 👍 is still absent')
  })

  it('summarizes mismatched pin reconciliation outcomes for reference surfaces', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c5',
        account_id: 'a1',
        command_kind: 'unpin',
        idempotency_key: 'idem-5',
        provider_chat_id: 'pc1',
        provider_message_id: 'pm1',
        target_ref: {},
        payload: {
          is_pinned: false,
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'failed',
        retry_count: 1,
        max_retries: 3,
        last_error: 'Provider observed a different pin state than requested',
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          observed_is_pinned: true,
        },
        reconciliation_status: 'mismatch',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Failed · Provider mismatch · message is still pinned')
  })

  it('summarizes dialog pin mismatches separately from message pin mismatches', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c6',
        account_id: 'a1',
        command_kind: 'unpin',
        idempotency_key: 'idem-6',
        provider_chat_id: 'pc1',
        provider_message_id: null,
        target_ref: {},
        payload: {
          is_pinned: false,
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'failed',
        retry_count: 1,
        max_retries: 3,
        last_error: 'Provider observed a different dialog pin state than requested',
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          observed_is_pinned: true,
        },
        reconciliation_status: 'mismatch',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Failed · Provider mismatch · chat is still pinned')
  })

  it('summarizes dialog archive mismatches from provider-observed state', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c7',
        account_id: 'a1',
        command_kind: 'unarchive',
        idempotency_key: 'idem-7',
        provider_chat_id: 'pc1',
        provider_message_id: null,
        target_ref: {},
        payload: {},
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'failed',
        retry_count: 1,
        max_retries: 3,
        last_error: 'Provider observed a different archive state than requested',
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          observed_is_archived: true,
        },
        reconciliation_status: 'mismatch',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Failed · Provider mismatch · chat is still archived')
  })

  it('summarizes dialog mute mismatches from provider-observed state', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c8',
        account_id: 'a1',
        command_kind: 'unmute',
        idempotency_key: 'idem-8',
        provider_chat_id: 'pc1',
        provider_message_id: null,
        target_ref: {},
        payload: {},
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'failed',
        retry_count: 1,
        max_retries: 3,
        last_error: 'Provider observed a different mute state than requested',
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          observed_is_muted: true,
        },
        reconciliation_status: 'mismatch',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Failed · Provider mismatch · chat is still muted')
  })

  it('summarizes dialog mark-unread mismatches from provider-observed state', () => {
    expect(
      summarizeTelegramCommandEvidence({
        command_id: 'c9',
        account_id: 'a1',
        command_kind: 'mark_unread',
        idempotency_key: 'idem-9',
        provider_chat_id: 'pc1',
        provider_message_id: null,
        target_ref: {},
        payload: {},
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'failed',
        retry_count: 1,
        max_retries: 3,
        last_error: 'Provider observed a different unread state than requested',
        result_payload: {},
        audit_metadata: {},
        actor_id: 'hermes-frontend',
        happened_at: '2026-06-16T10:00:00Z',
        next_attempt_at: null,
        last_attempt_at: null,
        locked_at: null,
        locked_by: null,
        provider_observed_at: '2026-06-16T10:00:05Z',
        provider_state: {
          observed_is_marked_as_unread: false,
        },
        reconciliation_status: 'mismatch',
        reconciled_at: '2026-06-16T10:00:05Z',
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:05Z',
      })
    ).toBe('Failed · Provider mismatch · chat is still read')
  })
})
