# Relationship Candidate Engine

Status: code-aligned documentation package created from ADR-0096 and current
backend modules.

The Relationship Candidate Engine proposes source-backed links between
entities. It is separate from the Relationships domain, which owns accepted
Relationship records.

ADR source of truth:

- [ADR-0096 Canonical Evidence, Review Inbox and Context Packs](../../adr/ADR-0096-canonical-evidence-review-and-context-packs.md)

## Current Implementation Evidence

Current backend file:

- `backend/src/engines/relationships/mod.rs`.

The current implementation exports:

- `RelationshipSubject`;
- `RelationshipCandidate`;
- `RelationshipEngineError`.

Candidates require source and target subjects, a non-empty relationship type,
confidence in the inclusive `0.0..=1.0` range and at least one evidence
observation id.

## Boundary Rule

The engine creates relationship candidates. Accepted relationship truth belongs
to [Relationships Domain](../../domains/relationships/README.md).

