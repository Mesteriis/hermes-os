# Implementation Alignment Plan

This document maps the current repository implementation to the canonical
Hermes product model.

It is a planning document only. It does not authorize code changes, route
renames, schema migrations or API redesign without a follow-up implementation
task and ADR where required.

## Target Model

Canonical references:

- [Product Master Spec](../product/master-spec.md)
- [Domain Catalog](../domains/README.md)
- [Engine Catalog](../engines/README.md)
- [Workflow Catalog](../workflows/README.md)
- [ADR-0084 Persona Intelligence System](../adr/ADR-0084-persona-intelligence-system.md)
- [ADR-0085 Communication Spine and Consistency / Contradiction Engine](../adr/ADR-0085-communication-spine-and-contradiction-engine.md)
- [ADR-0086 First-Class Relationship Persistence](../adr/ADR-0086-first-class-relationship-persistence.md)
- [ADR-0087 Contradiction Observation Persistence](../adr/ADR-0087-contradiction-observation-persistence.md)
- [ADR-0088 Obligation Persistence](../adr/ADR-0088-obligation-persistence.md)

Hermes is a local-first Personal Memory System. Communications are the primary
ingestion spine. Domains own source-of-truth entities. Engines produce derived
views, candidates, scores and review items.

## Current Implementation Evidence

The current backend has these relevant surfaces:

- route registration in `backend/src/app/router.rs`;
- domain modules under `backend/src/domains/`;
- search and automation modules under `backend/src/engines/`;
- workflow modules under `backend/src/workflows/`;
- provider integrations under `backend/src/integrations/`;
- migrations `0001` through `0067`;
- frontend pages under `frontend/src/lib/pages/`.

## Documentation Drift Corrected

This alignment pass corrected documentation that conflicted with current
implementation evidence:

- `docs/mail/modules.md` now maps to actual files under
  `backend/src/domains/mail/`, `backend/src/workflows/` and
  `backend/src/integrations/`.
- `docs/calendar/architecture.md` now maps to actual files under
  `backend/src/domains/calendar/`.
- `docs/tasks/architecture.md` now maps to actual files under
  `backend/src/domains/tasks/`.
- `docs/organizations/architecture.md` now maps to actual files under
  `backend/src/domains/organizations/`.
- `docs/tasks/api.md` now uses the current router base `/api/v1`, not the stale
  `/api/v2` value.
- Channel/status docs now clarify that implementation coverage percentages are
  local surface coverage, not product-wide completion of Memory, Knowledge,
  Obligations, Decisions or Polygraph.
- `docs/architecture/security-model.md` now follows ADR-0056 and current code:
  `HERMES_LOCAL_API_SECRET` plus `X-Hermes-Secret` are current; token/actor-id
  headers are historical terms from superseded ADRs.
- `docs/architecture/context-diagram.md` and
  `docs/architecture/container-diagram.md` now show Hermes as the Personal
  Memory System with Communications, Events, Documents, shared Engines and the
  Owner Persona.
- `docs/reviews/backend-architecture-review-2026-06-06.md` is explicitly marked
  as a historical review rather than the current implementation map.
- Root, backend and frontend README files now distinguish the current host vault
  from legacy database-vault compatibility, describe email networking under
  ADR-0055 read/write capability boundaries, and use Persona-compatible identity
  wording instead of target-level Contact terminology.
- `backend/migrations/0059_persona_owner_type_constraints.sql` and
  `backend/src/domains/persons/api.rs` now provide the first compatibility-layer
  implementation of `PersonaType` and single Owner Persona semantics on the
  existing `persons` table.
- `backend/migrations/0060_create_relationships.sql` and
  `backend/src/domains/relationships/mod.rs` now provide the first durable
  Relationship persistence baseline with evidence, trust score, strength score,
  confidence and review state.
- `backend/migrations/0061_relationship_graph_projection.sql` now connects
  active Persona-to-Persona Relationship records to graph traversal through
  generic `entity_relationship` graph edges.
- `backend/src/domains/relationships/api.rs` now exposes guarded backend routes
  for listing Relationships by entity and changing review state while keeping
  active Persona-to-Persona graph projections aligned.
