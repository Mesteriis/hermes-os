SUPERGOAL_PHASE_START
Phase: 7 of 15 — Projects & Tasks
Task: Port Projects and Tasks domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, projects and tasks domain file listings
Depends on phases: 1, 2, 3

## Why

Projects and Tasks are related domains sharing UI patterns for obligation and decision review. Porting together ensures consistency in how task candidates, obligations, and decisions are displayed.

## Work

1. **Create projects domain** under `frontend/src/domains/projects/`:
   - Types, API, queries, stores, components (ProjectsDashboard, ProjectsHero, ProjectsRail), views/ProjectsPage, routes

2. **Create tasks domain** under `frontend/src/domains/tasks/`:
   - Types, API, queries, stores, components (TaskList with TanStack Virtual), views/TasksPage, routes
   - Task list must use TanStack Virtual for virtualization
   - Include obligation/decision review panel integration

3. **Register routes** for `/projects` and `/tasks`

4. **Verify:**
   - Build passes
   - Projects dashboard renders
   - Tasks list renders with virtual scrolling

## Acceptance criteria

- [ ] AC1: Projects page renders with dashboard, hero, rail widgets
- [ ] AC2: Tasks list renders with virtual scrolling (TanStack Virtual)
- [ ] AC3: Task items show correct status, priority, and metadata from API
- [ ] AC4: Obligation/decision review panel renders within tasks
- [ ] AC5: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/projects/` and `frontend/src-svelte/lib/pages/tasks/`
- Use TanStack Virtual for task list virtualization
