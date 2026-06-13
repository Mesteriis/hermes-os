# Relationships Domain

Relationships are first-class source-of-truth records connecting Hermes world
entities.

Hermes is relationship-first: Personas, Organizations, Projects,
Communications, Documents, Tasks, Events, Decisions, Obligations and Knowledge
items gain meaning from source-backed relationships.

## Responsibilities

The Relationships domain owns:

- durable relationship records;
- source and target entity references;
- relationship type;
- trust score;
- strength score;
- confidence;
- provenance evidence;
- review state;
- validity period;
- relationship metadata.

The Relationships domain does not own:

- graph traversal indexes;
- timeline rendering;
- trust computation;
- risk computation;
- dossier generation;
- automatic contradiction resolution.

Those are engine or projection responsibilities.

## Persona Relationships

Persona-to-Persona relationships are the first implementation path:

```yaml
Relationship:
  source_entity_kind: persona
  source_entity_id:
  target_entity_kind: persona
  target_entity_id:
  relationship_type:
  trust_score:
  strength_score:
  confidence:
  review_state:
```

Examples:

- knows;
- collaborates_with;
- works_with;
- reports_to;
- represents;
- assists;
- owns;
- member_of;
- introduced.

## Evidence

Every durable Relationship must have source evidence:

```yaml
RelationshipEvidence:
  relationship_id:
  source_kind:
  source_id:
  excerpt:
  metadata:
```

Evidence can come from Communications, Documents, Events, Memory, Knowledge,
Decisions, Obligations, Tasks, Projects, Organizations, Personas or raw source
records.

AI may propose a Relationship, but it must remain source-backed and reviewable.

## Relationship Versus Graph Edge

A Relationship is a durable domain record.

A graph edge is a traversal/projection record.

Graph edges may be derived from Relationships, but they are not the only source
of truth for relationship semantics.

## Current Implementation Evidence

Current backend baseline:

- `backend/migrations/0060_create_relationships.sql`;
- `backend/migrations/0061_relationship_graph_projection.sql`;
- `backend/migrations/0068_expand_relationship_graph_node_kinds.sql`;
- `backend/src/domains/relationships/mod.rs`;
- `backend/src/domains/relationships/api.rs`;
- `backend/tests/relationships.rs`;
- `backend/tests/relationships_api.rs`;
- ADR-0086.

This baseline provides first-class Relationship persistence, validation and
graph projection for the current `RelationshipEntityKind` set: Persona,
Organization, Project, Communication, Document, Task, Event, Decision,
Obligation and Knowledge. Guarded backend routes can list Relationships by
entity or by review state and change review state while keeping the graph
projection aligned. The Personas workspace includes a desktop review panel for
global suggested Relationships, while still formatting selected-Persona
relationships compactly when a Persona is selected. It does not yet provide
role adapters, organization/project/task link adapters, cross-domain workflow
placement or timeline projection.

## Migration Direction

1. Keep `relationships` as the durable source-of-truth table.
2. Reclassify `person_roles`, organization links, project links and task
   relations as compatibility or read-model surfaces.
3. Feed Relationship records into Trust, Risk, Timeline, Memory and Dossier
   projections.
4. Move or duplicate Relationship review into a broader cross-domain workflow
   inbox after that shell exists.
