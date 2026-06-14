# Roadmap: Hermes Frontend Migration to Vue 3

**Task:** Complete migration of Hermes Hub frontend from SvelteKit 2 + Svelte 5 to Vue 3 + TypeScript + Vite + Tauri 2
**Type:** brownfield, refactor, ui
**Created:** 2026-06-14
**Total phases:** 15

## Context summary

- **Stack:** Vue 3 + TypeScript + Vite + Tauri 2 + Pinia + TanStack Query/Table/Virtual + Tailwind CSS + shadcn-vue + TipTap + Vue Flow + date-fns + Motion
- **Package manager:** pnpm
- **Build / test / lint commands:** `pnpm build`, `pnpm lint` (lint:styles + lint:ts via svelte-check → will become vue-tsc + eslint), `pnpm test:unit` (vitest)
- **Risky areas:** Visual identity preservation, 16-domain migration scale, Tauri coexistence during migration, complex state boundary decomposition

## Assumptions

Non-blocking decisions recorded here so we can proceed without round-trips. If any are wrong, stop the run and tell us:

- The new Vue app will be created in `frontend/` directory. The existing SvelteKit code will be moved to `frontend/src-svelte/` during migration to preserve it until cutover.
- Tauri config (`frontend/src-tauri/tauri.conf.json`) will be updated ONLY in Phase 15 (Cutover). During Phases 1-14, the Vue app runs on its own Vite dev server.
- Migration order: Foundation → Shell → Shared UI → Settings → Home → Personas+Organizations → Projects+Tasks → Calendar → Documents+Notes → Knowledge+Review → Agents+Timeline → Communications → Telegram+WhatsApp → Polish → Cutover.
- The existing SvelteKit `frontend/src/lib/api/` types and `frontend/src/lib/services/` business logic will be referenced during porting but not directly reused — Vue domains define their own types/API/queries/stores per DDD.
- i18n dictionaries (`ru.json`, `en.json`) will be ported to the new structure without semantic changes.
- Theme tokens from `frontend/src/lib/styles/tokens.css` will be ported to Tailwind CSS config.
- Testing: existing vitest tests cover services; new Vue integration tests will use vue-test-utils + vitest.

## Risk top 3

1. **R2: Visual identity preservation** — likelihood: HIGH, mitigation: Port theme tokens to Tailwind config first; use CSS custom properties as bridge; visual comparison at Polish phase.
2. **R1: Massive scope (16 domains)** — likelihood: HIGH, mitigation: Phase per domain group; each independently verifiable; start with simplest (Settings, Home) to build momentum.
3. **R3: Tauri coexistence** — likelihood: MEDIUM, mitigation: Update Tauri config only at Cutover; separate dev servers during migration.

## Phase map

| # | Phase | Depends on | Deliverable |
|---|-------|------------|-------------|
| 1 | Foundation | — | Vue project scaffold, all dependencies, platform layer, i18n, theme config |
| 2 | App Shell | 1 | Sidebar, Topbar, workspace layout, notifications, layout editor |
| 3 | Shared UI Primitives | 1 | Button, Input, Dialog, Dropdown, etc. via shadcn-vue |
| 4 | Settings Domain | 1, 2, 3 | Full settings page with all panels |
| 5 | Home Dashboard | 1, 2, 3 | Home page with all widgets |
| 6 | Personas & Organizations | 1, 2, 3 | Persons + Organizations pages |
| 7 | Projects & Tasks | 1, 2, 3 | Projects + Tasks pages |
| 8 | Calendar Domain | 1, 2, 3 | Calendar page with events |
| 9 | Documents & Notes | 1, 2, 3 | Documents + Notes pages |
| 10 | Knowledge & Review | 1, 2, 3 | Knowledge graph + Review polygraph pages |
| 11 | Agents & Timeline | 1, 2, 3 | Agents + Timeline pages |
| 12 | Communications/Mail | 1, 2, 3 | Mail list, viewer, compose, draft strip, health strip, account wizard |
| 13 | Telegram & WhatsApp | 1, 2, 3 | Telegram + WhatsApp messaging pages |
| 14 | Polish & Harden | 1..13 | Error states, animations, a11y, perf, edge cases |
| 15 | Cutover | 14 | SvelteKit removal, Tauri config update, full validation |

---

## Phase 1 — Foundation

**Why:** All subsequent phases depend on the Vue project scaffold, build configuration, platform layer, i18n system, and theme tokens.

