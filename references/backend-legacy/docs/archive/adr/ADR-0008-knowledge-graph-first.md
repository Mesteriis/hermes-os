# ADR-0008 Knowledge Graph First

Status: Proposed

## Context

The product's core value is long-term relationships between people, organizations, projects, documents, messages, tasks and decisions.

## Decision

Make the knowledge graph a first-class architectural component, with relationships represented as durable records carrying provenance and confidence.

## Consequences

- Memory queries can use relationship context, not only text similarity.
- Inferred links can be reviewed and corrected.
- Graph schema design becomes central early work.
- UI must expose graph value without overwhelming daily workflows.
