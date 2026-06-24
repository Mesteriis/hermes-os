# ADR-0012 OpenTelemetry Observability

Status: Proposed

## Context

Ingestion, projections, indexing and agent workflows will fail in ways that need diagnosis without leaking private content.

## Decision

Use OpenTelemetry for traces, metrics and structured observability.

## Clarification By ADR-0100

ADR-0100 clarifies canonical trace storage.

OpenTelemetry is allowed as an export and diagnostic layer. OpenTelemetry must
not be required to reconstruct Hermes causal traces.

The canonical trace store is PostgreSQL append-only `event_log`.

Hermes treats canonical events as spans:

```text
event_id = span id
correlation_id = trace id
causation_id = parent span id
```

## Consequences

- Long-running local workflows can be inspected.
- Projection and ingestion latency can be measured.
- Telemetry must avoid message bodies, secrets and private document content.
- A local collector should be supported before any remote telemetry.
