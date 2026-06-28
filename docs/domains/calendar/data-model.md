# Calendar/Events — Current Data Model

This file documents current scheduled-event storage. It is not the canonical
global Timeline model. Timeline, Obligation, Risk, Search and Enrichment are
shared engines that consume calendar event records together with other sources.

## Таблицы (15)

### Core (Phase 0)

| Таблица | Назначение |
|---|---|
| `calendar_accounts` | Учетные записи провайдеров (Google, Microsoft, Apple, CalDAV, ICS, Local) |
| `calendar_sources` | Календари внутри аккаунта |
| `calendar_events` | События с source identity, статусом, типом, importance/readiness |

### Participants & Relations (Phase 1)

| Таблица | Назначение |
|---|---|
| `event_participants` | Участники с email, role, response_status и совместимым `person_id`, который должен трактоваться как Persona reference |
| `event_relations` | Связи с entity_type/entity_id (Persona, Organization, Project, Document, ...) |
| `context_packs` | Engine-owned derived context packs for calendar subjects. Legacy `event_context_packs` remains historical schema compatibility only and must not be used by runtime code. |
| `event_agendas` | Повестки встреч |
| `event_checklists` | Чеклисты подготовки |

### Meetings (Phase 3)

| Таблица | Назначение |
|---|---|
| `meeting_notes` | Заметки встреч |
| `meeting_outcomes` | Результаты: decisions, tasks, obligations, risks, follow-ups |
| `event_recordings` | Записи встреч |
| `event_transcripts` | Расшифровки |

### Scheduling (Phase 4)

| Таблица | Назначение |
|---|---|
| `deadline_events` | Дедлайны с severity и статусом |
| `focus_blocks` | Фокус-блоки с protection_level |

### Rules (Phase 7)

| Таблица | Назначение |
|---|---|
| `calendar_rules` | Правила автоматизации с approval_mode |

## ID-форматы

| Сущность | Формат |
|---|---|
| Calendar Account | `cal:v1:{nanos_hex}` |
| Calendar Source | `src:v1:{nanos_hex}` |
| Calendar Event | `evt:v1:{nanos_hex}` |
| Calendar Rule | `rule:v1:{nanos_hex}` |

## Индексы

- `calendar_events`: account_id, source_id, start_at, end_at, status, event_type, time_range (start_at, end_at)
- `event_participants`: event_id, person_id
- `event_relations`: event_id, (entity_type, entity_id)
- `deadline_events`: due_at, status
- `focus_blocks`: (start_at, end_at)
