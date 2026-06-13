# Obligation Engine

The Obligation Engine detects commitments, duties and expected actions from
evidence.

It can create candidates for the Obligations and Tasks domains. It does not make
every obligation into a task.

## Responsibilities

The Obligation Engine produces:

- obligation candidates;
- follow-up candidates;
- task candidates;
- missing-response signals;
- due-date candidates;
- fulfillment evidence suggestions.

It does not own:

- accepted obligation truth;
- task lifecycle;
- calendar event identity;
- communication source records.

## Inputs

- communications;
- meetings and calls;
- documents;
- decisions;
- calendar events;
- owner rules;
- accepted obligations and tasks.

## Output Requirements

Every obligation candidate must include:

- source evidence;
- obligated party;
- beneficiary or counterparty when known;
- statement;
- due date or condition when detected;
- confidence;
- review state.

## Current Implementation Evidence

Current backend state:

- `backend/src/engines/obligation.rs` provides a deterministic candidate
  detection baseline for explicit commitment and request language;
- the Obligations domain has a persistence baseline in
  `backend/migrations/0063_create_obligations.sql` and
  `backend/src/domains/obligations/mod.rs`;
- `backend/src/domains/obligations/api.rs` exposes guarded backend routes for
  listing entity-scoped Obligations and changing accepted Obligation review
  state;
- message task candidate refresh in
  `backend/src/domains/tasks/candidates.rs` uses the engine for explicit
  commitment/request language that the legacy task scanner does not match;
- related candidate behavior still appears in task candidate, task rule and
  task intelligence modules.

The engine baseline remains candidate-first. The message task candidate wiring
creates reviewable task candidates only. It does not write accepted
Obligations, create Tasks, run full provider ingestion or provide review
candidate-to-Obligation routing. The desktop Tasks workspace now provides a
scoped review panel for already persisted entity-scoped Obligations and
Decisions.

## Migration Plan

1. Keep extracted obligations as candidates until reviewed.
2. Feed reviewed candidates into the ADR-0088 Obligations domain model.
3. Extend the current message task candidate wiring to full Communication,
   meeting and document ingestion.
4. Link accepted obligations to tasks, events and communications.
5. Use Risk Engine for overdue or blocked obligations.
6. Use Consistency / Contradiction Engine when evidence conflicts with
   fulfillment state.
