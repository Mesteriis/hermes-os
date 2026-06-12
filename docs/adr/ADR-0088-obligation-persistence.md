# ADR-0088 Obligation Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0015 Command Query Separation
- ADR-0020 Task Candidate Lifecycle
- ADR-0070 Tasks First-Class Domain
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0087 Contradiction Observation Persistence

## Context

Hermes is a Personal Memory System. Communications, meetings, calls and
documents often contain commitments, duties and promises. The documentation
distinguishes three concepts:

- an Obligation is a commitment or duty backed by evidence;
- a Task is an actionable unit with status lifecycle;
- a Follow-Up is a prompt to revisit something.

Current implementation represents adjacent behavior through task candidates,
meeting outcomes, person promises and follow-up status. That is not enough for
the target model because it collapses the reason something matters into the
action that may be created from it.

## Decision

Introduce first-class Obligation persistence.

The initial implementation creates durable, source-backed obligation records:

```yaml
Obligation:
  obligation_id:
  obligated_entity_kind:
  obligated_entity_id:
  beneficiary_entity_kind:
  beneficiary_entity_id:
  statement:
  status:
  review_state:
  due_at:
  condition:
  risk_state:
  confidence:
  metadata:
```

Every durable Obligation must have evidence:

```yaml
ObligationEvidence:
  obligation_id:
  source_kind:
  source_id:
  quote:
  confidence:
  metadata:
```

Initial statuses:

```yaml
ObligationStatus:
  open
  fulfilled
  waived
  disputed
  canceled
```

Initial review states:

```yaml
ObligationReviewState:
  suggested
  user_confirmed
  user_rejected
```

Initial risk states:

```yaml
ObligationRiskState:
  none
  watch
  at_risk
  breached
```

Obligations may link to Tasks, but a confirmed Obligation must not
automatically create a Task. Task creation requires a separate user action,
policy or candidate review flow.

## Consequences

Positive:

- Hermes can remember commitments without forcing them into task lifecycle.
- Tasks can cite Obligations as reasons instead of becoming the source of truth
  for commitments.
- Consistency / Contradiction Engine can point at obligation status conflicts.
- Risk and Timeline engines can consume obligations later.

Negative:

- Existing person promises, meeting outcomes and task candidates remain
  compatibility or source surfaces until adapters are added.
- Public routes and desktop UI are still follow-up work.
- Obligation extraction from Communications and Documents is not part of the
  first persistence slice.

## Non-Goals

- Public `/obligations` API routes.
- Desktop review UI.
- Automatic task creation.
- Automatic obligation extraction from every message.
- Removing task candidates, meeting outcomes or person promises.

## Required Follow-Up

- Add review API and desktop review UI.
- Connect Communication, meeting and document extraction to obligation
  candidates.
- Add adapters from person promises and meeting outcomes.
- Link accepted obligations to tasks and events when reviewed.
- Feed obligation conflicts into the Consistency / Contradiction Engine.
