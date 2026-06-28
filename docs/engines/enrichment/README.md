# Enrichment Engine

Status: documentation package aligned to the current repository structure.

The Enrichment Engine proposes new candidate knowledge.

Enrichment is not authority. It suggests updates that domains can review or
accept according to policy.

## Responsibilities

The Enrichment Engine produces:

- entity candidates;
- relationship candidates;
- profile and dossier observations;
- missing-field suggestions;
- conflict candidates;
- evidence-backed enrichment notes.

It does not own:

- accepted domain truth;
- external-source policy;
- provider credentials;
- automatic overwrites.

## Inputs

- provider records;
- public or approved local sources;
- documents;
- communications;
- graph context;
- existing memory;
- owner-approved enrichment policies.

## Output Requirements

Enrichment output must include:

- source;
- extracted claim;
- affected entity;
- confidence;
- review state;
- freshness;
- conflict markers when it disagrees with accepted memory.

## Current Implementation Evidence

Current implementation includes enrichment modules under Persons and
Organizations. That is a current shape, not a reason to create separate
per-domain engines.

The first backend Enrichment Engine baseline lives in
`backend/src/engines/enrichment/`. It converts deprecated Persona
compatibility `persons.is_favorite` state into a source-backed preference draft:

- preference type: `ui:favorite`;
- value: `true`;
- source: `persons.is_favorite:<persona_id>`;
- confidence: `1.0`.

`PersonEnrichmentStore` uses this draft when materializing compatibility
`person_preferences`. Turning the favorite state off removes the compatibility
preference and does not create a replacement draft.

The shared engine also builds source-backed Persona observation candidates for
legacy `enrichment_results`. These candidates preserve:

- affected entity kind and ID;
- source reference;
- extracted claim;
- confidence;
- pending review state;
- freshness marker;
- conflict marker.

`EnrichmentResultStore` uses this draft before writing compatibility
`enrichment_results`, storing the candidate metadata under `_enrichment` in the
existing JSON payload without changing the table shape.

## Migration Plan

1. Keep compatibility favorite-to-preference assembly in the Enrichment Engine.
2. Keep enrichment outputs as candidates or observations.
3. Define approved sources per domain before automated enrichment.
4. Feed conflicts to the Consistency / Contradiction Engine.
5. Preserve provenance and freshness for all suggested updates.
