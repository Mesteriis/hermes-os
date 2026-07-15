# ADR-0013 Local First Data Ownership

Status: Proposed

## Context

The user must own communication history, graph memory and document-derived knowledge.

## Decision

Design local storage and local operation as the default. Cloud services are optional integrations, not required infrastructure.

## Consequences

- The app remains useful offline for already-ingested data.
- Backup and restore become product-critical.
- Multi-device sync is deferred but must not be made impossible.
- Local machine lifecycle and storage capacity matter.
