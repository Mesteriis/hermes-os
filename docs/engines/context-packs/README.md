# Context Packs Engine

Status: code-aligned documentation package created from ADR-0096 and current
backend modules.

Context Packs are rebuildable context bundles derived from observations,
domains, knowledge, relationships and prior decisions. They are engine output,
not source-of-truth domain records.

ADR source of truth:

- [ADR-0096 Canonical Evidence, Review Inbox and Context Packs](../../adr/ADR-0096-canonical-evidence-review-and-context-packs.md)

## Current Implementation Evidence

Current backend files:

- `backend/src/engines/context_packs/mod.rs`;
- `backend/src/engines/context_packs/models.rs`;
- `backend/src/engines/context_packs/review.rs`;
- `backend/src/engines/context_packs/store.rs`.

Current model kinds include:

- persona;
- meeting;
- task;
- calendar;
- project;
- review.

Current source kinds include observations, domain entities, knowledge,
relationships, decisions, tasks, obligations, documents, calendar events and
projects. Review Context Packs also use `review_item` source links for their
subject Review item.

## Boundary Rule

Context Packs may persist rebuildable content and source links in
`context_packs` and `context_pack_sources`. They must not become the canonical
owner of memory, domain truth, provider records or accepted review outcomes.
