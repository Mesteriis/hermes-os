# ADR-0023 Rebuildable Projections

Status: Proposed

## Context

Search indexes, graph views, timelines and summaries will change as schemas and extraction improve.

## Decision

Treat projections and indexes as rebuildable from canonical events, raw records and document artifacts.

## Consequences

- Projection bugs can be repaired.
- Rebuild tooling is required.
- Derived state must record source versions.
- Canonical storage must be complete enough to rebuild.
