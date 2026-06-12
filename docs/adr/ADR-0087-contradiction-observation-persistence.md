# ADR-0087 Contradiction Observation Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0023 Rebuildable Projections
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0086 First-Class Relationship Persistence

## Context

Hermes is a Personal Memory System. New Communications, Documents, Events,
Decisions and Obligations can contradict accepted Memory and Knowledge.

ADR-0085 introduced the Consistency / Contradiction Engine, user-facing alias
Polygraph. The repository currently has no durable backend representation for
Polygraph observations. Without persistence, contradictions cannot be reviewed,
linked to source evidence or fed into Memory, Trust, Risk and Relationship
semantics.

## Decision

Introduce `ContradictionObservation` persistence as the first implementation
slice of the Consistency / Contradiction Engine.

The engine stores reviewable observations:

```yaml
ContradictionObservation:
  observation_id:
  old_source_kind:
  old_source_id:
  new_source_kind:
  new_source_id:
  affected_entities:
  conflict_type:
  old_claim:
  new_claim:
  confidence:
  severity:
  review_state:
  metadata:
```

Initial review states are:

```yaml
ContradictionReviewState:
  suggested
  user_confirmed
  user_rejected
```

Initial severities are:

```yaml
ContradictionSeverity:
  low
  medium
  high
  critical
```

The first detection path operates on structured claims produced by upstream
extraction or deterministic tests:

```yaml
AcceptedClaim:
  subject_id:
  claim_type:
  value:
  source_kind:
  source_id:

NewEvidenceClaim:
  subject_id:
  claim_type:
  value:
  source_kind:
  source_id:
```

When a new claim has the same subject and claim type but a different normalized
value, the engine creates a `direct_contradiction` observation.

The engine must not:

- overwrite accepted Memory or Knowledge;
- change source records;
- mark a Persona as dishonest;
- adjust Relationship trust automatically;
- resolve the conflict without owner review or an explicit future policy.

## Consequences

Positive:

- Polygraph becomes a concrete backend engine baseline.
- Contradictions become source-backed and reviewable.
- Memory and Knowledge remain protected from silent mutation.
- Future Trust, Risk and Relationship engines can consume reviewed outcomes.

Negative:

- The first detector only handles structured direct contradictions.
- Desktop review UI is still separate follow-up work.
- Provider-wide ingestion and natural-language claim extraction from
  Communications and Documents remain outside this first slice.

## Non-Goals

- Natural-language contradiction detection.
- Review UI.
- Automatic memory update.
- Automatic trust, risk or relationship score changes.
- Punitive judgments about Personas.

## Implementation Status

The backend now includes guarded routes for listing open contradiction
observations and changing review state:

- `GET /api/v1/contradictions`
- `PUT /api/v1/contradictions/{observation_id}/review`

These routes record API audit events and do not automatically overwrite Memory,
Trust, Risk or Relationship state.

`backend/src/engines/consistency.rs` also includes a deterministic extraction
baseline for simple structured Communication and Document evidence lines such
as `status: blocked` or `location=Madrid`. This converts evidence text into
`NewEvidenceClaim` values and reuses the same direct-contradiction detector.

## Required Follow-Up

- Add desktop review UI for contradiction observations.
- Connect provider-wide Communication and Document ingestion to structured
  claim extraction.
- Add natural-language claim extraction behind explicit review policy.
- Link reviewed outcomes to Memory, Trust, Risk and Relationship semantics.
