# Product Alignment Refactoring Plan

Date: 2026-06-12

Scope: documentation-derived product and implementation alignment plan.

This document records where the current implementation differs from the Product
Master Spec target model and what refactoring or delivery plans are needed.

It is not an implementation plan for code changes. Each implementation item
below requires its own ADR review, design or execution plan before code changes.

## Alignment Baseline

Target product model:

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
  -> Timeline / Dossier / Recall
```

Current implementation already includes communication ingestion, mail workflows,
Telegram and WhatsApp foundations, graph projection, documents, projects,
persons compatibility, organizations, calendar, tasks, AI runtime, settings and
vault support.

The gaps below are about target-model alignment, not lack of useful
implementation.

## Product Alignment Gaps

| Gap | Current evidence | Target direction | Plan type |
|---|---|---|---|
| Communications still read as email-heavy in code and docs. | `backend/src/domains/mail`, `docs/mail/`, many `/api/v1/communications/*` routes backed by email modules. | Communications is the domain; email is one channel. | Documentation first, implementation naming later. |
| Persona model is compatibility-based. | `persons`, `person_id`, `person_roles`, `person_personas`, `person_promises`, `/api/v1/persons/*`. | Persona, Owner Persona, PersonaType and first-class Relationships. | ADR and migration plan before code. |
| Relationships are fragmented. | Graph edges, organization contact links, task relations, project link reviews, relationship events and person roles coexist. First-class Relationship persistence, graph projection for all current Relationship entity kinds, guarded entity/global review routes, manual/API and email-sync organization contact link adapters, manual task relation adapters, project link review adapters and Personas workspace review have a baseline. | Relationship is first-class with type, confidence, provenance, trust and validity. | Continue remaining person-role adapter work and cross-domain review workflow placement. |
| Polygraph engine is partially implemented. | Migration `0062`, `backend/src/engines/consistency.rs` and `backend/src/engines/consistency_api.rs` provide structured direct-contradiction detection, deterministic structured and limited natural-language `location` / `status` claim extraction from Communication/Document/Event evidence text, reviewable observations, guarded backend review routes, Knowledge workspace review UI and projected email/Telegram/WhatsApp message/imported Document/meeting-note/call-transcript refresh against active `person_facts`. Broad natural-language extraction and broader provider evidence are incomplete. | Cross-domain engine for contradiction observations and review items. | Expand ingestion refresh to broader provider evidence, then add reviewed-outcome semantics. |
| Decisions and Obligations are partial. | Migrations `0063`, `0064`, `0065`, `0066` and `0067` plus `backend/src/domains/obligations/` and `backend/src/domains/decisions/` provide source-backed persistence, accepted graph projection and task-candidate classification for obligation-derived candidates. `backend/src/engines/obligation.rs` provides a first obligation candidate detector, `backend/src/engines/decision.rs` provides a first explicit-decision candidate detector, message task candidate refresh uses Obligation detection for explicit commitments/requests, confirmed `obligation_task` candidates materialize source-backed Obligations linked to Tasks, reset/reject review on those candidates synchronizes durable Obligation state, email sync refreshes reviewable Decision and obligation-derived task candidates for projected email Communications, compatibility `person_promises` materialize source-backed `user_confirmed` Obligations, explicit message/imported-document Decision candidates persist as source-backed `suggested` Decisions, project link reviews materialize source-backed `user_confirmed` Decisions, meeting outcomes create reviewable Decisions or Obligations for `decision`, `promise`, `task` and `follow_up` outcomes, accepted Obligations/Decisions have guarded backend entity/global list/review routes, and the Tasks workspace has a global suggested review panel. Candidate-to-domain routing still needs broader workflow coverage. | Durable Decisions and Obligations with evidence and review. | Expand ingestion wiring, review workflows and compatibility adapters. |
| Engine ownership is partly embedded in domains. | Health/watchtower, intelligence, enrichment and timeline-like modules appear in domain folders. | Engines are reusable mechanisms; domains own durable truth. | Engine spec wave before refactoring. |
| Notes are ambiguous. | Frontend has Notes page; foundation treats Notes as document-like artifacts. | Notes remain lightweight document artifacts unless a future ADR promotes them. | Documentation clarification; no implementation change yet. |
| UI vocabulary exposes compatibility names. | Frontend pages include Persons, Notes, Timeline and domain-specific health/watchtower concepts. | UI should surface Personal Memory System concepts without hiding compatibility state. | UI vocabulary plan after product docs. |

## Refactoring And Delivery Plans To Create

### 1. Communications Normalization Plan

Goal: align Mail, Telegram, WhatsApp, calls and meetings under the
Communications product model.

Required scope:

- document channel-specific source boundaries;
- preserve provider-specific implementation modules;
- define canonical Communication lifecycle;
- identify which current mail-specific routes are compatibility names;
- avoid code renames until API/schema compatibility is planned.

### 2. Persona Migration Plan

Goal: move from `persons` compatibility toward the Persona target model.

Required scope:

- Owner Persona semantics;
- `PersonaType` values: `human`, `ai_agent`, `organization_proxy`, `system`;
- target identity trace model;
- `/persons` compatibility strategy;
- treatment of `person_roles`, `person_personas`, `person_promises`,
  `health_status`, `watchlist` and `is_favorite`;
- migration safety and graph impact.

### 3. Relationship Model Plan

Goal: define first-class Relationship records across Personas, Organizations,
Projects, Documents, Communications, Tasks, Events, Decisions and Obligations.

Required scope:

- relationship type taxonomy;
- source and target entity references;
- confidence and provenance;
- trust and strength scores;
- validity period;
- review state for inferred relationships;
- integration with graph projection.

### 4. Polygraph Engine Plan

Goal: introduce Consistency / Contradiction Engine behavior.

Required scope:

- contradiction taxonomy;
- accepted memory inputs;
- new evidence inputs;
- `ContradictionObservation` target shape;
- review workflow;
- effect on Risk and Trust signals;
- source citation requirements;
- UI surface for contradiction review.

### 5. Decisions And Obligations Plan

Goal: separate durable Decisions and Obligations from Tasks, Follow-Ups,
Promises and meeting outcomes.

Required scope:

- Decision evidence and rationale model;
- Obligation evidence and lifecycle;
- Task creation from Obligations;
- Follow-Up as prompt, not always task;
- meeting outcome mapping;
- communication-to-obligation workflow.

### 6. Engine Boundary Plan

Goal: keep domain truth separate from reusable intelligence mechanisms.

Required scope:

- Memory Engine;
- Timeline Engine;
- Trust Engine;
- Search Engine;
- Enrichment Engine;
- Obligation Engine;
- Risk Engine;
- Consistency / Contradiction Engine;
- which current modules are domain-owned and which are engine-like.

### 7. UI Vocabulary Plan

Goal: align desktop surfaces with the Personal Memory System model.

Required scope:

- Personas vs Persons labeling;
- Notes as capture/document artifacts;
- Timeline as engine view;
- Health/watchtower as attention/risk views;
- Communications as the shared entry point;
- product navigation around Context, Memory and Action.

## Documentation Execution Order

1. Complete Product Spine.
2. Create Communications, Personas, Relationships and Knowledge domain specs.
3. Create Obligations, Tasks and Decisions specs.
4. Create Projects, Documents, Organizations and Events specs.
5. Create engine specs, including Polygraph.
6. Create workflow specs.
7. Only then write implementation migration plans for code/schema/API changes.

Wave 2 adds the active domain catalog under `docs/domains/` and creates missing
canonical domain documents for Communications, Organizations, Projects,
Calendar/Events, Decisions, Obligations, Agents and Notes. These documents are
documentation alignment only; they do not authorize code, route or schema
changes without a follow-up implementation plan and ADR where needed.

Wave 3 adds the active engine catalog under `docs/engines/` and creates detailed
specs for Memory, Timeline, Trust, Search, Enrichment, Obligation, Risk and
Consistency / Contradiction. The current code still has several domain-local
engine-like modules; this is a migration gap, not a target boundary.

Wave 4 adds the workflow catalog under `docs/workflows/` for
communication-to-knowledge, communication-to-obligation, meeting-to-decisions,
document-to-context, contradiction-review, dossier-generation and
agent-assisted-recall. These workflows coordinate domains and engines; they do
not define new APIs or authorize implementation changes by themselves.

Wave 5 adds `docs/refactoring/implementation-alignment-plan.md`, which maps the
current backend routes, domain modules, migrations and frontend surfaces to the
target model and splits future code work into safe refactoring slices.

## Current Non-Goals

- No code changes.
- No route renames.
- No schema migrations.
- No generated API design.
- No rewriting historical ADRs.

## Validation Expectation

Every future refactoring plan must include:

- implementation evidence inspected;
- target model reference;
- affected docs;
- affected modules, migrations and frontend surfaces if code work is proposed;
- migration and rollback strategy if persisted data changes;
- validation commands scoped to the actual change.
