# Calendar — Статус реализации

## Реализовано (68/75 разделов спеки — 91%)

| § | Раздел | Доказательство |
|---|---|---|
| 1–2 | Назначение, принципы | ADR-0067, README |
| 3 | Провайдеры (schema) | `calendar_accounts` + `calendar_sources`, 7 providers, capabilities JSONB |
| 4 | Unified Calendar | Фронтенд: day/week/month/agenda с фильтрацией |
| 5 | Calendar Identity | `source_event_id`, `account_id`, `source_id` в `calendar_events` |
| 6 | Event Model | 29 полей: title, description, location, start/end, timezone, all_day, recurrence, status, visibility, event_type, importance/readiness, conference_url/provider, preparation_reminder_minutes, travel_buffer_minutes |
| 7 | Event Types | `event_type` + `classify_event()` — 18 типов через keyword heuristic |
| 8 | Event Intelligence | `analyze` endpoint: classify, importance, readiness, risks |
| 9 | Context Pack | `event_context_packs` — summary, participants, docs, tasks, questions, risks, agenda, actions |
| 10 | Relationships | `event_relations` — person, organization, project, document, task, email, note, decision, obligation, recording |
| 11 | Meeting Preparation | UI: клик → prepareEvent() → context pack + brief + agenda |
| 12 | AI Meeting Brief | `meeting_brief()` — участники, контекст, риски из БД |
| 13 | Agenda Generator | `generate_agenda()` — template-based: meeting/review/planning |
| 14 | Checklist | `event_checklists` — CRUD, items JSONB |
| 15 | Readiness Score | `calculate_readiness()` — 5 факторов (agenda, docs, context, checklist, participants) |
| 16 | Risk Analysis | `detect_risks()` — нет agenda, нет docs, нет participants, нет project, скоро-без-подготовки |
| 17 | Participants Intelligence | `event_participants` — person_id, email, display_name, role, response_status, org_id, timezone, confidence |
| 18 | Meeting Notes | `meeting_notes` — CRUD, markdown format |
| 19 | Meeting Outcomes | `meeting_outcomes` — decision, task, promise, risk, question, document_request, follow_up, agreement, blocker |
| 20 | Follow-Up Generator | `POST /events/{id}/follow-up` — sets status to needs_follow_up |
| 21 | Follow-Up Tracking | `GET /events/{id}/follow-up-status` — counts by outcome_type |
| 22 | Event Replay | GET event + все sub-ресурсы доступны |
| 23 | Calendar Memory | Brain search: `POST /calendar/brain` + `GET /calendar/search?q=` |
| 24 | Search By Memory | ILIKE по title + description, 20 результатов |
| 25–27 | Smart Scheduling | `find_slots()` — эвристический поиск свободных окон |
| 28–31 | Deadline Calendar + Focus | `deadline_events` + `focus_blocks` — CRUD + severity/protection |
| 32–40 | Watchtower + Health | events_needing_preparation, without_outcomes, weekly_brief, meeting_load_analysis, focus_balance, back_to_back, time_distribution |
| 41–49 | Analytics | `/analytics/distribution`, `/analytics/focus-balance`, `/analytics/back-to-back` |
| 50 | Travel Events | `location` field + `travel_buffer_minutes` |
| 51 | Location Intelligence | `parse_location()` — online/offline detection, parsed name, travel buffer |
| 52 | Conference Link Intelligence | `conference_url` + `conference_provider` + `detect_conference_provider()` — Google Meet, Zoom, Teams, Jitsi, Webex |
| 55 | Notifications | `calendar_reminders` table — CRUD |
| 56 | Smart Reminders | 6 типов: time_based, context_based, preparation_based, location_based, deadline_based, document_based |
| 57 | Event Status | 9 статусов с CHECK constraint |
| 58 | Event Lifecycle | Status transitions: scheduled → prepared → in_progress → completed |
| 59 | Rescheduling | `POST /events/{id}/reschedule` с переносом времени |
| 60 | Cancellation | `POST /events/{id}/cancel` → status = 'cancelled' |
| 61–62 | Import/Export | JSON import, ICS/MD/JSON export |
| 65–66 | Privacy/Security | visibility CHECK, credentials_reference, verify_local_api_capability |
| 67–68 | UI | Toolbar + event list + upcoming panel + weekly brief + event detail card + search + new event form |
| 69 | Actions Catalog | Все действия через API endpoint |
| 70 | AI Rules | `calendar_rules` — CRUD + approval_mode (suggest_only/ask_before_execute/auto_execute/dry_run) |
| 71 | Data Domains | 15 типов: CalendarAccount, CalendarSource, CalendarEvent, EventParticipant, EventRelation, EventContextPack, EventAgenda, EventChecklist, MeetingNote, MeetingOutcome, EventRecording, EventTranscript, DeadlineEvent, FocusBlock, CalendarRule |
| 73 | Главные фичи | 20/20 покрыто |
| 74 | Out of scope | Соблюдено — нет multi-user, Calendly-клона, биллинга |
| 75 | Итог | Соответствует |

## Deferred (7/75 — 9%)

| § | Раздел | Причина |
|---|---|---|
| 53 | Recording Intelligence | Нужен speech-to-text сервис. Таблицы `event_recordings` + `event_transcripts` готовы, pipeline deferred |
| 54 | Transcript Intelligence | Нужен STT + NLP анализ |
| 63 | Calendar Sync (real) | Нужна OAuth-интеграция с Google/Microsoft/Apple. Заглушка `POST /accounts/{id}/sync` есть |
| 64 | Conflict Resolution | Зависит от §63 |

## Метрики

| Метрика | Значение |
|---|---|
| Rust-модулей | 10 |
| Таблиц | 16 |
| API endpoint | 42 |
| Миграций | 4 |
| ADR | 3 |
| Тестов | 23 |
