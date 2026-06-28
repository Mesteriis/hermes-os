# Trust Engine

Status: documentation package aligned to the current repository structure.

The Trust Engine computes relationship and source reliability signals.

Trust is not a vague profile field. Durable trust belongs to Relationship
records or source reliability observations when accepted.

## Responsibilities

The Trust Engine produces:

- trust signals;
- source reliability scores;
- confidence adjustments;
- relationship strength inputs;
- review recommendations;
- explanation of trust changes.

It does not own:

- relationship source of truth;
- Persona identity;
- Organization identity;
- final user judgment.

## Inputs

- relationship history;
- accepted and rejected suggestions;
- fulfilled or broken obligations;
- source consistency;
- communication patterns;
- contradiction observations;
- owner feedback.

## Output Requirements

Trust output must include:

- affected relationship or source;
- signal type;
- evidence;
- confidence;
- direction of impact;
- explanation suitable for review.

## Current Implementation Evidence

Current implementation includes `backend/src/domains/persons/trust.rs` and
trust-related Persona Intelligence language. This should be treated as an
implementation-local stage on the way to shared Trust Engine semantics.

The first backend Trust Engine baseline lives in `backend/src/engines/trust/`.
It converts the deprecated Persona compatibility `persons.trust_score` value
into a source-backed Relationship signal:

- relationship type: `trusts`;
- trust score: clamped compatibility score normalized from `0..100` to `0..1`;
- strength score: `0.5` until relationship strength has independent evidence;
- confidence: `1.0` because the adapter records an explicit compatibility
  source, not an inferred private judgment.

`PersonEnrichmentStore` uses this signal when materializing suggested Owner
Persona -> Persona `trusts` Relationships. The root `persons.trust_score`
column remains a temporary compatibility cache.

The shared engine also builds source reliability signals for reviewable
evidence. `PersonEnrichmentStore` records this signal in Relationship evidence
metadata when adapting compatibility `persons.trust_score` input, preserving:

- affected source;
- signal type;
- evidence;
- confidence;
- impact direction;
- review explanation.

## Migration Plan

1. Keep compatibility-score normalization in the Trust Engine.
2. Keep trust derivation source-backed.
3. Avoid storing unexplained trust values.
4. Reconcile Persona trust, Organization risk and Relationship strength through
   shared engine language.
5. Use contradiction observations as trust inputs, not as automatic judgments.
