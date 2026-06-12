# Hermes Engine Catalog

Engines are reusable system mechanisms. They are used by domains but do not own
domain source-of-truth entities.

Canonical foundation map: [Foundation Engines](../foundation/engines.md).

## Engine Boundary

An engine may:

- read domain entities and event history;
- build derived views;
- emit candidates, observations, scores or review items;
- maintain rebuildable projections or indexes.

An engine must not:

- become the owner of Persona, Organization, Project, Task, Document,
  Communication, Decision or Obligation truth;
- silently overwrite accepted memory;
- hide uncertainty;
- emit conclusions without provenance.

## Engine Specs

| Engine | Spec |
|---|---|
| Memory Engine | [Memory Engine](memory.md) |
| Timeline Engine | [Timeline Engine](timeline.md) |
| Trust Engine | [Trust Engine](trust.md) |
| Search Engine | [Search Engine](search.md) |
| Enrichment Engine | [Enrichment Engine](enrichment.md) |
| Obligation Engine | [Obligation Engine](obligation.md) |
| Risk Engine | [Risk Engine](risk.md) |
| Consistency / Contradiction Engine | [Consistency / Contradiction Engine](consistency-contradiction.md) |

## Current Implementation Evidence

The current repository contains dedicated baseline modules for:

- `backend/src/engines/search.rs`;
- `backend/src/engines/consistency.rs`;
- `backend/src/engines/obligation.rs`.

Many other engine-like behaviors are still domain-local:

- `backend/src/domains/persons/memory.rs`;
- `backend/src/domains/persons/trust.rs`;
- `backend/src/domains/persons/enrichment_engine.rs`;
- `backend/src/domains/organizations/memory.rs`;
- `backend/src/domains/*/health.rs`;
- `backend/src/domains/*/intelligence.rs`;
- `backend/src/workflows/email_intelligence.rs`.

That implementation shape is a migration fact, not a domain rule. Future
refactors should move shared reusable behavior toward engine boundaries only
after an implementation plan and ADR where persistence or public contracts
change.
