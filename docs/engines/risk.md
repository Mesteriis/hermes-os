# Risk Engine

The Risk Engine detects evidence-backed conditions that may require attention.

Risk is not generic anxiety text. A risk observation must identify what is at
risk, why, and which evidence supports it.

## Responsibilities

The Risk Engine produces:

- risk observations;
- attention signals;
- blocker signals;
- stale-state warnings;
- suspicious-source signals;
- risk review items.

It does not own:

- task status;
- project state;
- organization truth;
- relationship truth;
- final owner decisions.

## Inputs

- tasks;
- projects;
- organizations;
- communications;
- obligations;
- contradiction observations;
- trust signals;
- deadlines and events.

## Output Requirements

Risk output must include:

- affected entity;
- risk type;
- evidence;
- confidence;
- severity;
- suggested handling state;
- review state.

## Current Implementation Evidence

Risk-like behavior is currently named through domain-local `health` and
`intelligence` modules in Calendar, Organizations, Persons and Tasks. Those
modules should be treated as current implementation surfaces, not separate
engine ownership.

The first backend Risk Engine baseline lives in `backend/src/engines/risk.rs`.
It derives an attention status from unresolved risk severities:

- no unresolved risk -> `healthy`;
- unresolved `low` or `medium` risk -> `needs_attention`;
- unresolved `high` or `critical` risk -> `at_risk`.

The current Persona compatibility layer uses that output to update
`persons.health_status`. This field remains a compatibility cache, not the
Risk Engine source of truth.

The shared engine also builds source-backed Persona risk observation drafts.
Those drafts preserve affected entity, risk type, evidence, source, confidence,
severity, suggested handling state and suggested review state. The current
`PersonRiskStore` uses this draft before writing compatibility `person_risks`
records and then derives the legacy `persons.health_status` cache from the
unresolved observations.

## Migration Plan

1. Keep the shared attention-status logic in the Risk Engine.
2. Normalize health/watchtower language into Risk Engine semantics.
3. Keep risk source-backed and reviewable.
4. Do not let risk output directly mutate domain state without policy.
5. Route cross-domain risk observations through review workflows before they
   alter task, project, organization or relationship state.
6. Connect risk observations to tasks, obligations and project context.
