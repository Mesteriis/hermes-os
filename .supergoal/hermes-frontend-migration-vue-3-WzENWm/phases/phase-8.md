SUPERGOAL_PHASE_START
Phase: 8 of 15 — Calendar Domain
Task: Port Calendar domain page to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 4
Evidence required: build output, calendar domain file listing
Depends on phases: 1, 2, 3

## Why

Calendar is a self-contained domain with events from multiple providers. Porting it validates the domain pattern for date/time-intensive views.

## Work

1. **Create calendar domain** under `frontend/src/domains/calendar/`:
   - Types, API, queries (useCalendarEventsQuery), stores, components, views/CalendarPage, routes
   - Calendar event display with date formatting via date-fns

2. **Register route** for `/calendar`

3. **Verify:**
   - Build passes
   - Calendar renders with events

## Acceptance criteria

- [ ] AC1: Calendar page renders with events from API
- [ ] AC2: Events show correct date/time info formatted via date-fns
- [ ] AC3: Empty state renders when no events
- [ ] AC4: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/calendar/` and Svelte CalendarPage
- Use date-fns for all date formatting
