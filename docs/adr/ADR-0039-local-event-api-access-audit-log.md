# ADR-0039 Local Event API Access Audit Log

Status: Proposed

## Context

ADR-0038 protects local event API reads and writes with a temporary API capability token. Protection without durable auditability still leaves an operational gap: authorized event reads and writes cannot be reviewed later.

The canonical `event_log` is the domain event spine. Recording API reads as canonical events would make query operations mutate replay state and projection cursors, which would mix operational access audit with domain history.

## Decision

Create a separate append-only `api_audit_log` table for local event API access attempts.

Rules:

- Authorized `POST /api/events` records an `event.append` audit attempt before appending the canonical event.
- Authorized `GET /api/events/{event_id}` records an `event.get` audit attempt before loading the event.
- Audit records store operation, method, path template, target kind, target ID, actor kind and metadata.
- API tokens and secrets are never stored in audit records.
- Audit logging is fail-closed: if the audit insert fails, the API operation is not performed.
- `GET /api/audit/events` exposes protected read-only audit inspection with optional `target_id`, `after_audit_id` and `limit` query parameters.
- Audit inspection uses monotonic `audit_id` cursor pagination; `after_audit_id` returns records with greater audit IDs.
- Reading audit records is not itself recorded in `api_audit_log` to avoid unbounded self-audit noise.
- `api_audit_log` is append-only and must reject updates and deletes.

## Consequences

- Local event API reads and writes are reviewable.
- Audit review requires the same temporary local API capability token as event API access.
- Query operations no longer need to pollute canonical event replay streams to become auditable.
- Audit records are operational security data, not domain events.
- Future capability runtime can replace `actor_kind = local_api_token` with richer actor/capability identifiers.