- `backend/migrations/0062_create_contradiction_observations.sql` and
  `backend/src/engines/consistency.rs` now provide the first
  Consistency / Contradiction Engine baseline: structured direct-contradiction
  detection and reviewable `ContradictionObservation` persistence.
- `ContradictionObservationStore::refresh_deterministic_observations` now adds
  the first Communication/Document/Event-to-Polygraph refresh paths by
  comparing active `person_facts` Memory claims with structured claims
  extracted from projected Communication message subject/body evidence matched
  by Persona email sender, imported Document title/extracted-text evidence that
  references the Persona email, meeting-note content linked through event
  participants and successful call transcript text linked through active
  Telegram identity.
- `backend/src/engines/consistency_api.rs` now exposes guarded backend routes
  for listing open contradiction observations and changing review state without
  automatically overwriting Memory.
- `backend/migrations/0063_create_obligations.sql` and
  `backend/src/domains/obligations/mod.rs` now provide the first source-backed
  Obligation persistence baseline with evidence, status, review state, risk
  state, confidence and task links.
- `backend/migrations/0064_create_decisions.sql` and
  `backend/src/domains/decisions/mod.rs` now provide the first source-backed
  Decision persistence baseline with evidence, rationale, alternatives, review
  state, confidence and impacted entity links.
- `backend/src/domains/decisions/api.rs` now exposes guarded backend routes for
  listing accepted Decisions by entity and changing accepted Decision review
  state without changing Projects, Tasks or Obligations.
- `backend/src/engines/obligation.rs` now provides the first Obligation Engine
  candidate detection baseline for explicit commitment and request language.
- `backend/src/domains/tasks/candidates.rs` now uses the Obligation Engine when
  refreshing message task candidates for explicit commitment/request language
  that the legacy task scanner does not match.
- `backend/migrations/0067_task_candidate_kind_metadata.sql` and
  `backend/src/domains/tasks/candidates.rs` now classify obligation-derived
  task candidates and materialize confirmed `obligation_task` candidates into
  source-backed `user_confirmed` Obligations linked to the created Task through
  `fulfillment_task`.
- `backend/src/domains/obligations/api.rs` now exposes guarded backend routes
  for listing accepted Obligations by entity and changing accepted Obligation
  review state without creating Tasks.
- `backend/src/domains/decisions/mod.rs` now refreshes explicit Communication
  message and imported Document Decision candidates into source-backed
  `suggested` Decisions and preserves reviewed Decision state across repeat
  refreshes.

## Alignment Matrix

