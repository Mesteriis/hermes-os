# Signal Hub Gap Analysis

Status: target-vs-current gap analysis for the uploaded repository snapshot.

## Summary

The repository already has a strong event foundation, but Signal Hub as a
first-class source control domain does not yet exist.

The largest gap is not transport. The largest gap is ownership: Hermes needs a
domain that owns source registry, source runtime policy, health, profiles,
fixtures, mute/pause/replay and source recovery without making Telegram, Mail or
WhatsApp separate product domains.

## Current Strengths

- Event envelope and append-only event log already exist.
- Event consumers already have retry/DLQ direction.
- Communications is already documented as the owner of communication state.
- Telegram and Mail docs already demote providers to integrations/channels.
- Architecture boundary ADRs already prohibit direct domain-to-domain mutation.
- PostgreSQL and Axum are already part of the backend stack.

## Missing Pieces

| Gap | Impact |
|---|---|
| No `domains/signal_hub` module | source control logic will scatter into integrations/settings/app handlers |
| No source registry tables | UI cannot consistently show what is connected, muted, paused or healthy |
| No schema-agnostic recovery fixture | accidental deletion or migration drift can break system source definitions |
| No fixture source contract | tests will depend on real providers or ad-hoc mocks |
| No source mute/pause/replay policy | testing and maintenance will require code-level hacks |
| No NATS JetStream transport | production live delivery remains tied to local broadcast / polling patterns |
| No ConnectRPC contracts | DTO sprawl can grow before API migration |
| No Signal Hub projections | UI may query raw state and cross domains directly |

## Migration Risks

- Duplicating event systems instead of extending `platform/events`.
- Treating Signal Hub as another integration folder.
- Letting Signal Hub write Communications/Tasks/Documents tables directly.
- Storing provider secrets or raw message bodies in Signal Hub state.
- Encoding fixture row IDs or FK references that break on future migrations.
- Introducing sidecar processes before the modular boundary is stable.
- Adding Redis as a second event system without a clear ownership problem.

## Closure Conditions

Signal Hub documentation can be considered implemented when:

1. `backend/src/domains/signal_hub` exists.
2. Signal Hub tables and projections exist.
3. System recovery fixture exists and is loaded idempotently.
4. Signal Hub can list and control built-in source definitions.
5. Real source publication can be globally and selectively muted.
6. Fixture sources can emit deterministic events through the normal EventBus.
7. Event write path supports PostgreSQL event log and NATS JetStream transport.
8. Signal Hub UI reads projections and uses generated ConnectRPC clients.
9. SSE updates Signal Hub UI state.
10. `make validate` passes without architecture boundary exceptions.
