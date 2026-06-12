# ADR-0019 Contact Identity Resolution

Status: Superseded by ADR-0084

Superseded because ADR-0084 replaces the Contact framing with Persona
Intelligence. The safety rule remains: ambiguous identity resolution must be
reviewable and must not silently collapse subjects.

## Context

People appear through emails, phone numbers, usernames, aliases and organizations. Incorrect automatic merges can damage memory integrity.

## Decision

Model identity resolution as confidence-scored candidates with explicit merge and split workflows.

## Consequences

- Ambiguity remains visible.
- User correction can improve future linking.
- Contact profiles require provenance for channels and aliases.
- Fully automatic identity collapse is disallowed for ambiguous cases.
