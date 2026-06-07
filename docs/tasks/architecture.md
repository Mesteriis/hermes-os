# Tasks — Архитектура

## Модули

| Модуль | Назначение |
|---|---|
| `tasks.rs` | Ядро: Task + CRUD + status transitions |
| `task_core.rs` | Providers, external identities, context pack, evidence, relations, checklists, subtasks |
| `task_intelligence.rs` | Priority, risk, readiness scoring, missing context, next action |
| `task_brain.rs` | Explain task, search, daily brief |
| `task_health.rs` | Overdue, stale, waiting too long, without context, cycle time, workload |
| `task_rules.rs` | Rules + templates |
| `task_sync.rs` | Markdown/JSON export |

## Слои

```
API (lib.rs handlers)
  ↓
Domain services (task_intelligence, task_brain, task_health)
  ↓
Stores (PgPool-backed)
  ↓
PostgreSQL (12+ tables)
```
