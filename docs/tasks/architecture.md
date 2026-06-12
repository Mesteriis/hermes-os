# Tasks Architecture

## Position

The Tasks domain owns actionable work-item lifecycle and evidence links. It uses
shared engines for priority, readiness, risk, context and obligation extraction.

## Modules

Paths below refer to `backend/src/domains/tasks/`.

| Module | Responsibility |
|---|---|
| `core.rs` | Task core, providers, external identities, evidence, relations, checklists, subtasks and status transitions |
| `candidates.rs` | task candidates created from Communications, Documents, Events or engines |
| `intelligence.rs` | engine-facing priority/readiness helpers |
| `brain.rs` | context answers over Search/Memory engines |
| `health.rs` | Risk Engine/attention views for stale or blocked work |
| `rules.rs` | rules and templates |
| `sync.rs` | export/import helpers |
| `api.rs` | current route handlers and DTO-facing compatibility surface |

## Layers

```text
API
  -> Task domain services
  -> shared engines for context, risk, obligations and search
  -> stores
  -> PostgreSQL
```

## Domain Rules

- AI and engine extraction produce candidates.
- Low-confidence candidates require review.
- Confirmed candidates can become Tasks.
- Obligations remain separate commitments even when they generate Tasks.
- Status changes and destructive operations must be auditable.