| Target area | Current evidence | Gap | Required plan |
|---|---|---|---|
| Communications domain | `/api/v1/communications/*`, `backend/src/domains/mail/*`, Gmail/Telegram/WhatsApp integrations, communication migrations | Public API is communication-shaped, implementation module is still email/mail-shaped. | Communications migration plan before any module rename. |
| Email channel | `docs/mail/*`, email account routes, mail blob migrations | Email is a channel but still has broad module ownership. | Keep channel docs; do not promote Mail to product domain. |
| Persona Intelligence | `backend/src/domains/persons/*`, `/api/v1/persons/*`, ADR-0084, person/contact migrations, migration `0059` for `is_self` and `person_type` constraints | Target entity is Persona, current compatibility name is Person/Person ID. Owner Persona and PersonaType have a compatibility-layer baseline, but route/schema naming and Dossier/Relationship semantics remain incomplete. | Schema/API naming migration ADR before code rename. |
| Relationships | `backend/src/domains/relationships/mod.rs`, `backend/src/domains/relationships/api.rs`, migrations `0060`, `0061` and `0068`, graph core, person roles, organization contacts, task relations, project link reviews, Personas workspace review panel | First-class Relationship persistence, graph projection for all current `RelationshipEntityKind` endpoints, guarded entity/global review routes and a Personas workspace global suggested review panel have a baseline, but compatibility adapters and broader cross-domain workflow placement are incomplete. | Migrate roles/links/read models behind compatibility boundaries and move or duplicate review into the cross-domain workflow shell when defined. |
| Memory Engine | persons memory, organization memory, project memory docs | Memory behavior is domain-local. | Shared Memory Engine implementation plan after domain source boundaries are stable. |
| Timeline Engine | calendar events, person timeline, organization timeline, project timelines, frontend timeline page | Timeline views exist in multiple places. | Timeline Engine extraction plan; Calendar remains scheduled event domain. |
| Trust Engine | `persons/trust.rs`, risk/health modules, relationship scores in docs | Trust is partly Persona-local and partly risk/health language. | Normalize trust as source/relationship signal, not generic entity field. |
| Risk Engine | `health.rs`, `watchtower`, risks routes in persons/orgs/calendar/tasks | Health/watchtower naming hides shared Risk Engine semantics. | Risk terminology migration plan for docs/UI/API compatibility. |
| Enrichment Engine | persons enrichment, organization enrichment | Enrichment exists per domain. | Shared engine semantics with domain-specific source policies. |
| Obligation Engine | `backend/src/engines/obligation.rs`, `backend/src/domains/obligations/mod.rs`, `backend/src/domains/obligations/api.rs`, migrations `0063`, `0066` and `0067`, task candidates, task rules, communication extraction, Tasks workspace review panel | Candidate detection, accepted Obligation persistence, accepted Obligation graph projection, guarded accepted-Obligation backend entity/global review routes, global Tasks workspace review and confirmed `obligation_task` candidate materialization have baselines. Provider-wide Communication ingestion, meeting/document adapters, candidate-to-Obligation review routing and compatibility adapters are incomplete. | Extend Communication/meeting/document ingestion to the engine and feed reviewed candidates to the Obligations domain without auto-creating Tasks outside explicit task-candidate review. |
| Consistency / Contradiction Engine | `backend/src/engines/consistency.rs`, `backend/src/engines/consistency_api.rs`, migration `0062`, ADR-0085, ADR-0087 | Structured direct-contradiction detection, deterministic structured and limited natural-language `location` / `status` claim extraction from Communication/Document/Event evidence text, observation persistence, guarded backend review routes, Knowledge workspace review panel and projected email/Telegram/WhatsApp message, imported Document, meeting-note and call-transcript refresh against active `person_facts` have baselines. Broad natural-language extraction and broader provider evidence are incomplete. | Expand ingestion refresh to broader provider evidence, then add reviewed-outcome semantics without automatic overwrite. |
| Decisions domain | `backend/src/domains/decisions/mod.rs`, `backend/src/domains/decisions/api.rs`, `backend/src/engines/decision.rs`, migrations `0064` and `0065`, meeting outcomes, project link review decisions, communication/document evidence, Tasks workspace review panel | Accepted Decision persistence, deterministic explicit-Decision candidate extraction, explicit message/imported-document candidate persistence as `suggested` Decisions, accepted Decision graph projection, guarded accepted-Decision backend entity/global review routes and global Tasks workspace review have baselines. Provider-wide ingestion, meeting adapters, candidate-to-Decision review routing and adapters from meeting outcomes/project reviews are incomplete. | Connect remaining meeting/communication/document candidates to the Decisions domain without auto-changing Projects, Tasks or Obligations. |
| Agents domain | AI runtime/control center, Ollama/OmniRoute, frontend Agents page | Runtime exists; graph identity and Owner Persona attribution are incomplete. | Agent Persona and capability audit plan. |
| Notes boundary | frontend Notes page, documents treat notes as artifacts | No backend Notes domain and no ADR promotes one. | Keep Notes as document-like artifacts until ADR changes boundary. |

## Refactoring Slices

### Slice 1: Communications Naming Boundary

Goal: make the implementation shape explicit without breaking working code.

Work items:

- keep `/api/v1/communications/*` as the product-facing route family;
- document `backend/src/domains/mail/*` as the current email-channel
  implementation;
- identify modules that are email-specific versus communication-generic;
- avoid module renames until tests and migration scope are defined;
- add ADR if route or module naming changes.

Validation for future code work:

- communications API tests;
- email provider networking tests;
- Telegram and WhatsApp tests;
- route snapshot or router smoke test if available.

