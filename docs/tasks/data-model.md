# Tasks Data Model

## Tables

| Table | Phase | Purpose |
|---|---|---|
| `tasks` | 0 | actionable work items with status, priority, readiness and context fields |
| `task_candidates` | existing | AI/engine-extracted candidates requiring review |
| `task_provider_accounts` | 1 | external tracker accounts |
| `external_task_identities` | 1 | local-to-provider task identity mapping |
| `provider_status_mappings` | 1 | provider status mapping |
| `context_packs` | 2 | engine-owned derived task context packs; legacy `task_context_packs` is historical schema compatibility only |
| `task_evidence` | 2 | provenance for extracted or linked tasks |
| `task_relations` | 2 | links between tasks and world-model entities |
| `task_checklists` | 2 | checklist items |
| `task_subtasks` | 2 | task decomposition |
| `task_rules` | 4 | task automation rules |
| `task_templates` | 4 | task templates |
| `task_snapshots` | 4 | state snapshots |

## ID Formats

| Entity | Format |
|---|---|
| Task | `task:v1:{nanos_hex}` |
| Task Provider | `tprov:v1:{nanos_hex}` |
| Task Rule | `taskrule:v1:{nanos_hex}` |

## Statuses

`new` -> `triaged` -> `ready` -> `in_progress` -> `review` -> `done` -> `archived`

Blocking: `blocked`.

Waiting: `waiting`.

## Obligation Boundary

Obligations are commitments or duties with evidence. A Task may implement an
Obligation, but the Task table is not the canonical model for every Obligation.

## Pre-Seeded Templates

Bug, feature, research, contract review, AEAT response, client follow-up,
invoice review and code review templates are task creation aids, not domain
definitions.
