# ADR-0059 Person Communication DNA and Personas

Status: Proposed

## Context

The functional spec distinguishes between Roles (who the person is to the user) and Personas (how the user interacts in a specific context). Communication DNA captures the person's natural style independently of any persona: formality, verbosity, technical depth, call preference, and response patterns.

## Decision

Store Communication DNA as typed columns on the `persons` table (`communication_style`, `verbosity`, `technical_depth`, `question_frequency`, `call_preference`, `response_pattern`, `active_hours`, `active_days`). Personas live in `person_personas` as named interaction contexts with their own tone, language, and channel preferences. The `PersonIntelligenceService` computes DNA from message corpus with heuristic fallback and optional LLM refinement via Ollama.

## Consequences

- DNA is always available even when Ollama is offline (heuristic computation).
- Personas override DNA defaults during compose/reply flows.
- DNA columns are nullable; missing values indicate uncomputed profile.