**Deliverables:**
- `frontend/` directory restructuring (move SvelteKit src/ to src-svelte/)
- `frontend/package.json`, `frontend/pnpm-lock.yaml` with Vue 3 + all target dependencies
- `frontend/vite.config.ts` configured for Vue 3 + Tauri 2
- `frontend/tsconfig.json` with strict TypeScript
- `frontend/tailwind.config.ts` with Hermes theme tokens
- `frontend/src/platform/api-client.ts` — ported ApiClient with X-Hermes-Secret
- `frontend/src/platform/sse-client.ts` — SSE client foundation
- `frontend/src/platform/i18n/` — i18n system (ru.json, en.json, composable)
- `frontend/tailwind.config.ts` — theme tokens from tokens.css
- `frontend/src/app/App.vue` — minimal Vue app that renders
- `frontend/index.html` — entry HTML

**Acceptance criteria:**
- [ ] `pnpm build` exits 0 with no errors
- [ ] `pnpm lint` passes (or has baseline lint config)
- [ ] Vue app renders blank page at `http://localhost:5173`
- [ ] i18n system loads ru.json and renders Russian text
- [ ] ApiClient sends X-Hermes-Secret header correctly
- [ ] Tailwind theme matches Hermes color tokens (check primary, surface, text colors)
- [ ] TypeScript strict mode enabled (no `strict: false`)
- [ ] No `.svelte` or `.svelte-kit` files in new `src/` directory
- [ ] Old SvelteKit code exists in `frontend/src-svelte/` and is ignored by new build

**Mandatory commands:**
- `cd frontend && pnpm install && pnpm build`
- `cd frontend && pnpm lint` (once configured)

**Evidence required:**
- `pnpm build` output last 10 lines
- File listing of new `frontend/src/` directory
- File listing confirming `frontend/src-svelte/` exists with old code
- Tailwind config showing Hermes theme tokens

**Dependencies:** none

---

## Phase 2 — App Shell

**Why:** The app shell (sidebar, topbar, workspace layout) is the container for all domain views. Must be ported before any domain can render.

**Deliverables:**
- `frontend/src/app/shell/AppShell.vue` — main layout component
- `frontend/src/app/shell/Sidebar.vue` — navigation sidebar
- `frontend/src/app/shell/Topbar.vue` — top bar with notifications, user menu
- `frontend/src/app/shell/NotificationsDrawer.vue` — notification panel
- `frontend/src/app/shell/LayoutEditor.vue` — layout editing controls
- `frontend/src/app/router.ts` — Vue Router config with all route definitions
- `frontend/src/app/App.vue` — updated with router-view and shell
- `frontend/src/shared/stores/navigation.ts` — Pinia store for navigation state
- `frontend/src/shared/stores/theme.ts` — Pinia store for theme state
- `frontend/src/shared/stores/sidebar.ts` — Pinia store for sidebar state
- `frontend/src/shared/stores/notifications.ts` — Pinia store for notifications
- `frontend/src/shared/stores/layoutEditor.ts` — Pinia store for layout editing

**Acceptance criteria:**
- [ ] AppShell renders sidebar, topbar, and workspace area
- [ ] Sidebar navigation switches between placeholder route views
- [ ] Topbar shows view title and notification count
- [ ] Notifications drawer opens/closes
- [ ] User menu opens/closes
- [ ] Layout editing mode can be toggled
- [ ] All CSS/styling matches existing Hermes visual identity
- [ ] `pnpm build` passes
- [ ] No Svelte code in new src/

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Screenshot of shell with sidebar and topbar
- Build output last 10 lines
- Route config showing all defined routes

**Dependencies:** Phase 1

---

## Phase 3 — Shared UI Primitives

**Why:** UI primitives are used by all domains. Porting them early ensures consistency and avoids duplication.

**Deliverables:**
- `frontend/src/shared/ui/` — shadcn-vue components initialized as project-owned code:
  - Button, Input, Dialog, Dropdown, Select, Switch, Tabs, Card, Badge, Avatar, Tooltip, Popover, Command (palette), Sheet (drawer), Separator, ScrollArea, Skeleton, Progress, Toast
- `frontend/src/shared/ui/index.ts` — barrel exports

**Acceptance criteria:**
- [ ] All shadcn-vue components render correctly in isolation
- [ ] Components use Hermes theme tokens (colors, spacing, typography)
- [ ] Button supports variants (default, secondary, outline, ghost, destructive)
- [ ] Dialog opens/closes with animation
- [ ] Dropdown menu opens on click
- [ ] Tooltip shows on hover
- [ ] Toast notification shows and auto-dismisses
- [ ] `pnpm build` passes
- [ ] Components match existing Hermes visual style (not generic shadcn defaults)
- [ ] TypeScript strict — no `any` without written explanation

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output
- List of files in `shared/ui/`

**Dependencies:** Phase 1

---

## Phase 4 — Settings Domain

**Why:** Settings is the simplest domain and serves as the first real domain port to validate the architecture pattern.

