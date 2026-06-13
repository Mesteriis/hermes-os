# Consistency / Contradiction Engine

User-facing alias: Polygraph.

The Consistency / Contradiction Engine detects conflicts between new evidence
and accepted memory or knowledge.

It does not determine that a person is lying. It does not silently overwrite
truth. It creates source-backed contradiction observations and review items.

## Responsibilities

The engine produces:

- contradiction observations;
- stale fact warnings;
- disputed claim candidates;
- mismatched obligation signals;
- conflicting decision signals;
- review items for the owner or an authorized agent.

It does not own:

- accepted memory;
- domain truth;
- relationship trust;
- final conflict resolution;
- punitive judgments about Personas.

## Inputs

- new communications;
- documents and document versions;
- meeting or call notes;
- decisions;
- obligations;
- accepted facts;
- memory records;
- knowledge graph relationships;
- source reliability and trust signals.

## Detection Scope

The engine detects:

- direct contradictions;
- stale facts;
- conflicting dates;
- conflicting ownership or responsibility claims;
- obligation status conflicts;
- decision conflicts;
- claims that weaken an existing trust assumption.

## Output Model

```yaml
ContradictionObservation:
  id:
  old_source:
  new_source:
  affected_entities:
  conflict_type:
  old_claim:
  new_claim:
  confidence:
  severity:
  review_state:
  created_at:
```

## Review Rules

The engine can suggest:

- accept new claim;
- keep existing memory;
- mark both claims disputed;
- split entities;
- update relationship confidence;
- create a task or follow-up for manual verification.

The owner or an explicitly authorized policy decides what becomes accepted
memory.

## Current Implementation Evidence

Current backend baseline:

- `backend/migrations/0062_create_contradiction_observations.sql`;
- `backend/src/engines/consistency.rs`;
- `backend/src/engines/consistency_api.rs`;
- `backend/tests/consistency_contradiction.rs`;
- `backend/tests/contradictions_api.rs`;
- ADR-0087.

This baseline provides structured direct-contradiction detection and
reviewable `ContradictionObservation` persistence. It also provides a
deterministic baseline that extracts simple structured claim lines from
Communication and Document evidence, for example `status: blocked` or
`location=Madrid`, and feeds those claims into the contradiction detector.
`ContradictionObservationStore::refresh_deterministic_observations` now adds the
first backend refresh paths from projected Communication messages, imported
Documents, meeting notes and call transcripts to Polygraph: active
`person_facts` are treated as accepted Memory claims, matched through the
compatibility `persons.email_address` field, `event_participants.person_id` or
active `person_identities.telegram` identity, and compared with structured
claims extracted from message subject/body evidence, Document
title/extracted-text evidence that references the Persona email, meeting-note
content for linked event participants or successful call transcript text for
linked Telegram identities.
Guarded backend routes can list open contradiction observations and update
review state without automatically overwriting Memory. It does not yet provide
desktop review UI, non-email message-channel wiring or natural-language claim
extraction from Communications and Documents.

## Migration Plan

1. Keep this spec as the source for Polygraph terminology.
2. Keep reviewable contradiction observations, not automatic memory rewrites.
3. Expand refresh wiring beyond projected email messages, imported Documents,
   meeting notes and call transcripts to other Communication channels.
4. Add natural-language claim extraction behind explicit review policy.
5. Add desktop review UI for owner review.
6. Feed reviewed outcomes into Memory, Trust, Risk and Relationship semantics.
