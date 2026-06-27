# ADR-0021 Calendar as Event Source

Status: Proposed

## Context

Meetings and calendar changes are important context for projects, tasks and communications.

## Decision

Treat calendars as event sources that produce meeting, schedule and attendance events, not merely UI widgets.

## Consequences

- Meeting context can enrich graph and search.
- Calendar provider changes need idempotent sync.
- Meeting completion can trigger summaries and task extraction.
- Outbound calendar edits require capability checks.
