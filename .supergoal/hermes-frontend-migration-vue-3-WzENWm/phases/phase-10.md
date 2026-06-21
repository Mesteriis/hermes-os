SUPERGOAL_PHASE_START
Phase: 10 of 15 — Knowledge & Review
Task: Port Knowledge graph and Review (polygraph) domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 6
Evidence required: build output, knowledge and review domain file listings
Depends on phases: 1, 2, 3

## Why

Knowledge graph (using Vue Flow) and Review (polygraph/contradictions) are intelligence-focused domains. Knowledge graph validates Vue Flow integration for graph visualization.

## Work

1. **Create knowledge domain** under `frontend/src/domains/knowledge/`:
   - Types, API (graph, contradictions), queries (useGraphQuery, useContradictionsQuery)
   - Stores (Pinia for UI state: selected node, active tab)
   - Components:
     - KnowledgeGraphCanvas.vue — Vue Flow graph visualization
     - KnowledgeNodeInspector.vue — node detail panel
     - KnowledgePolygraphReview.vue — contradiction observations list
   - Views/KnowledgePage.vue — main page
   - Routes

2. **Create review domain** under `frontend/src/domains/review/`:
   - Types, API, queries (useObligationsQuery, useDecisionsQuery)
   - Components: ReviewObligations, ReviewDecisions
   - Views/ReviewPage.vue — main page
   - Routes

3. **Register routes** for `/knowledge` and `/review`

4. **Verify:**
   - Build passes
   - Knowledge graph canvas renders with Vue Flow
   - Review page lists obligations and decisions

## Acceptance criteria

- [ ] AC1: Knowledge graph canvas renders using Vue Flow
- [ ] AC2: Node inspector shows selected node details
- [ ] AC3: Polygraph review lists contradiction observations from API
- [ ] AC4: Review page lists obligations and decisions with status
- [ ] AC5: Graph nodes/edges render with correct styling
- [ ] AC6: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/knowledge/` and `frontend/src-svelte/lib/pages/review/`
- Vue Flow replaces the Svelte-based graph implementation
- Polygraph is the user-facing name for the Consistency/Contradiction Engine per ADR-0085
- Keep graph canvas component under 500 lines
