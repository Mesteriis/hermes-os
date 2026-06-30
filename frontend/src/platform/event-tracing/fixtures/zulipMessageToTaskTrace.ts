import type { EventEnvelope, EventTrace, StoredEventEnvelope } from '../types'

const traceId = 'trace:fixture:zulip-message-to-task'
const recordedAt = '2026-06-30T00:49:02.805Z'

export const zulipMessageToTaskTraceEventTypes = [
  'signal.raw.zulip.message.observed',
  'signal.accepted.zulip.message',
  'communication.message.recorded',
  'task.candidate.detected.v1',
  'review.item.available.v1',
] as const

export const zulipMessageToTaskTraceFixture: EventTrace = {
  correlation_id: traceId,
  root_event_ids: ['event:fixture:zulip:raw-message'],
  events: [
    stored(1, event({
      event_id: 'event:fixture:zulip:raw-message',
      event_type: 'signal.raw.zulip.message.observed',
      source: {
        kind: 'signal_hub',
        provider: 'zulip',
        account_id: 'zulip-fixture-account',
      },
      subject: {
        kind: 'provider_message',
        provider: 'zulip',
        raw_record_id: 'raw:fixture:zulip:message:1',
        provider_record_id: '1001',
      },
      payload: {
        event_kind: 'message',
        channel_kind: 'zulip',
        provider_conversation_id: 'Hermes Lab / Tasks',
        body_excerpt: 'Надо проверить backup до пятницы.',
        credential_payload_present: false,
      },
      provenance: {
        fixture: 'zulip_message_to_task_trace',
        sanitized: true,
      },
    })),
    stored(2, event({
      event_id: 'event:fixture:zulip:accepted-message',
      event_type: 'signal.accepted.zulip.message',
      source: {
        kind: 'signal_hub',
        provider: 'zulip',
      },
      subject: {
        kind: 'raw_signal',
        provider: 'zulip',
        raw_record_id: 'raw:fixture:zulip:message:1',
      },
      payload: {
        accepted: true,
        schema: 'signal.accepted.zulip.message',
      },
      provenance: {
        source_event_id: 'event:fixture:zulip:raw-message',
      },
      causation_id: 'event:fixture:zulip:raw-message',
    })),
    stored(3, event({
      event_id: 'event:fixture:communication:message-recorded',
      event_type: 'communication.message.recorded',
      source: {
        kind: 'communication_projection',
        consumer: 'communication_provider_observation_projection',
      },
      subject: {
        kind: 'communication_message',
        id: 'message:fixture:zulip:1001',
        entity_id: 'message:fixture:zulip:1001',
        message_id: 'message:fixture:zulip:1001',
      },
      payload: {
        channel_kind: 'zulip',
        provider_record_id: '1001',
        provider_observation_event_id: 'event:fixture:zulip:accepted-message',
      },
      provenance: {
        ownership: 'communications_projection',
        source_event_id: 'event:fixture:zulip:accepted-message',
      },
      causation_id: 'event:fixture:zulip:accepted-message',
    })),
    stored(4, event({
      event_id: 'event:fixture:review:task-candidate',
      event_type: 'task.candidate.detected.v1',
      source: {
        domain: 'ingestion',
        source_id: 'task.candidate.detected.v1:review:fixture:task',
      },
      subject: {
        review_item_id: 'review:fixture:task',
        item_kind: 'potential_task',
      },
      payload: {
        title: 'Надо проверить backup до пятницы.',
        confidence: 0.82,
        evidence_observation_ids: ['observation:fixture:zulip:message:1001'],
      },
      provenance: {
        observation_ingestion: true,
      },
      causation_id: 'event:fixture:communication:message-recorded',
    })),
    stored(5, event({
      event_id: 'event:fixture:review:item-available',
      event_type: 'review.item.available.v1',
      source: {
        domain: 'review',
        source_id: 'review.item.available.v1:review:fixture:task',
      },
      subject: {
        review_item_id: 'review:fixture:task',
        item_kind: 'potential_task',
      },
      payload: {
        status: 'new',
        confidence: 0.82,
        evidence_observation_ids: ['observation:fixture:zulip:message:1001'],
      },
      provenance: {
        review_inbox: true,
      },
      causation_id: 'event:fixture:review:task-candidate',
    })),
  ],
  edges: [
    edge('event:fixture:zulip:raw-message', 'event:fixture:zulip:accepted-message'),
    edge('event:fixture:zulip:accepted-message', 'event:fixture:communication:message-recorded'),
    edge('event:fixture:communication:message-recorded', 'event:fixture:review:task-candidate'),
    edge('event:fixture:review:task-candidate', 'event:fixture:review:item-available'),
  ],
  orphan_event_ids: [],
  missing_parent_ids: [],
  consumer_annotations: [
    {
      event_id: 'event:fixture:zulip:accepted-message',
      consumer_name: 'communications_projection',
      status: 'processed',
      processed_at: recordedAt,
      attempts: 1,
    },
    {
      event_id: 'event:fixture:communication:message-recorded',
      consumer_name: 'review_task_candidate_refresh',
      status: 'processed',
      processed_at: recordedAt,
      attempts: 1,
    },
  ],
  dead_letters: [],
}

function stored(position: number, event: EventEnvelope): StoredEventEnvelope {
  return { position, event }
}

function event(input: {
  event_id: string
  event_type: string
  source: Record<string, unknown>
  subject: Record<string, unknown>
  payload: Record<string, unknown>
  provenance: Record<string, unknown>
  causation_id?: string
}): EventEnvelope {
  return {
    event_id: input.event_id,
    event_type: input.event_type,
    schema_version: 1,
    occurred_at: recordedAt,
    recorded_at: recordedAt,
    source: input.source,
    actor: null,
    subject: input.subject,
    payload: input.payload,
    provenance: input.provenance,
    causation_id: input.causation_id ?? null,
    correlation_id: traceId,
  }
}

function edge(parent_event_id: string, child_event_id: string) {
  return { parent_event_id, child_event_id }
}
