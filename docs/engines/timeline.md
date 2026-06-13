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
shared timeline policy for bounded entity timeline queries and source-backed
timeline event validation. Persona relationship events, Organization timeline
events and Project detail timelines now use this shared policy while retaining
their current compatibility storage and API shapes.

## Migration Plan

1. Avoid new domain-specific timeline ownership.
2. Link all timeline views to source events or dated records.
3. Keep Calendar/Events separate from Timeline Engine.
4. Keep compatibility tables as inputs until a schema migration ADR explicitly
   changes persisted event or timeline schemas.
5. Expand the shared engine from policy validation into event-log replay,
   cross-domain timelines, period summaries, diffs, recency signals and gap
   detection.
