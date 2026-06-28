# Calendar And Events Architecture

## Position

Calendar owns scheduled event records and calendar source identity. It does not
own the global Timeline Engine.

## Modules

Paths below refer to `backend/src/domains/calendar/`.

| Module | Responsibility |
|---|---|
| `core.rs` | CalendarAccount, CalendarSource, CalendarEvent and stores |
| `events.rs` | participants, relations, context packs, agendas, checklists |
| `intelligence.rs` | engine-facing classification and readiness helpers |
| `meetings.rs` | meeting notes, outcomes, recordings and transcripts |
| `scheduling.rs` | deadlines, focus blocks and scheduling support |
| `health.rs` | Risk Engine/attention signals for calendar context |
| `brain.rs` | context answers and briefs over shared engines |
| `rules.rs` | calendar rules |
| `sync.rs` | import/export helpers |
| `reminders.rs` | reminder records and toggles |

## Layers

```text
API
  -> Calendar domain services
  -> Stores
  -> PostgreSQL
  -> Shared engines for context, timeline, risk and search
```

## Patterns

- Store pattern: stores receive a pool and return domain errors.
- Event-backed changes.
- Provider identity remains at the provider/source boundary.
- Heuristic-first intelligence should work without Ollama.
