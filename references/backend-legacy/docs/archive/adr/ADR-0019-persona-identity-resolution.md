# ADR-0019 Persona Identity Resolution

Status: Proposed

## Context

Personas appear through emails, phone numbers, usernames, aliases and organizations. Incorrect automatic merges can damage memory integrity.

## Decision

Model identity resolution as confidence-scored candidates with explicit merge and split workflows.

## Consequences

- Ambiguity remains visible.
- User correction can improve future linking.
- Persona profiles require provenance for channels and aliases.
- Fully automatic identity collapse is disallowed for ambiguous cases.
