# ADR-0060 Person Timeline and Graph Integration

Status: Proposed

## Context

Relationship events (first message, contract signed, invoice paid, etc.) form a timeline that must be queryable and rebuildable. The relationship map and mutual connections views need to surface graph relationships between Personas, projects, documents, and other entities.

## Decision

Store timeline events in `relationship_events` with optional links to source entities (`related_entity_id`, `related_entity_kind`). The timeline is a rebuildable projection materialized from communication history and document metadata. Graph integration uses existing `graph_nodes`/`graph_edges` tables from ADR-0045 with Persona relationship types (`persona_has_identity`, `persona_works_at_organization`, `persona_has_expertise`, `persona_involved_in_project`). Relationship map and mutual connections are graph traversal queries, not separate storage.

## Consequences

- Timeline is queryable by event type, date range, and related entity.
- History diff works by comparing Persona snapshots across dates.
- Graph traversal depth is intentionally limited; complex queries use application-layer joins.
- No new graph tables; Personas participate in the existing graph projection.