**Deliverables:**
- `frontend/src/domains/settings/types/` — settings types
- `frontend/src/domains/settings/api/` — settings API functions
- `frontend/src/domains/settings/queries/` — TanStack Query hooks
- `frontend/src/domains/settings/stores/` — Pinia stores (UI state only)
- `frontend/src/domains/settings/views/` — settings page views
- `frontend/src/domains/settings/components/` — settings-specific components
- `frontend/src/domains/settings/routes/` — settings route definitions
- All settings panels: Appearance, Language, Integrations, Sidebar, Application, AI Settings

**Acceptance criteria:**
- [ ] All settings panels render with correct data
- [ ] Theme toggle (light/dark) works and is persisted
- [ ] Language switch (en/ru) works
- [ ] Integration settings show connected accounts
- [ ] AI settings panels render
- [ ] Settings changes are saved via API
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes settings

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output
- List of settings domain files

**Dependencies:** Phase 1, 2, 3

---

## Phase 5 — Home Dashboard

**Why:** Home dashboard is the landing view with multiple widgets (metrics, people, projects, priorities, upcoming, system status, what's new).

**Deliverables:**
- `frontend/src/domains/home/types/` — home types
- `frontend/src/domains/home/api/` — home API functions
- `frontend/src/domains/home/queries/` — home query hooks
- `frontend/src/domains/home/views/HomePage.vue` — main home page
- `frontend/src/domains/home/components/` — home widget components
- Widget components: ActiveProjects, Metrics, PeopleTalked, Priorities, SystemStatus, Upcoming, WhatsNew

**Acceptance criteria:**
- [ ] Home page renders all widgets in correct layout
- [ ] Widgets show real data from API (not mock data)
- [ ] Widget layout responds to workspace resizing
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes home

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 6 — Personas & Organizations

**Why:** Personas (people) and Organizations are related entity domains that share patterns. Porting together reduces overhead.

**Deliverables:**
- `frontend/src/domains/personas/` — full domain structure
- `frontend/src/domains/organizations/` — full domain structure
- Personas: list, detail view, identity review, relationship review, identity trace review
- Organizations: page with widgets

**Acceptance criteria:**
- [ ] Personas list renders with data from API
- [ ] Persona detail view shows identity, relationships, communication history
- [ ] Identity review panel renders
- [ ] Relationship review panel renders
- [ ] Organizations page renders with dashboard, hero, rail widgets
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 7 — Projects & Tasks

**Why:** Projects and Tasks are related project/task domains. Porting together ensures consistency in how task candidates, obligations, and decisions are displayed.

**Deliverables:**
- `frontend/src/domains/projects/` — full domain structure
- `frontend/src/domains/tasks/` — full domain structure
- Projects: dashboard, hero, rail widgets
- Tasks: task list with virtualization (TanStack Virtual)

**Acceptance criteria:**
- [ ] Projects page renders with dashboard, hero, rail
- [ ] Tasks list renders with virtual scrolling
- [ ] Task items show correct status, priority, and metadata
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 8 — Calendar Domain

**Why:** Calendar is a self-contained domain with events from multiple providers.

**Deliverables:**
- `frontend/src/domains/calendar/` — full domain structure
- Calendar page with event display

**Acceptance criteria:**
- [ ] Calendar page renders with events
- [ ] Events show correct date/time info
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 9 — Documents & Notes

**Why:** Documents and Notes are content-focused domains with similar UI patterns (lists, source filters, insights).

**Deliverables:**
- `frontend/src/domains/documents/` — full domain structure
- `frontend/src/domains/notes/` — full domain structure
- Documents: list, navigation, source cards, insights, processing jobs
- Notes: list, source filters, insights

**Acceptance criteria:**
- [ ] Documents page renders with list, navigation, source cards
- [ ] Notes page renders with list, source filters, insights
- [ ] Virtual scrolling for document/note lists
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 10 — Knowledge & Review

**Why:** Knowledge graph and Review (polygraph/contradictions) are intelligence-focused domains. Knowledge uses Vue Flow for graph visualization.

**Deliverables:**
- `frontend/src/domains/knowledge/` — full domain structure
- `frontend/src/domains/review/` — full domain structure
- Knowledge: graph canvas (Vue Flow), node inspector, polygraph review
- Review: obligations and decisions review panel

**Acceptance criteria:**
- [ ] Knowledge graph canvas renders with Vue Flow
- [ ] Node inspector shows selected node details
- [ ] Polygraph review lists contradiction observations
- [ ] Review page lists obligations and decisions
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 11 — Agents & Timeline

**Why:** Agents and Timeline are the smaller remaining domain views.

**Deliverables:**
- `frontend/src/domains/agents/` — full domain structure
- `frontend/src/domains/timeline/` — full domain structure
- Agents: detail, grid, rail, runtime metrics, workflows
- Timeline: stream, filters

**Acceptance criteria:**
- [ ] Agents page renders with grid, detail, rail, metrics, workflows
- [ ] Timeline page renders with stream and filters
- [ ] Virtual scrolling for timeline
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 12 — Communications/Mail Domain

**Why:** Communications (mail) is the most complex domain. It involves mail list, message viewer, compose, draft strip, health strip, account wizard, and context inspector. Last domain ported before messaging.

**Deliverables:**
- `frontend/src/domains/communications/` — full domain structure
- Mail list with virtualization (TanStack Virtual)
- Message viewer with tabs (body, headers, attachments, related, timeline)
- Context inspector and context rail
- Conversation list
- Compose drawer (compose/reply/forward)
- Draft strip
- Health strip
- Account setup wizard modal

**Acceptance criteria:**
- [ ] Mail list renders with virtual scrolling from API
- [ ] Message viewer renders HTML content in sandboxed iframe
- [ ] Compose drawer supports compose/reply/forward modes
- [ ] Draft strip shows and manages drafts
- [ ] Account wizard modal renders provider selection flow
- [ ] Health strip shows mailbox health status
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes communications

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 13 — Telegram & WhatsApp

**Why:** Telegram and WhatsApp are messaging sub-sections of the Communications view. They share message thread UI patterns.

**Deliverables:**
- `frontend/src/domains/telegram/` — full domain structure
- `frontend/src/domains/whatsapp/` — full domain structure
- Telegram: chat list (virtualized), message thread, rail, command header, action rail, status messages
- WhatsApp: session list, message thread, rail

**Acceptance criteria:**
- [ ] Telegram chat list renders with virtual scrolling
- [ ] Telegram message thread renders messages
- [ ] Telegram rail shows chat details
- [ ] WhatsApp session list renders
- [ ] WhatsApp message thread renders
- [ ] `pnpm build` passes
- [ ] Visual style matches existing Hermes

**Mandatory commands:**
- `cd frontend && pnpm build`

**Evidence required:**
- Build output

**Dependencies:** Phase 1, 2, 3

---

## Phase 14 — Polish & Harden

**Why:** Catch what earlier phases missed because they were focused on shipping behavior. This is how "every aspect is perfect" gets enforced.

**Sub-passes (each must produce evidence):**

- [ ] **UX & copy** — every visible string reads well, no debug placeholders
- [ ] **States** — empty, loading, error, unauthorized verified for every domain surface
- [ ] **Edges** — empty inputs, long inputs, special chars, slow network
- [ ] **Security** — input validation, X-Hermes-Secret in all API calls, no secrets in bundle
- [ ] **A11y** — keyboard nav, focus management, screen reader, contrast ≥ AA
- [ ] **Perf** — virtual scrolling verified for all large collections, bundle size analysis, no N+1
- [ ] **Diff review** — review for stray debug logs, TODOs from this run, unused imports
- [ ] **Regression sweep** — full build + visual comparison with existing Hermes app
- [ ] **Animation** — workspace transitions, panel animations, micro-interactions smooth via Motion

**Mandatory commands:**
- `cd frontend && pnpm build`
- `cd frontend && pnpm lint` (if configured)

**Evidence required:**
- One paragraph per sub-pass with what was checked and what was found/fixed
- Bundle size analysis
- Final screenshots of key surfaces

**Dependencies:** Phase 1..13

---

## Phase 15 — Cutover

**Why:** Final phase — remove all SvelteKit code, update Tauri configuration, and run full validation gate to confirm the migration is complete.

**Deliverables:**
- `frontend/src-svelte/` removed entirely
- `frontend/src-tauri/tauri.conf.json` updated for Vue build output (verify already correct)
- `frontend/package.json` scripts updated (remove svelte-related scripts)
- `frontend/svelte.config.js` removed
- CI/CD configuration updated if needed
- Makefile targets updated if needed
- Full validation gate passed

**Acceptance criteria:**
- [ ] No `.svelte` files exist anywhere in `frontend/` (except node_modules)
- [ ] No `svelte` or `@sveltejs/*` dependencies in `frontend/package.json`
- [ ] `pnpm build` exits 0
- [ ] `pnpm lint` passes (vue-tsc + eslint configured)
- [ ] `pnpm test:unit` passes
- [ ] Tauri `frontendDist` points to correct Vue build output
- [ ] Full `make validate` passes from repo root
- [ ] `frontend/src-svelte/` directory no longer exists

**Mandatory commands:**
- `cd frontend && pnpm build`
- `cd frontend && pnpm lint`
- `cd frontend && pnpm test:unit`
- `make validate` (from repo root)

**Evidence required:**
- Build output last 10 lines
- Test output
- File listing confirming no .svelte files
- package.json confirming no svelte dependencies
- tauri.conf.json showing correct frontendDist

**Dependencies:** Phase 14
