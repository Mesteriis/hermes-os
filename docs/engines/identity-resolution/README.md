# Identity Resolution Engine

Status: code-aligned documentation package created from ADR-0096 and current
backend modules.

The Identity Resolution Engine creates evidence-backed candidates that two
subjects may represent the same entity.

ADR source of truth:

- [ADR-0096 Canonical Evidence, Review Inbox and Context Packs](../../adr/ADR-0096-canonical-evidence-review-and-context-packs.md)

## Current Implementation Evidence

Current backend file:

- `backend/src/engines/identity_resolution/mod.rs`.

The current implementation exports:

- `IdentityResolutionSubject`;
- `IdentityResolutionCandidate`;
- `IdentityResolutionError`.

Candidates require two different subjects, confidence in the inclusive
`0.0..=1.0` range and at least one evidence observation id.

## Boundary Rule

The engine proposes identity resolution candidates. Personas or another owning
domain must own accepted identity state after review.

