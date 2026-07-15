# ADR-0029 Explicit Schema Evolution

Status: Proposed

## Context

Events, relational tables, graph relationships and extracted artifacts will evolve over years.

## Decision

Use explicit schema versions, migrations and compatibility checks for durable data.

## Consequences

- Long-term upgrades become safer.
- Importers and projectors must handle older schemas.
- Migration testing is required.
- Breaking storage decisions need ADRs.
