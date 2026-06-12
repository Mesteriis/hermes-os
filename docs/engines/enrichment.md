# Enrichment Engine

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

## Migration Plan

1. Keep enrichment outputs as candidates or observations.
2. Define approved sources per domain before automated enrichment.
3. Feed conflicts to the Consistency / Contradiction Engine.
4. Preserve provenance and freshness for all suggested updates.
