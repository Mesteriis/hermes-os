# ADR-0057 Person Memory and Provenance System

Status: Proposed

## Context

The functional spec requires every AI-extracted or discovered fact about a person to carry provenance: source, confidence, and verification timestamp. Ad-hoc storage in JSON columns or free-text notes breaks auditability and prevents systematic conflict detection and memory decay.

## Decision

Store all facts, memory cards, preferences, and expertise in dedicated domain tables (`person_facts`, `person_memory_cards`, `person_preferences`, `person_expertise`) with mandatory `source`, `confidence`, and `last_verified_at` columns. The enrichment engine writes through these tables and never mutates the person profile directly. Memory decay is a scheduled projection that lowers confidence for unverified facts older than a threshold.

## Consequences

- Every fact is traceable to its source.
- Knowledge conflicts (contradictory facts from different sources) are detectable.
- Memory decay provides automatic staleness detection.
- Snapshot-based history diff is possible via `person_snapshots`.
- Relationship timeline events (`relationship_events`) form an event-sourced projection.
