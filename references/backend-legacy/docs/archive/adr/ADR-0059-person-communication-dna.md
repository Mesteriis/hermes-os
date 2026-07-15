# ADR-0059 Person Communication DNA and Personas

Status: Superseded by ADR-0084

Superseded because ADR-0084 makes Persona the root domain entity. Communication
patterns remain part of Persona Intelligence, but `person_personas` as nested
interaction contexts is no longer the target domain model.

## Context

The functional spec distinguishes between Roles (who the person is to the user) and Personas (how the user interacts in a specific context). Communication DNA captures the person's natural style independently of any persona: formality, verbosity, technical depth, call preference, and response patterns.

## Historical Decision

The original decision stored Communication DNA as typed columns on the
historical `persons` table and modeled nested interaction personas through
`person_personas`. That naming is no longer the target model.

Current implementation direction is owned by ADR-0084: Persona is the root
domain entity, storage is persona-native, and the former nested persona concept
is treated as deprecated interaction-context compatibility.

## Consequences

- DNA is always available even when Ollama is offline (heuristic computation).
- Personas override DNA defaults during compose/reply flows.
- DNA columns are nullable; missing values indicate uncomputed profile.
