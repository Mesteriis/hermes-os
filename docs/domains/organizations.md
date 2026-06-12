# Organizations Domain

Organizations are first-class memory anchors for collective actors such as
companies, institutions, agencies, communities and product teams.

An Organization is not a Persona, Project or CRM account object.

## Responsibilities

The Organizations domain owns:

- organization identity;
- legal and display names;
- domains, websites and external identifiers;
- departments and sub-units;
- relationships to Personas;
- relationships to Projects;
- portals, procedures and playbooks;
- organization-specific memory;
- risk, finance and enrichment observations with provenance.

The Organizations domain does not own:

- Persona identity resolution;
- Project lifecycle;
- Communication source records;
- global Memory, Timeline, Search or Risk engines.

## Relationship Boundary

Organizations connect to Personas and Projects through first-class
relationships, not embedded fields.

Examples:

- Persona works for Organization;
- Persona represents Organization;
- Organization sponsors Project;
- Organization provides service to Owner Persona;
- Organization owns Portal or Procedure.

Relationship records require provenance, confidence and validity period when
time-bounded.

## Memory Boundary

Organization memory answers questions such as:

- what this organization does;
- how to interact with it;
- what procedures or portals matter;
- which Personas are associated with it;
- what risks or obligations exist;
- which Projects, Documents, Communications and Decisions reference it.

Organization memory is evidence-backed. It can use Memory Engine views, but the
Organization domain owns only organization-specific source records and accepted
facts.

## Current Implementation Evidence

Current backend implementation includes:

- `backend/src/domains/organizations/*`;
- organization identity, department, memory, timeline, workflow, finance,
  enrichment, risk and alert migrations `0038` through `0043`;
- Organizations frontend page.

This is closer to the target model than several other domains, but relationship
and engine boundaries still need to be kept explicit in future plans.

## Migration Plan

1. Keep Organization as a separate domain, not a subtype of Persona.
2. Use `organization_proxy` Persona only when an organization-like actor must
   participate in Persona-to-Persona memory.
3. Move relationship semantics toward the shared Relationship model.
4. Keep enrichment and risk outputs as engine-derived observations with
   citations.
5. Avoid reintroducing CRM account language in organization documentation.
