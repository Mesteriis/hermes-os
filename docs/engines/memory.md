# Memory Engine

The Memory Engine assembles durable, source-backed context across Hermes.

Memory is not a generic note store and not an AI summary. Memory is accepted,
reviewable understanding derived from evidence.

## Responsibilities

The Memory Engine produces:

- memory views;
- context packs;
- memory gaps;
- stale-memory candidates;
- source-backed summaries;
- recall inputs for agents.

It does not own:

- domain entities;
- raw communications;
- document versions;
- graph relationships;
- private-data fine-tuning.

## Inputs

- canonical events;
- accepted facts;
- relationship records;
- communications;
- documents;
- projects;
- tasks;
- decisions;
- obligations;
- owner-reviewed observations.

## Output Requirements

Every durable memory output must include:

- source citations;
- affected entities;
- confidence or review state;
- created or updated time;
- actor or process that produced it;
- invalidation or supersession behavior where relevant.

## Current Implementation Evidence

Memory behavior currently exists inside domain modules such as
`persons/memory.rs` and `organizations/memory.rs`, plus project and document
memory plans. This is acceptable during migration but should not be documented
as separate engines per domain.

The first backend Memory Engine baseline lives in `backend/src/engines/memory.rs`.
It converts deprecated Persona compatibility `persons.notes` text into a
source-backed Persona memory-card draft:

- title: `Compatibility notes`;
- description: trimmed notes text;
- source: `persons.notes:<persona_id>`;
- confidence: `1.0`;
- importance: `5`.

`PersonEnrichmentStore` uses this draft when materializing compatibility
`person_memory_cards`. Empty notes remove the compatibility memory-card source
and do not create a new card.

## Migration Plan

1. Keep compatibility notes-to-card assembly in the Memory Engine.
2. Keep domain-specific memory docs focused on owned source records.
3. Move reusable memory assembly language to this engine spec.
4. Preserve source citations and review state before expanding automation.
5. Treat AI summaries as derived observations until reviewed.
