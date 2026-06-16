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
        completed_at: null,
        created_at: '2026-06-16T10:00:00Z',
        updated_at: '2026-06-16T10:00:00Z',
      })
    ).toBe('provider_write · available · confirmed')
  })
})
