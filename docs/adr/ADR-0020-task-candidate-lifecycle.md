# ADR-0020 Task Candidate Lifecycle

Status: Proposed

## Context

AI can extract tasks from messages and documents, but false positives can create operational noise or false obligations.

## Decision

AI extraction creates task candidates. Activation requires user confirmation or a narrowly scoped policy.

## Consequences

- The user remains in control of commitments.
- Task provenance remains clear.
- UI must support efficient review.
- Automation policies require careful design later.
