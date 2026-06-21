SUPERGOAL_PHASE_START
Phase: 11 of 15 — Agents & Timeline
Task: Port Agents and Timeline domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, agents and timeline domain file listings
Depends on phases: 1, 2, 3

## Why

Agents and Timeline are the remaining smaller domain views. Agents shows AI runtime status; Timeline shows activity stream. Validates TanStack Virtual for timeline items.

## Work

1. **Create agents domain** under `frontend/src/domains/agents/`:
   - Types, API, queries, stores (Pinia for UI state)
   - Components: AgentsDetail, AgentsGrid, AgentsRail, AgentsRuntimeMetrics, AgentsWorkflows
   - Views/AgentsPage.vue
   - Routes

2. **Create timeline domain** under `frontend/src/domains/timeline/`:
   - Types, API, queries, stores
   - Components: TimelineStream (with TanStack Virtual), TimelineFilters
   - Views/TimelinePage.vue
   - Routes

3. **Register routes** for `/agents` and `/timeline`

4. **Verify:**
   - Build passes
   - Agents page renders
   - Timeline renders with virtual scrolling

## Acceptance criteria

- [ ] AC1: Agents page renders with grid, detail, rail, metrics, workflows
- [ ] AC2: Timeline page renders with stream and filters
- [ ] AC3: Timeline items use TanStack Virtual for virtualization
- [ ] AC4: Timeline filters affect displayed items
- [ ] AC5: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/agents/` and `frontend/src-svelte/lib/pages/timeline/`
- Use TanStack Virtual for timeline stream
