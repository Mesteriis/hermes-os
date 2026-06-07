# ADR-0066 Organization Graph Integration

Status: Proposed

## Context

Organizations must participate in the knowledge graph to surface relationships between organizations, persons, documents, projects, and domains. ADR-0045 established the graph core projection with PostgreSQL tables.

## Decision

Organizations participate in the existing `graph_nodes`/`graph_edges` tables. New relationship types: `org_has_domain`, `org_has_contact`, `org_has_document`, `org_involved_in_project`, `org_parent_of` (parent/subsidiary). The organization graph, relationship map, and mutual connections are read-side graph traversal queries, not separate storage. The `related_organizations` table provides explicit parent/subsidiary/division/partner relationships with provenance.

## Consequences

- No new graph tables; organizations reuse the existing graph infrastructure.
- Related organizations are queryable both through the graph and through the direct `related_organizations` table.
- Graph traversal depth is intentionally limited; complex queries use application-layer joins.
