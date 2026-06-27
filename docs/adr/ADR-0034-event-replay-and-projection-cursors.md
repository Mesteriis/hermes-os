# ADR-0034 Event Replay and Projection Cursors

Status: Proposed

## Context

ADR-0001 makes the event log the system spine. ADR-0023 requires projections and indexes to be rebuildable from canonical events. Rebuildable projections need a durable replay position and a safe way to resume after interruption.

## Decision

Use the `event_log.position` identity column as the durable replay order. Projection workers read events using `list_after_position(after_position, limit)` and persist progress in `projection_cursors`.

Projection cursor updates are monotonic: saving a lower position must not move a projection backward.

## Consequences

- Projection workers can resume after interruption.
- Replay order is independent from wall-clock timestamps.
- Rebuild workflows can reset or create projection-specific cursors intentionally.
- Future concurrent projection workers may need lease/lock semantics, which are not part of this ADR.
