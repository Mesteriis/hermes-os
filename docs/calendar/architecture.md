# Calendar — Архитектура

## Модули

| Модуль | Назначение |
|---|---|
| `calendar.rs` | Ядро: CalendarAccount, CalendarSource, CalendarEvent + stores |
| `calendar_core.rs` | EventParticipant, EventRelation, EventContextPack, EventAgenda, EventChecklist |
| `calendar_intelligence.rs` | Эвристики: classify, importance, readiness, risks, fingerprint |
| `calendar_meetings.rs` | MeetingNote, MeetingOutcome, EventRecording, EventTranscript |
| `calendar_scheduling.rs` | DeadlineEvent, FocusBlock, SmartSchedulingService |
| `calendar_health.rs` | CalendarWatchtowerService: preparation gaps, missing outcomes, load analysis |
| `calendar_brain.rs` | CalendarBrainService: answer, search, meeting brief, agenda generation |
| `calendar_rules.rs` | CalendarRule + CRUD |
| `calendar_sync.rs` | ICS/Markdown export, import stub |

## Слои

```
API (`app::router` + `domains::calendar::handlers`)
  ↓
Domain services (calendar_intelligence, calendar_brain, calendar_health)
  ↓
Stores (PgPool-backed, one per entity)
  ↓
PostgreSQL (15 tables, 3 migrations)
```

## Паттерны

- **Store pattern**: каждый store принимает `PgPool::new(pool)`, все методы возвращают `Result<T, DomainError>`
- **Error handling**: `#[derive(Error)]` enum → `From<X> for ApiError` в lib.rs
- **API auth**: router-level `x-hermes-secret`; audit actor is the constant `hermes-frontend`
- **Heuristic-first intelligence**: все AI-фичи работают без Ollama
- **No uuid crate**: ID генерируются как `{prefix}:v1:{timestamp_nanos_hex}`
