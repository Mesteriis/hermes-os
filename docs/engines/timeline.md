# Timeline Engine

The Timeline Engine builds chronological views from canonical events and dated
domain records.

Timeline is a view, not a domain.

## Responsibilities

The Timeline Engine produces:

- entity timelines;
- cross-domain timelines;
- period summaries;
- change diffs;
- recency signals;
- timeline gaps.

It does not own:

- calendar events;
- communication messages;
- task state changes;
- project lifecycle;
- source event truth.

## Inputs

- event log entries;
- domain lifecycle events;
- dated communications;
- calendar events;
- document versions;
- task status changes;
- project changes;
- decisions and obligations.

## Output Requirements

Timeline output must preserve:

- source event reference;
- event time and observed/imported time distinction;
- affected entities;
- confidence for inferred dates;
- sorting rules when exact time is unknown.

## Current Implementation Evidence

Timeline-like concepts appear in Personas, Organizations, Calendar and product
UI surfaces. Calendar owns scheduled events. The Timeline Engine owns derived
chronological views across those records.

The first backend baseline lives in `backend/src/engines/timeline.rs`. It owns
shared timeline policy for bounded entity timeline queries, source-backed
timeline event validation and period summaries over source-backed dated event
drafts. It also emits source-backed recency signals for a specific entity by
selecting the latest non-future event relative to an `as_of` time and preserving
the source reference, and detects source-backed gaps between adjacent entity
events when the interval exceeds a caller-provided threshold. It can also diff
two source-backed entity timeline snapshots by source reference to report added
and removed events, and assemble a bounded cross-domain timeline from
source-backed events across entity kinds. It also maps canonical
`StoredEventEnvelope` replay batches into bounded timeline entries while
tracking the last replayed event-log position. A cursor-backed projection
runner baseline now reads canonical events through `EventStore::list_after_position`,
validates them through the Timeline replay mapper, advances
`ProjectionCursorStore` progress and returns derived timeline entries. Persona
relationship events, Organization timeline events and Project detail timelines
now use this shared policy while retaining their current compatibility storage
and API shapes.

## Migration Plan

1. Avoid new domain-specific timeline ownership.
2. Link all timeline views to source events or dated records.
3. Keep Calendar/Events separate from Timeline Engine.
4. Keep compatibility tables as inputs until a schema migration ADR explicitly
   changes persisted event or timeline schemas.
5. Add durable Timeline read-model storage only after a follow-up schema/API
   decision defines which projected views must be persisted instead of rebuilt.
