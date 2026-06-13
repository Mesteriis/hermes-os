# Obligations Domain

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
- `backend/src/domains/tasks/candidates.rs`;
- `backend/tests/obligations.rs`;
- `backend/tests/obligations_api.rs`;
- `backend/tests/task_candidates.rs`;
- ADR-0088.

This baseline provides source-backed Obligation persistence with evidence,
status, review state, risk state, confidence, due date or condition and optional
Task links. It also projects accepted Obligations into the graph for supported
obligated and beneficiary entity kinds, using `obligation` graph nodes and
source-backed `entity_relationship` edges. It explicitly does not auto-create
Tasks.

Task candidate review has a backend baseline for obligation-derived candidates:
message candidates produced by the Obligation Engine are stored as
`candidate_kind = obligation_task`, preserve the source `ObligationCandidate` in
metadata and, when user-confirmed, create or update a `user_confirmed`
Obligation with Communication evidence and a `fulfillment_task` link to the
created Task. Generic task candidates remain task-only.

Backend routes currently expose:

- `GET /api/v1/obligations?entity_kind=&entity_id=&limit=`;
- `PUT /api/v1/obligations/{obligation_id}/review`.

These routes are guarded by the local API secret and support accepted
Obligation review state changes. They do not create Tasks or convert task
candidates into accepted Obligations.

The Tasks workspace includes the first scoped desktop review panel for
entity-scoped Obligations and Decisions. It lists Obligations through the
guarded Obligation route and submits explicit owner confirm/reject review state
without creating Tasks or converting candidates into accepted Obligations.

Related behavior still exists through:

- `backend/src/domains/tasks/candidates.rs`;
- `backend/src/domains/tasks/rules.rs`;
- `backend/src/domains/tasks/intelligence.rs`;
- communication extraction and workflow state;
- task candidate migrations.

## Migration Plan

1. Keep Obligations distinct from Tasks in all documentation.
2. Keep the ADR-0088 persistence boundary intact.
3. Expand Obligation Engine extraction beyond explicit message task candidates.
4. Expand desktop review beyond the scoped entity panel and expand reviewed
   Obligation links to events and compatibility sources without converting
   every obligation into a task.
5. Project reviewed Obligations into timeline and dossier views.
6. Use the Consistency / Contradiction Engine when new evidence conflicts with
   obligation status or remembered commitments.
