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

There is no dedicated backend module, migration, table or review workflow for
this engine in the current repository. It is a target architecture concept
approved during product documentation refactoring.

## Migration Plan

1. Keep this spec as the source for Polygraph terminology.
2. Add an ADR before introducing persistence or public API behavior.
3. Start with reviewable contradiction observations, not automatic memory
   rewrites.
4. Use communications and documents as the first evidence sources.
5. Feed reviewed outcomes into Memory, Trust, Risk and Relationship semantics.
