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

There is no dedicated `backend/src/domains/obligations` module in the current
repository. Related behavior appears through:

- `backend/src/domains/tasks/candidates.rs`;
- `backend/src/domains/tasks/rules.rs`;
- `backend/src/domains/tasks/intelligence.rs`;
- communication extraction and workflow state;
- task candidate migrations.

## Migration Plan

1. Keep Obligations distinct from Tasks in all documentation.
2. Add an engine spec for obligation extraction and candidate review.
3. Add an ADR before introducing a dedicated Obligations persistence model.
4. Link accepted obligations to tasks rather than converting every obligation
   into a task.
5. Use the Consistency / Contradiction Engine when new evidence conflicts with
   obligation status or remembered commitments.
