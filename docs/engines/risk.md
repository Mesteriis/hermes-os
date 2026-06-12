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

## Migration Plan

1. Normalize health/watchtower language into Risk Engine semantics.
2. Keep risk source-backed and reviewable.
3. Do not let risk output directly mutate domain state without policy.
4. Connect risk observations to tasks, obligations and project context.
