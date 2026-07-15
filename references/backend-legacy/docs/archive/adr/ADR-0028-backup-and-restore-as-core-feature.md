# ADR-0028 Backup and Restore as Core Feature

Status: Proposed

## Context

Local-first data ownership is only credible if the user can recover from machine loss, corruption or migration.

## Decision

Treat backup and restore as core product architecture, not operational afterthought.

## Consequences

- Storage layout must be backup-aware.
- Restore verification is required.
- Secret export requires explicit secure handling.
- Indexes may be rebuilt, but canonical data and artifacts must be preserved.
