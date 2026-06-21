SUPERGOAL_PHASE_START
Phase: 5 of 15 — Home Dashboard
Task: Port the Home dashboard page with all widgets to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, home domain file listing
Depends on phases: 1, 2, 3

## Why

Home dashboard is the default landing view. It validates the widget-based workspace pattern and serves as reference for other domain views.

## Work

1. **Create home domain structure** under `frontend/src/domains/home/`:
   - `types/`, `api/`, `queries/`, `stores/`, `components/`, `views/`, `routes/`

2. **Port HomePage** and all widget components:
   - HomeActiveProjects — active projects list
   - HomeMetrics — key metrics display
   - HomePeopleTalked — recent contacts
   - HomePriorities — priority items
   - HomeSystemStatus — system status indicators
   - HomeUpcoming — upcoming events/tasks
   - HomeWhatsNew — what's new feed

3. **Register route** for `/home`

4. **Verify:**
   - Build passes
   - Home page renders all widgets with real API data

## Acceptance criteria

- [ ] AC1: Home page renders all 7 widget types in correct layout
- [ ] AC2: Widgets show real data from API (not mock data)
- [ ] AC3: Widget layout responds to workspace resizing
- [ ] AC4: `cd frontend && pnpm build` exits 0
- [ ] AC5: Empty state renders when no data is available

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/home/`
- Widget components use TanStack Query for data
- Keep each component under 500 lines
