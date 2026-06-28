# Event Tracing Operations

## Purpose

Event Tracing gives local operators and developers a deterministic way to
explain domain state.

## Questions Trace Must Answer

- Why does this task exist?
- Which provider event created this message?
- Which observation is the root evidence?
- Which consumer failed?
- Can this trace be replayed?
- Which events are missing parent links?

## Debug Flow

```text
Open task
  -> get provenance event id
  -> fetch trace
  -> inspect root observation
  -> inspect workflow chain
  -> inspect consumer annotations
```

Communication debug flow:

```text
Open communication message
  -> read message provenance event id
  -> GET /api/v1/events/{event_id}/trace
  -> inspect provider/source signal chain
  -> inspect Signal Hub accepted/rejected state
  -> inspect consumer/DLQ annotations
```

## Legacy Events

Events created before trace normalization may have null `correlation_id`. They
should be displayed as `legacy_orphan_trace` unless migrated.

Do not rewrite append-only event rows casually. Any backfill must have a safe
migration plan, clear validation and rollback posture.

## Replay

Replay decisions use `event_log` and consumer state. Replaying a trace should
not invent missing parent links. Missing parent ids are data quality findings
that need explicit repair or acceptance.

## Privacy

Trace responses must sanitize private payloads. Trace-specific records must not
store raw message bodies, secrets, provider cookies, access tokens or session
material.
