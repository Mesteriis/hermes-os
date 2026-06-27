# ADR-0061 Organization as First-Class Domain Entity

Status: Proposed

## Context

The persons module has `organization_reference` as a free-text field. The functional spec requires organizations as independent entities with their own identities, memory, timeline, contacts, and enrichment. A string field cannot support this.

## Decision

Organizations are first-class domain entities with `organization_id = org:v1:{nanos}`. The `organization_contact_links` table provides many-to-many linkage between persons and organizations with role, department, and primary-flag semantics. The `organization_reference` field on persons becomes a cached value derived from the primary active link.

## Consequences

- 27 tables under the organizations domain.
- `organization_contact_links` enables one person to belong to multiple organizations with different roles.
- Person merge/split may trigger organization contact link reconciliation.
- The free-text `organization_reference` field is retained for backward compatibility.
