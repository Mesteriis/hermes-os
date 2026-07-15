# Obligations Domain

Status: documentation package aligned to the current repository structure.

Obligations are commitments, duties or promised responsibilities backed by
evidence.

A Task is an actionable unit. An Obligation is the reason something may need to
be done.

## Responsibilities

The Obligations domain owns:

- obligation records;
- obligated party;
- beneficiary or counterparty;
- source evidence;
- due date or condition when known;
- fulfillment state;
- related tasks and reminders;
- risk and contradiction observations.

The Obligations domain does not own:

- every task;
- every follow-up;
- task status lifecycle;
- communication source records;
- calendar event identity.

## Obligation Sources

Obligations can be extracted from:

- communications;
- meetings;
- calls;
- contracts and documents;
- calendar events;
- manual owner input;
- agent suggestions with review.

The Obligation Engine creates candidates. The domain stores reviewed obligations
or policy-approved low-risk captures.

## Obligation Model

```yaml
Obligation:
  id:
  obligated_entity:
  beneficiary_entity:
  statement:
  status:
  due_at:
  condition:
  evidence:
  linked_tasks:
  linked_events:
  risk_state:
  review_state:
```

## Current Implementation Evidence

Current backend baseline:

- `backend/migrations/0063_create_obligations.sql`;
- `backend/migrations/0066_obligation_graph_projection.sql`;
- `backend/migrations/0067_task_candidate_kind_metadata.sql`;
- `backend/src/domains/obligations/mod.rs`;
- `backend/src/domains/obligations/api.rs`;
- `backend/src/domains/personas/trust.rs`;
- `backend/src/domains/tasks/candidates.rs`;
- `backend/tests/obligations.rs`;
- `backend/tests/obligations_api.rs`;
- `backend/tests/task_candidates.rs`;
- `backend/tests/calendar.rs`;
- ADR-0088.

This baseline provides source-backed Obligation persistence with evidence,
status, review state, risk state, confidence, due date or condition and optional
Task links. It also projects accepted Obligations into the graph for supported
obligated and beneficiary entity kinds, using `obligation` graph nodes and
source-backed `entity_relationship` edges. It explicitly does not auto-create
Tasks.

Task candidate review has a backend baseline for obligation-derived candidates:
message and document candidates produced by the Obligation Engine are stored as
`candidate_kind = obligation_task`, preserve the source `ObligationCandidate` in
metadata and, when user-confirmed, create or update a `user_confirmed`
Obligation with source evidence and a `fulfillment_task` link to the created
Task. Resetting or rejecting that candidate synchronizes the durable Obligation
review state and removes the concrete Task link. Generic task candidates remain
task-only.
Email sync and Telegram/WhatsApp fixture ingestion call the same targeted
message refresh path after Communication projection. This creates suggested
obligation-derived task candidates only; it does not auto-create Tasks or
accepted Obligations.

Meeting outcomes with `outcome_type = promise`, `task` or `follow_up` now adapt
into source-backed `suggested` Obligations without creating Tasks. If the
meeting outcome has an `owner_person_id`, the Obligation is owed by that
Persona; otherwise the meeting Event remains the obligated compatibility anchor.
The meeting outcome keeps the created Obligation id in `linked_entity_id`.

Compatibility `person_promises` created through `PersonaPromiseStore::create`
now adapt into source-backed `user_confirmed` Obligations with `raw_record`
evidence. This preserves the old promise compatibility table while making
Obligation the durable commitment record. It does not create Tasks.

Backend routes currently expose:

- `GET /api/v1/obligations?entity_kind=&entity_id=&limit=`;
- `GET /api/v1/obligations?review_state=&limit=`;
- `PUT /api/v1/obligations/{obligation_id}/review`.

These routes are guarded by the local API secret and support accepted
Obligation review state changes. They do not create Tasks or convert task
candidates into accepted Obligations.

The Tasks workspace includes the first desktop review panel for global
suggested Obligations and Decisions, with optional entity-scoped filtering. It
lists Obligations through the guarded Obligation route and submits explicit
owner confirm/reject review state without creating Tasks or converting
candidates into accepted Obligations.

Related behavior still exists through:

- `backend/src/domains/tasks/candidates.rs`;
- `backend/src/domains/tasks/rules.rs`;
- `backend/src/domains/tasks/intelligence.rs`;
- `backend/src/domains/personas/trust.rs`;
- meeting outcomes;
- communication extraction and workflow state;
- task candidate migrations.

## Migration Plan

1. Keep Obligations distinct from Tasks in all documentation.
2. Keep the ADR-0088 persistence boundary intact.
3. Expand Obligation Engine extraction beyond explicit message/document task
   candidates and the current meeting outcome adapter.
4. Expand candidate-to-Obligation review routing beyond the current
   obligation-derived task-candidate path and add reviewed Obligation links to
   documents, events and compatibility sources without converting every
   obligation into a task.
5. Project reviewed Obligations into timeline and dossier views.
6. Use the Consistency / Contradiction Engine when new evidence conflicts with
   obligation status or remembered commitments.
