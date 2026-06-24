# Event Tracing Status

Status date: 2026-06-24.

## Implemented

- ADR-0100 records Trace-First Event Observability.
- Canonical event builder normalizes missing or empty `correlation_id` to
  `event_id`.
- `TraceContext` supports root and child contexts.
- EventStore exposes trace by event id, trace by correlation id and children by
  causation id.
- Trace reconstruction returns roots, edges, orphan events, missing parents,
  consumer annotations and DLQ annotations.
- Observation capture writes `observation.captured.v1` as a root trace event in
  the current store path.
- Raw Mail, Telegram and WhatsApp source signal builders set causation to the
  deterministic observation captured event id in current raw-record paths.
- Signal Hub derived events already set `causation_id = raw_event.event_id` and
  inherit correlation when present.
- Communications emits canonical `communication.message.recorded` or
  `communication.message.updated` events after accepted Mail, Telegram and
  WhatsApp message signals mutate the communication projection.
- Event trace API endpoints exist under platform event routes.
- Shared frontend trace UI exists under `frontend/src/platform/event-tracing/`
  and uses provider-neutral `events` / `event-traces` query keys.
- Telegram, WhatsApp and Mail fixture tests cover complete chains from
  observation to communication event.

## Partially Implemented

- Realtime payloads expose trace fields, but in-memory bus events do not always
  have a stored `recorded_at`.
- Provider observation events support explicit correlation context; derived
  provider observations retain the parent correlation when a caller supplies
  it, while root provider observations may still use their idempotency key as
  trace id.
- Legacy events are readable as orphan traces but are not backfilled.
- Domain detail pages can link to the shared trace surface in future slices;
  trace state itself is already provider-neutral and platform-owned.

## Planned

- DLQ annotation API regression tests.
- Optional OpenTelemetry trace exporter sourced from `event_log`.

## Blocked

- Full live-provider trace validation is blocked on deterministic provider
  fixture coverage and should not require live accounts.

## Deprecated / Superseded

- Treating OpenTelemetry as canonical trace storage is superseded by ADR-0100.
- Provider-specific trace ownership is rejected.
