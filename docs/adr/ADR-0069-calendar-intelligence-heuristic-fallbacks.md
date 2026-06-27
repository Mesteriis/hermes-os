# ADR-0069 Calendar Intelligence Layer with Heuristic Fallbacks

Status: Proposed

## Context

The calendar module needs event classification, importance scoring, readiness assessment, risk detection, meeting briefs, agenda generation, and natural-language search. Ollama is available but must not be mandatory per ADR-0009 (local AI through Ollama) and ADR-0022 (no fine-tuning on private data).

## Decision

All intelligence features use deterministic heuristics as primary implementation with optional Ollama refinement. Heuristics cover:
- **Event classification** — keyword analysis of title, participant count, duration
- **Importance scoring** — urgency keywords, participant count, project/deadline presence
- **Readiness scoring** — checklist completion (agenda, docs, context, participants)
- **Risk detection** — missing agenda, missing docs, no participants, no project link, upcoming-soon gap
- **Meeting brief** — aggregation of event data, participants, context pack
- **Agenda generation** — template-based per event type (meeting, review, planning)
- **Brain search** — ILIKE over title and description; structured weekly overview

The `CalendarIntelligenceService` is a pure function service (no state). The `CalendarBrainService` accepts a `PgPool` reference for database queries. All functions have explicit `CalendarIntelligenceError` / `CalendarBrainError` types.

## Consequences

- Calendar intelligence works without Ollama running.
- Heuristics are transparent, debuggable, and fast.
- Ollama can be added later as an optional refinement layer without changing the API.
- Template-based agenda generation produces predictable, domain-appropriate results.
