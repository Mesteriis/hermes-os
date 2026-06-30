# Attention Engine

Status: initial backend contract.

The Attention Engine ranks, groups and explains Review-backed candidates. It is
the engine boundary for Radar/attention vocabulary; it is not a durable domain,
store, task list or promotion owner.

## Current Implementation Evidence

Current backend files:

- `backend/src/engines/attention/mod.rs`;
- `backend/src/engines/attention/models.rs`;
- `backend/src/engines/attention/engine.rs`;
- `backend/src/engines/attention/errors.rs`.

The first contract accepts evidence-backed `AttentionCandidate` inputs and
returns `AttentionCard` outputs with:

- stable card id;
- title and summary;
- importance and confidence;
- evidence count;
- related entities;
- trace id;
- Review item ids;
- suggested actions;
- source summary;
- explicit explainability fields.

## Boundary Rule

Attention Cards are rebuildable read-model output over Review and evidence.
They must preserve Review item ids, observation evidence references and trace
ids. The engine must not create Tasks, Personas, Organizations, Documents,
Knowledge, Decisions, Obligations or Relationships directly.
