# Tasks Domain

## Responsibilities

The Tasks domain owns concrete actionable units with lifecycle, owner, status,
evidence and links to context.

A Task is not the same as an Obligation or Follow-Up:

- an Obligation is a commitment or duty with evidence;
- a Follow-Up is a prompt to revisit something;
- a Task is an actionable unit with status lifecycle.

## Task Sources

- manual creation;
- Communication extraction;
- Document extraction;
- meeting summary;
- Obligation Engine output;
- agent suggestion;
- imported task provider in future versions.

## Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Candidate
    Candidate --> Active
    Candidate --> Rejected
    Active --> Waiting
    Active --> Done
    Active --> Canceled
    Waiting --> Active
    Done --> Archived
    Canceled --> Archived
```

## Required Fields

- title;
- source or manual provenance;
- status;
- owner;
- created_at;
- updated_at;
- optional deadline;
- optional reminder policy;
- linked entities.

## Extraction Rules

AI extraction and engine output create task candidates, not automatically active
commitments, unless a user policy explicitly allows auto-activation for a
low-risk source.

## Audit

Status changes, deadline changes, assignment changes and deletions must emit
events.
