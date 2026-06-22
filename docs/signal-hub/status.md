# Signal Hub Status

Status date: 2026-06-22.

## Overall Status

`PLANNED TARGET SPECIFICATION`

Signal Hub is not yet implemented as a first-class backend domain in the uploaded
repository snapshot. This document records the intended architecture and the
current foundations that already exist.

## Existing Foundations

| Foundation | Current status |
|---|---|
| Append-only event log | implemented through `backend/migrations/0001_create_event_log.sql` |
| Canonical EventEnvelope | implemented in `backend/src/platform/events/models.rs` |
| In-process EventBus | implemented as broadcast bus in `backend/src/platform/events/bus.rs` |
| Durable event consumers | implemented in `backend/src/platform/events/consumers.rs` |
| DLQ concepts | implemented in event consumer store/ADR-0095 |
| Communications domain | implemented as provider-neutral domain target |
| Telegram channel integration docs | implemented |
| Mail channel docs | implemented |
| WhatsApp foundation migration | present as V5 foundation migration |

## Not Yet Implemented

| Area | Status |
|---|---|
| `domains/signal_hub` backend module | missing |
| Signal Hub database tables | missing |
| Signal Hub ConnectRPC service | missing |
| NATS JetStream transport | missing from current dependencies/code |
| Protobuf contracts | missing from current repository layout |
| Signal Hub UI | missing |
| Fixture recovery loader | missing |
| signal profiles | missing |
| source mute/pause/replay UI | missing |
| provider-neutral source control API | missing |

## Target Dependencies

Backend target additions:

```toml
async-nats = "..."
uuid = { version = "...", features = ["v7", "serde"] }
prost = "..."
connectrpc = "..."
connectrpc-axum = "..."
```

Version pins should be chosen during implementation from the current Rust
compatibility matrix. Do not add Redis for Signal Hub events.

## Current Decision Summary

- Build as modular monolith first.
- No Redis sidecar.
- No Telegram/Mail/WhatsApp sidecar processes for the initial implementation.
- NATS JetStream is the production event transport target from the start.
- PostgreSQL `event_log` remains the audit/recovery source of truth.
- ConnectRPC + Protobuf are the canonical API contract target from the start.
- Axum SSE is the browser realtime update path.
- Fixture-first testing is required.
- Recovery fixture must be schema-agnostic and reference-free.
