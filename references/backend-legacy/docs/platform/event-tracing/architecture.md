# Event Tracing Architecture

Status: target architecture with partial backend implementation.

## Purpose

Event Tracing reconstructs causal chains from canonical events. It explains how
observations, provider/source signals, domain events, workflow events and
projection events relate to one another.

## Position

Event Tracing belongs to `platform/events`.

Timeline Engine is a chronological projection engine. It may display trace
links, but it must not reconstruct or own traces.

## High-Level Flow

```text
Root Event
  -> Derived Event
  -> Domain Event
  -> Workflow Event
  -> Projection Event / Domain Object Event
```

Communication example:

```text
observation.captured.v1
  -> signal.raw.telegram.message.observed
  -> signal.accepted.telegram.message
  -> communication.message.recorded
  -> radar.signal.detected
  -> review.item.promoted
  -> task.created
```

WhatsApp example:

```text
observation.captured.v1
  -> signal.raw.whatsapp.message.observed
  -> signal.accepted.whatsapp.message
  -> communication.message.recorded
  -> radar.signal.detected
```

Mail example:

```text
observation.captured.v1
  -> signal.raw.mail.message.observed
  -> signal.accepted.mail.message
  -> communication.message.recorded
  -> document.import.requested
```

Calendar example:

```text
signal.calendar.event.observed
  -> calendar.event.recorded
  -> timeline.projection.updated
  -> meeting.preparation.requested
```

## Layer Ownership

| Layer | Owns | Must not own |
|---|---|---|
| `platform/events` | envelope, event store, trace context, trace reconstruction, event API support | business meaning |
| `platform/observations` | root evidence observations | provider protocol logic |
| `domains/signal_hub` | source control state, replay, mute/pause/health policy | communication messages |
| `integrations/*` | provider protocol, runtime, session and command execution | domain state |
| `domains/communications` | canonical messages, conversations, attachments and outbox | provider runtime sessions |
| `workflows/*` | cross-domain orchestration | direct store mutation outside owner |
| `engines/timeline` | chronological projections | causal trace reconstruction |
| `app/*` | HTTP, ConnectRPC, SSE and WebSocket surfaces | business causation decisions |

Trace reconstruction belongs to `platform/events`.

Timeline projection belongs to `engines/timeline`.

Timeline may display trace links, but must not be the trace source of truth.

## Backend Layers

```text
NewEventEnvelopeBuilder
  -> TraceContext
  -> EventStore append
  -> EventStore trace queries
  -> Trace API / realtime surfaces
```

Consumer state, retry metadata and DLQ records annotate trace execution. They
do not create business facts unless a domain explicitly emits an event.

## Frontend Layers

Trace UI belongs to shared platform/event observability surfaces, not provider
product pages.

Allowed locations:

```text
frontend/src/platform/events/*
frontend/src/platform/event-tracing/*
```

Provider runtime UI may link to a trace. Telegram and WhatsApp do not own trace
state, trace query keys or trace transport.

Provider-neutral query keys:

```text
['events', eventId, 'trace']
['event-traces', correlationId]
['events', eventId, 'children']
```

## Event Contracts

Every persistent event has:

```text
event_id
event_type
schema_version
occurred_at
recorded_at
source
actor
subject
payload
provenance
causation_id
correlation_id
```

Root events have no parent but still have a trace id. Derived events inherit
trace id from their parent and set parent span id through `causation_id`.

## API

Trace APIs are platform-level event APIs:

```text
GET /api/v1/events/{event_id}/trace
GET /api/v1/event-traces/{correlation_id}
GET /api/v1/events/{event_id}/children
```

## Testing

Trace tests should cover builder normalization, `TraceContext`, trace graph
reconstruction, missing parents, observation roots, provider fixture chains,
realtime payload fields and DLQ annotations.

## Operations

Operators debug from the domain object back to provenance:

```text
domain object
  -> provenance event id
  -> trace API
  -> root observation
  -> derived signal and workflow chain
  -> consumer and DLQ annotations
```

## Blockers/Gaps

See [gap-analysis.md](gap-analysis.md).

## Status

See [status.md](status.md).
