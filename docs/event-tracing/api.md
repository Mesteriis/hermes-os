# Event Tracing API

## Purpose

The Event Tracing API exposes causal traces reconstructed from `event_log`.

## Position

Trace APIs are platform event APIs. Provider integrations may link to them but
must not own provider-specific trace endpoints.

## Endpoints

```text
GET /api/v1/events/{event_id}/trace
GET /api/v1/event-traces/{correlation_id}
GET /api/v1/events/{event_id}/children
```

`GET /api/v1/events/{event_id}/trace` resolves the anchor event and then loads
its trace by `correlation_id`. If the anchor is a legacy event with null
`correlation_id`, the event id is used as the legacy trace id.

`GET /api/v1/event-traces/{correlation_id}` loads a trace directly by trace id.

`GET /api/v1/events/{event_id}/children` returns events where
`causation_id = event_id`.

## Response Shape

```json
{
  "correlation_id": "obs_123",
  "root_event_ids": [
    "event:v1:observation-captured:obs_123"
  ],
  "events": [
    {
      "position": 1,
      "event": {
        "event_id": "event:v1:observation-captured:obs_123",
        "event_type": "observation.captured.v1",
        "schema_version": 1,
        "occurred_at": "2026-06-24T10:00:00Z",
        "recorded_at": "2026-06-24T10:00:01Z",
        "source": {},
        "actor": null,
        "subject": {},
        "payload": {},
        "provenance": {},
        "causation_id": null,
        "correlation_id": "obs_123"
      }
    }
  ],
  "edges": [
    {
      "parent_event_id": "event:v1:observation-captured:obs_123",
      "child_event_id": "signal_raw_telegram_..."
    }
  ],
  "orphan_event_ids": [],
  "missing_parent_ids": [],
  "consumer_annotations": [],
  "dead_letters": []
}
```

## Realtime Payload Requirements

Realtime event payloads must include:

- `event_id`;
- `event_type`;
- `schema_version`;
- `occurred_at`;
- `recorded_at` when available from stored events;
- `source`;
- `actor`;
- `subject`;
- sanitized `payload`;
- `provenance`;
- `causation_id`;
- `correlation_id`.

Private content must remain sanitized. Trace-specific structures must not store
secrets, tokens, raw private blobs or provider session material.

## Errors

Missing anchor event returns `404` from the event-id trace endpoint.

Invalid empty event id or trace id returns the existing platform validation
error mapping.

## Frontend Surface

The frontend event tracing surface lives in
`frontend/src/platform/event-tracing/`. It calls the platform event endpoints
with provider-neutral query keys:

```text
['events', eventId, 'trace']
['event-traces', correlationId]
['events', eventId, 'children']
```

Provider-specific runtime pages may link to event traces, but they must not own
trace state or provider-specific trace query namespaces.
