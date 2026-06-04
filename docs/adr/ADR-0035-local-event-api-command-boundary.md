# ADR-0035 Local Event API Command Boundary

Status: Proposed

## Context

ADR-0015 requires durable mutations to pass through command boundaries. The backend already has an `EventStore`, but without an HTTP/API command boundary the UI and future local clients cannot append or query canonical events through the application layer.

## Decision

Expose a local backend API for canonical event operations:

- `POST /api/events` appends a validated canonical event envelope.
- `GET /api/events/{event_id}` loads a canonical event by ID.

The API uses the same event envelope validation and PostgreSQL idempotency constraints as the storage layer.

## Consequences

- UI and future local clients can use the same command/query boundary.
- Invalid envelopes fail before persistence.
- Duplicate event/source identity conflicts are explicit.
- The API is currently local-development/local-desktop scoped.
- Event API calls are guarded by the temporary local API token from ADR-0038.
- Remote exposure and the full capability runtime require future ADR-backed work before network deployment.
