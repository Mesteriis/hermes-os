# ADR-0072 Task Intelligence with Heuristic Fallbacks

Status: Proposed

## Context

The task module needs priority scoring, risk analysis, readiness assessment, missing context detection, and next-action suggestions. Ollama is available but must not be mandatory per ADR-0009.

## Decision

All intelligence features use deterministic heuristics:
- **Priority**: weighted by deadline proximity, legal/tax context, contact presence, blockers
- **Risk**: deadline closeness, missing docs, no owner, external dependencies, legal context
- **Readiness**: description, context pack, docs, deadline, no blockers, contacts resolved
- **Missing context**: checklist-based gap detection
- **Next action**: template-based per status

Ollama refinement is optional and can be added without API changes.

## Consequences

- Task intelligence works without Ollama running.
- Heuristics are transparent, fast, and debuggable.
- Priority/risk/readiness scores are stored on the task row for sorting and filtering.
