SUPERGOAL_PHASE_START
Phase: 9 of 15 — Documents & Notes
Task: Port Documents and Notes domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, documents and notes domain file listings
Depends on phases: 1, 2, 3

## Why

Documents and Notes are content-focused domains with similar UI patterns (lists with source filters, insights panels, virtual scrolling). Porting together validates pattern reuse.

## Work

1. **Create documents domain** under `frontend/src/domains/documents/`:
   - Types, API, queries (TanStack Query hooks), stores (Pinia for UI state only)
   - Views/DocumentsPage.vue — main page
   - Components: DocumentsList (with TanStack Virtual), DocumentsNavigation, DocumentsSourceCards, DocumentsInsights, DocumentsProcessingJobs
   - Routes

2. **Create notes domain** under `frontend/src/domains/notes/`:
   - Types, API, queries, stores
   - Views/NotesPage.vue — main page
   - Components: NotesList (with TanStack Virtual), NotesSourceFilters, NotesInsights
   - Routes

3. **Register routes** for `/documents` and `/notes`

4. **Verify:**
   - Build passes
   - Documents and Notes pages render with data

## Acceptance criteria

- [ ] AC1: Documents page renders with list, navigation, source cards, insights
- [ ] AC2: Documents list uses TanStack Virtual for scrolling
- [ ] AC3: Notes page renders with list, source filters, insights
- [ ] AC4: Notes list uses TanStack Virtual for scrolling
- [ ] AC5: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/documents/` and `frontend/src-svelte/lib/pages/notes/`
- Both domains use TanStack Virtual for list virtualization
