SUPERGOAL_PHASE_START
Phase: 6 of 15 — Personas & Organizations
Task: Port Personas (people) and Organizations domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 6
Evidence required: build output, personas and organizations domain file listings
Depends on phases: 1, 2, 3

## Why

Personas and Organizations are related entity domains that share UI patterns (list + detail, identity review, relationship review). Porting together reduces overhead and reinforces the domain-driven pattern.

## Work

1. **Create personas domain** under `frontend/src/domains/personas/`:
   - `types/persona.ts` — Persona types (PersonaType, Identity, Relationship, etc.)
   - `api/personas.ts` — API functions for person CRUD, identity review, relationship review
   - `queries/usePersonasQuery.ts` — TanStack Query hooks
   - `stores/personas.ts` — Pinia store for UI state (selected persona, active tab)
   - `components/` — PersonsList, PersonsDetail, PersonsIdentityReview, PersonsRelationshipReview, PersonsIdentityTraceReview
   - `views/PersonsPage.vue` — main persons page with widget layout
   - `routes/index.ts`

2. **Create organizations domain** under `frontend/src/domains/organizations/`:
   - Same structure as personas
   - Components: OrganizationsDashboard, OrganizationsHero, OrganizationsRail

3. **Register routes** for `/persons` and `/organizations`

4. **Verify:**
   - Build passes
   - Personas list renders with data
   - Detail view shows identity, relationships, communication history
   - Organizations page renders

## Acceptance criteria

- [ ] AC1: Personas list renders with virtual scrolling from API data
- [ ] AC2: Persona detail view shows identity, relationships, and communication
- [ ] AC3: Identity review panel renders with reviewable data
- [ ] AC4: Relationship review panel renders with suggestions
- [ ] AC5: Organizations page renders with dashboard, hero, rail widgets
- [ ] AC6: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output
- List of personas domain files
- List of organizations domain files

## Notes

- Reference `frontend/src-svelte/lib/pages/persons/` and `frontend/src-svelte/lib/pages/organizations/`
- Personas terminology per ADR-0084 (not "contacts")
- API endpoints: study `frontend/src-svelte/lib/api/endpoints/persons.ts` and organizations
- Persona types: PersonaType = 'human' | 'ai_agent' | 'organization_proxy' | 'system'
