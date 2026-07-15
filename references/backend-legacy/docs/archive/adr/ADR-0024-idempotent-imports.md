# ADR-0024 Idempotent Imports

Status: Proposed

## Context

Provider sync jobs will be interrupted, retried and re-run. Duplicate messages or documents would corrupt timelines and graph links.

## Decision

All imports must be idempotent using provider IDs, content fingerprints, import batch IDs and source-specific checkpoints.

## Consequences

- Retry safety improves.
- Provider-specific identity logic is required.
- Some sources without stable IDs need fingerprint strategy.
- Import audit data must be retained.
