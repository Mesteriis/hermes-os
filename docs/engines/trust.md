# Trust Engine

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

## Migration Plan

1. Keep trust derivation source-backed.
2. Avoid storing unexplained trust values.
3. Reconcile Persona trust, Organization risk and Relationship strength through
   shared engine language.
4. Use contradiction observations as trust inputs, not as automatic judgments.
