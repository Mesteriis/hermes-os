# Communication To Obligation

This workflow explains how a communication can create an obligation, follow-up
or task candidate.

## Trigger

The workflow starts when a communication contains a commitment, request, due
date, expectation or responsibility.

Examples:

- someone asks the owner to send a document;
- the owner promises to call back;
- a provider states a deadline;
- a meeting follow-up appears in an email thread.

## Flow

```text
source communication
  -> preserve source evidence
  -> extract commitment language
  -> identify obligated party
  -> identify beneficiary or counterparty
  -> detect due date or condition
  -> create obligation candidate
  -> create optional task candidate
  -> review or policy gate
  -> store accepted obligation and linked task if needed
```

## Required Outputs

- source evidence reference;
- obligation candidate;
- optional task candidate;
- linked Personas, Organizations, Projects or Events;
- confidence and review state;
- risk signal when obligation is urgent or overdue.

## Domain And Engine Boundaries

- Communications owns the source evidence.
- Obligation Engine creates candidates.
- Obligations domain owns accepted obligations.
- Tasks domain owns task lifecycle.
- Risk Engine detects overdue, blocked or high-impact obligations.

## Current Implementation Evidence

Current related implementation exists through task candidates, task rules, task
intelligence and communication workflow state. The accepted Obligation
persistence baseline exists in `backend/src/domains/obligations/mod.rs`.
Message task candidate refresh now uses `backend/src/engines/obligation.rs`
for explicit commitment/request language when the legacy task scanner does not
match.

This is still not the full communication-to-obligation workflow. Accepted
Obligation backend list/review routes exist, but candidate-to-Obligation
creation, provider-wide extraction, meeting/document adapters and desktop
review UI routing remain incomplete.

## Migration Plan

1. Keep obligations distinct from tasks.
2. Require review before converting candidates into accepted obligations.
3. Link accepted obligations to tasks only when action is needed.
4. Add extraction and review workflow before automated capture.
