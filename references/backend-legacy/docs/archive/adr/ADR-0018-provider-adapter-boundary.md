# ADR-0018 Provider Adapter Boundary

Status: Proposed

## Context

Email, Telegram, WhatsApp and future sources have different APIs, IDs, pagination and delivery semantics.

## Decision

Use provider adapters that preserve raw source records and emit normalized commands or events through application boundaries.

## Consequences

- Provider quirks stay isolated.
- Raw evidence remains available for replay and debugging.
- Adapter contracts must handle idempotency and rate limits.
- Outbound provider writes require separate capability checks.
