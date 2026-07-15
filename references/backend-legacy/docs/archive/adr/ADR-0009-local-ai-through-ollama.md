# ADR-0009 Local AI Through Ollama

Status: Proposed

## Context

The product is local-first and handles private communications and documents. AI should work without mandatory cloud model calls.

## Decision

Use Ollama as the initial local AI runtime boundary.

## Consequences

- Local inference is available by default.
- Model replacement remains feasible.
- Performance depends on user hardware.
- Remote models, if added later, must be opt-in and policy controlled.
