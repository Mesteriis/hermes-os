# Documentation Refactoring Completion Audit

Date: 2026-06-12

Scope: documentation-only alignment of Hermes Hub around the Personal Memory
System model. This audit does not authorize or perform code, schema, route or UI
renames.

## Result

The documentation now has one canonical product model:

```text
Hermes is a local-first Personal Memory System.
```

Communication is documented as the primary ingestion spine. Personas,
Organizations, Projects, Documents, Tasks, Events, Decisions, Obligations,
Relationships, Knowledge items and Source records are documented as primary
domain concepts. Memory, Timeline, Trust, Search, Enrichment, Obligation, Risk
and Consistency / Contradiction are documented as shared engines.

## Requirement Coverage

| Requirement | Evidence |
|---|---|
| Create a single foundation model. | `docs/foundation/vision.md`, `world-model.md`, `glossary.md`, `engines.md`, `architecture-principles.md`, `domain-map.md`. |
| Rework Personas away from CRM/contact thinking. | `docs/persons/README.md`, `architecture.md`, `data-model.md`, `refactoring-report.md`, ADR-0084. |
| Define Owner/Self Persona. | `docs/foundation/world-model.md`, `docs/persons/data-model.md`, ADR-0084. |
| Treat Communication as the system spine. | `docs/product/master-spec.md`, `docs/domains/communications.md`, `docs/workflows/communication-to-knowledge.md`, ADR-0085. |
| Add Polygraph / contradiction detection concept. | `docs/engines/consistency-contradiction.md`, `docs/workflows/contradiction-review.md`, ADR-0085. |
| Separate domains from engines. | `docs/foundation/engines.md`, `docs/engines/README.md`, `docs/domains/README.md`. |
| Audit all domains. | Domain catalog plus `communications.md`, `organizations.md`, `projects.md`, `calendar-events.md`, `decisions.md`, `obligations.md`, `agents.md`, `notes.md`, existing `persons.md`, `documents.md`, `tasks.md`, `knowledge-graph.md`. |
| Prepare implementation-aware refactoring plans. | `docs/refactoring/product-alignment-plan.md`, `docs/refactoring/implementation-alignment-plan.md`, `docs/product/development-roadmap.md`. |
| Align active implementation docs with current code. | Root `README.md`, `backend/README.md`, `frontend/README.md`, `CONTRIBUTING.md`, `docs/mail/*`, `docs/calendar/*`, `docs/tasks/*`, `docs/organizations/*`, architecture diagrams and security model. |
| Preserve historical traceability. | `docs/refactoring/documentation-audit.md`, `design-qa.md`, historical ADRs, reviews and roadmap files are treated as history unless a current canonical doc references them. |

## Implementation Evidence Reviewed

Current implementation was checked against:

- `backend/src/app/router.rs`;
- backend domain modules under `backend/src/domains/`;
- backend engine/workflow/integration modules;
- migrations `0001` through `0067`;
- `Makefile` development targets;
- frontend API and page surfaces under `frontend/src/lib/`;
- active root, backend and frontend README files.

Key verified points:

- active identity route is `/api/v1/persons/{person_id}/identity`, not
  `/api/v1/contacts/{contact_id}/identity`;
- migration `0034` renamed the historical `contacts` projection to `persons`;
- `backend-contacts-smoke-dev` is a legacy command name that runs the `persons`
  integration suite;
- current protected local API auth uses `HERMES_LOCAL_API_SECRET` and
  `X-Hermes-Secret` per ADR-0056;
- current account setup uses host vault behavior per ADR-0076, while
  `HERMES_SECRET_VAULT_KEY` remains legacy database-vault compatibility;
- ADR-0055 enables email provider read/write capability boundaries while some
  tests and dev sync paths intentionally remain read-only.

## Remaining Refactoring Work

These are known product/implementation gaps, not hidden documentation failures:

- Persona-native naming is not implemented end-to-end. Current code still uses
  `persons`, `person_id`, `person_*` tables and compatibility surfaces.
- First-class Relationship storage now has an initial implementation baseline in
  migration `0060` and `backend/src/domains/relationships/`, plus active
  Persona-to-Persona graph projection through migration `0061`. Guarded backend
  routes can list Relationships by entity and update review state while keeping
  the graph projection aligned. Relationship semantics still need desktop
  review UI and compatibility adapters for roles, organization links and
  project/task links.
- Memory, Timeline, Trust, Risk and Enrichment behavior still appears in
  domain-local modules and routes such as `health`, `watchtower`,
  `intelligence` and `memory`.
- Obligations and Decisions now have source-backed persistence baselines in
  migrations `0063` and `0064`, plus `backend/src/domains/obligations/` and
  `backend/src/domains/decisions/`. Migrations `0065` and `0066` add accepted
  Decision and Obligation graph projection support. Obligation candidate
  detection has a first engine baseline in `backend/src/engines/obligation.rs`,
  Decision candidate detection has a first explicit-evidence baseline in
  `backend/src/engines/decision.rs`, and message task candidate refresh uses
  the Obligation engine for explicit commitments/requests. Migration `0067`
  classifies obligation-derived task candidates; confirming an
  `obligation_task` candidate now creates or updates a source-backed
  `user_confirmed` Obligation and links it to the created Task as a
  `fulfillment_task`. Explicit message/imported-document Decision candidates
  can now refresh into source-backed `suggested` Decisions while preserving
  confirmed/rejected review state across repeat refreshes. Guarded backend
  routes can list accepted Obligations/Decisions and update accepted review
  state without creating Tasks, Projects or Obligations. Provider-wide
  ingestion, desktop UI and adapters from person promises, meeting outcomes and
  project review decisions remain incomplete.
- Consistency / Contradiction Engine now has a structured-claim detection and
  `ContradictionObservation` persistence baseline in migration `0062` and
  `backend/src/engines/consistency.rs`. It can also extract simple structured
  claim lines and limited natural-language `location` / `status` patterns from
  Communication and Document evidence text before comparing them to accepted
  claims. Projected email/Telegram/WhatsApp message refresh, imported Document
  refresh, meeting-note refresh and call-transcript refresh can now compare
  active `person_facts` Memory claims with evidence claims matched by Persona
  email sender, active Telegram/WhatsApp identity, Document email reference,
  event participant link or active Telegram call identity.
  Guarded backend routes can list open contradiction observations and update
  review state without overwriting Memory. The Knowledge workspace includes a
  Polygraph review panel for owner confirm/reject actions. Broad
  natural-language claim extraction remains incomplete.
- Communication is product-facing, but much current implementation still lives
  under `backend/src/domains/mail/` as the email-channel implementation.
- Notes remain document-like capture artifacts unless a future ADR promotes a
  first-class Notes domain.
- `AGENTS.md` has been aligned after this documentation audit to the Personal
  Memory System model, ADR-0056 local API auth, ADR-0076 host vault and
  ADR-0084/ADR-0085 Persona/Communication rules.

## Non-Goals Confirmed

- No code changes.
- No schema migration.
- No API redesign.
- No route rename.
- No generated implementation scaffolding.
- No rewrite of historical ADRs without explicit supersession.
