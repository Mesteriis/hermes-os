# ADR-0016 Secrets and Encryption Boundary

Status: Proposed

## Context

Provider tokens, app passwords, private keys and backup credentials are high-value secrets.

## Decision

Keep secrets outside ordinary application tables and access them through an OS-backed secret store or encrypted vault abstraction.

## Consequences

- Database compromise does not automatically expose provider credentials.
- Backups need explicit treatment for encrypted secret export.
- Cross-platform behavior must be validated.
- Tests need secret-store substitutes.
