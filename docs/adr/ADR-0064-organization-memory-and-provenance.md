# ADR-0064 Organization Memory and Provenance

Status: Proposed

## Context

Organizations accumulate facts, memory cards, preferences, and timeline events. Like Personas (ADR-0057), every piece of information must carry provenance: source, confidence, and verification timestamp.

## Decision

Store facts in `organization_facts`, memory cards in `organization_memory_cards`, preferences in `organization_preferences`, and timeline events in `organization_timeline_events`. All carry mandatory `source`, `confidence`, and `last_verified_at` columns. Memory decay lowers confidence for unverified facts. Snapshots (`organization_snapshots`) enable history diff. Knowledge conflicts are detected and surfaced in `organization_knowledge_conflicts`.

## Consequences

- Every organizational fact is traceable to its source.
- Required documents (`organization_required_documents`) track what documents an organization typically needs.
- Timeline is rebuildable from communication history and document metadata.
- History diff works by comparing two snapshots.
