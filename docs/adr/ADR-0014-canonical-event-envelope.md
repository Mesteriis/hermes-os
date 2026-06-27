# ADR-0014 Canonical Event Envelope

Status: Proposed

## Context

Events from providers, user actions, document processing and agents need consistent metadata for replay, audit and provenance.

## Decision

Define a canonical event envelope with event ID, type, schema version, timestamps, source, actor, subject, payload, provenance, causation and correlation IDs.

## Consequences

- Cross-domain events can be processed uniformly.
- Replay and audit become practical.
- All producers must populate required metadata.
- Payload schemas require version discipline.