### Slice 2: Persona Compatibility Boundary

Goal: migrate language toward Persona without corrupting existing `persons`
storage contracts.

Work items:

- keep ADR-0084 as the target model;
- list all `person_id`, `/persons`, `person_*` tables and DTOs before any
  rename;
- rename development compatibility targets such as `backend-contacts-smoke-dev`
  only after the command surface is reviewed;
- separate compatibility names from product language in docs and UI labels;
- keep the migration `0059` Owner Persona uniqueness and PersonaType validation
  baseline intact;
- design first-class Relationship storage before removing role/contact-shaped
  fields.

Validation for future code work:

- person API tests;
- identity review tests;
- migration replay tests;
- graph projection tests.

### Slice 3: Relationship Model Consolidation

Goal: make Relationship a shared source-of-truth concept instead of scattered
fields.

Work items:

- inventory graph edges, person roles, organization contacts, task relations and
  project link reviews;
- define relationship type taxonomy;
- require provenance, confidence, source evidence, validity period and review
  state;
- map `trust_score` and `strength_score` to relationship semantics;
- keep the migration `0060` and `RelationshipStore` source-of-truth baseline
  intact;
- define graph projection and compatibility adapters for existing tables.

Validation for future code work:

- graph core tests;
- projection replay tests;
- relationship query tests;
- identity split/merge regression tests.

### Slice 4: Engine Boundary Extraction

Goal: separate reusable engines from domain-local intelligence modules.

Work items:

- inventory all `health`, `watchtower`, `intelligence`, `enrichment`, `trust`
  and `memory` modules;
- map each output to Memory, Timeline, Trust, Search, Enrichment, Obligation,
  Risk or Consistency / Contradiction Engine;
- keep domain source records in their owning domains;
- convert engine output to reviewable observations or derived projections.

Validation for future code work:

- existing domain tests;
- engine-specific unit tests;
- projection rebuild tests;
- source-citation tests.

### Slice 5: Decisions And Obligations

Goal: add missing target domains only after candidate/review behavior is clear.

Work items:

- define candidate-first flow for decisions and obligations;
- use Communications, Calendar/Events and Documents as initial evidence sources;
- require owner review or explicit narrow policy before accepted state;
- link accepted obligations to tasks instead of converting every obligation into
  a task;
- link accepted decisions to projects, meetings and documents.

Validation for future code work:

- candidate extraction tests;
- review workflow tests;
- event/audit tests;
- graph link tests.

### Slice 6: Polygraph Implementation

Goal: implement contradiction detection as reviewable observations.

Work items:

- start with communications and documents as evidence sources;
- compare new claims to accepted Memory and Knowledge;
- create `ContradictionObservation` records with old and new source references;
- expose review outcomes without automatic memory overwrite;
- feed reviewed outcomes into Memory, Trust, Risk and Relationship semantics.

Validation for future code work:

- contradiction detection unit tests;
- no-auto-overwrite regression tests;
- source citation tests;
- review outcome tests;
- privacy/audit tests.

### Slice 7: UI Vocabulary Migration

Goal: make the desktop UI express the Personal Memory System model.

Work items:

- keep compatibility route names internal;
- prefer Persona, Communication, Context, Memory, Obligation and Decision in UI
  labels;
- treat Timeline as an engine view;
- treat Health/Watchtower as Risk or Attention views;
- treat Notes as capture/document artifacts;
- add Polygraph review surface only after engine implementation exists.

Validation for future code work:

- frontend translation checks;
- page-level rendering tests;
- accessibility checks for review and confirmation states.

## ADR Requirements

Create ADRs before:

- renaming persisted tables or route families;
- introducing first-class Relationships;
- adding Decisions or Obligations persistence;
- implementing Consistency / Contradiction Engine persistence or routes;
- allowing agent write automation beyond current capability policy.

## Non-Goals

- No code changes in this documentation pass.
- No API redesign in this document.
- No schema migration in this document.
- No removal of compatibility routes or historical migrations.
- No rewrite of historical ADRs except explicit supersession.
