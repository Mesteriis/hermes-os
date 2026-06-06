# ADR-0016 Secrets and Encryption Boundary

Status: Superseded by ADR-0053

This decision was superseded by ADR-0053, which keeps the secret boundary but moves encrypted provider credential payloads into a dedicated PostgreSQL vault table while keeping the vault key outside PostgreSQL.

## Context

Provider tokens, app passwords, private keys and backup credentials are high-value secrets.

## Decision

Keep secrets outside ordinary application tables and access them through an OS-backed secret store or encrypted vault abstraction.

Current implementation stores PostgreSQL `secret_references` metadata only. Secret values are not stored in PostgreSQL.

## Consequences

- Database compromise does not automatically expose provider credentials.
- Backups need explicit treatment for encrypted secret export.
- Cross-platform behavior must be validated.
- Tests need secret-store substitutes.
