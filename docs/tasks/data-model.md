# Tasks — Модель данных

## Таблицы

| Таблица | Фаза | Назначение |
|---|---|---|
| `tasks` (расширена) | 0 | Задачи с полным доменом (status, priority, risk, readiness, context) |
| `task_candidates` | — | AI-извлечённые кандидаты (существующая) |
| `task_provider_accounts` | 1 | Аккаунты внешних трекеров |
| `external_task_identities` | 1 | Связь локальных задач с внешними |
| `provider_status_mappings` | 1 | Маппинг статусов |
| `task_context_packs` | 2 | Контекст-паки |
| `task_evidence` | 2 | Доказательства происхождения |
| `task_relations` | 2 | Связи между задачами и сущностями |
| `task_checklists` | 2 | Чеклисты |
| `task_subtasks` | 2 | Подзадачи |
| `task_rules` | 4 | Правила автоматизации |
| `task_templates` | 4 | Шаблоны задач |
| `task_snapshots` | 4 | Снимки состояния |

## ID-форматы

| Сущность | Формат |
|---|---|
| Task | `task:v1:{nanos_hex}` |
| Task Provider | `tprov:v1:{nanos_hex}` |
| Task Rule | `taskrule:v1:{nanos_hex}` |

## Статусы

`new` → `triaged` → `ready` → `in_progress` → `review` → `done` → `archived`
При блокировке: `→ blocked`, при ожидании: `→ waiting`

## Pre-seeded Templates

bug, feature, research, contract_review, aeat_response, client_followup, invoice_review, code_review
