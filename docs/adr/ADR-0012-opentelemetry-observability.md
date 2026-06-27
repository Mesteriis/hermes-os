# ADR-0012 OpenTelemetry Observability

Status: Proposed

## Context

Ingestion, projections, indexing and agent workflows will fail in ways that need diagnosis without leaking private content.

## Decision

Use OpenTelemetry for traces, metrics and structured observability.

## Consequences

- Long-running local workflows can be inspected.
- Projection and ingestion latency can be measured.
- Telemetry must avoid message bodies, secrets and private document content.
- A local collector should be supported before any remote telemetry.
