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

Related behavior currently appears in task candidate, task rule and task
intelligence modules. No dedicated Obligations domain or engine implementation
exists yet.

## Migration Plan

1. Keep extracted obligations as candidates until reviewed.
2. Add a dedicated implementation plan before persistence changes.
3. Link accepted obligations to tasks, events and communications.
4. Use Risk Engine for overdue or blocked obligations.
5. Use Consistency / Contradiction Engine when evidence conflicts with
   fulfillment state.
