# Event Model

## Purpose

The event model is the system spine. It records facts that happened in or around Hermes Hub and lets projections build current state, graph links, indexes and user-facing timelines.

## Event Categories

- communication events
- document events
- task events
- calendar events
- person events
- project events
- agent events
- system events
- security and permission events

## Canonical Event Envelope

```json
{
  "event_id": "01HERMES...",
  "event_type": "message_received",
  "schema_version": 1,
  "occurred_at": "2026-06-04T12:00:00Z",
  "recorded_at": "2026-06-04T12:00:05Z",
  "source": {
    "kind": "email",
    "provider": "imap",
    "source_id": "provider-message-id",
    "import_batch_id": "01BATCH..."
  },
  "actor": {
    "kind": "external_contact",
    "entity_id": "person_..."
  },
  "subject": {
    "kind": "message",
    "entity_id": "message_..."
  },
  "payload": {},
  "provenance": {
    "raw_record_id": "raw_...",
    "confidence": 1.0
  },
  "causation_id": null,
  "correlation_id": "01FLOW..."
}
```

## Required Properties

- Append-only by default.
- Idempotent ingestion using provider source IDs and import batch IDs.
- Explicit schema version.
- Separate occurred_at and recorded_at.
- Traceable causation and correlation IDs.
- Provenance for imported and AI-derived facts.
- Projection rebuild support.

## Event Examples

- `message_received`
- `message_sent`
- `message_classified`
- `document_uploaded`
- `document_version_created`
- `document_ocr_completed`
- `entity_extracted`
- `task_created`
- `task_status_changed`
- `meeting_completed`
- `person_created`
- `relationship_created`
- `payment_received`
- `agent_tool_invoked`
- `permission_granted`

## Projections

Events feed projections for:

- message threads
- unified timeline
- person profiles
- project timelines
- task views
- graph edges
- full text index
- semantic index
- agent memory traces

Projection failures must be observable and replayable. A broken projection must not corrupt the canonical event log.
