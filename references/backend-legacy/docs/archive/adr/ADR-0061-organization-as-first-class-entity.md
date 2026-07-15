# ADR-0061 Organization as First-Class Domain Entity

Status: Proposed

## Context

The legacy persons module had `organization_reference` as a free-text field.
The functional spec requires organizations as independent entities with their
own identities, memory, timeline, address-book sources and enrichment. A string
field cannot support this.

## Decision

Organizations are first-class domain entities with
`organization_id = org:v1:{nanos}`. The compatibility
`organization_persona_links` table provides many-to-many linkage between
Personas and Organizations with role, department, and primary-flag semantics.
The legacy `organization_reference` field on Personas remains a cached value
derived from the primary active link until the compatibility surface is retired.

## Consequences

- 27 tables under the organizations domain.
- `organization_persona_links` enables one Persona to belong to multiple Organizations with different roles.
- Persona merge/split may trigger Organization-Persona link reconciliation.
- The free-text `organization_reference` field is retained for backward compatibility.
