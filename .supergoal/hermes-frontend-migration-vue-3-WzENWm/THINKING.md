# THINKING.md — Hermes Frontend Vue 3 Migration

## Goals

1. Complete migration from SvelteKit 2 + Svelte 5 to Vue 3 + TypeScript + Vite
2. Zero remaining Svelte/SvelteKit code at completion
3. Domain-Driven Frontend architecture per ADR-0093
4. Preserve all visual identity (colors, typography, spacing, surfaces, navigation)
5. Introduce Query Layer (TanStack Query), proper state boundaries (Pinia for UI only)
6. Real-time first approach (SSE/WebSocket instead of polling)
7. Virtualization for all large collections
8. No god files (>500 lines), no god components
9. Full parity with existing functionality (16 domains/views)

## Constraints

- **ADR-0093**: Vue 3 + TypeScript + Vite + Tauri 2. Pinia for UI state, TanStack Query for server state. Domain-Driven structure.
- **ADR-0004**: Tauri 2 desktop shell. `frontendDist` in tauri.conf.json points to build output. `beforeDevCommand` runs pnpm dev.
- **ADR-0026**: Desktop-first responsive UI (800x600 minimum).
- **ADR-0031**: No mobile UI design or validation.
- **ADR-0056**: X-Hermes-Secret header for API auth. Actor = "hermes-frontend".
- **ADR-0077**: i18n with ru.json and en.json dictionaries. English strings = translation keys.
- **AGENTS.md**: TDD preferred, no broad rewrites without validation, must run configured lint/test.
- **User spec**: Visual identity must be preserved. No redesign. Same colors, typography, spacing, surfaces.
- **God file rule**: Max 500 lines per file. >700 lines = architecture violation.
- **No direct API calls from components**: Only through query hooks (useXxxQuery).
- **No premature shared/ promotion**: Only after use in 2+ independent domains.

## Risks

### R1: Massive scope — 16+ domains with complex UI
The current app has 16 views (Home, Communications, Persons, Projects, Tasks, Calendar, Documents, Notes, Knowledge, Review, Settings, Agents, Organizations, Timeline, Telegram, WhatsApp) plus shell (sidebar, topbar, notifications, layout editor, vault wizard, compose drawer, account setup modal, draft strip, health strip). Each domain has multiple widgets, stores, API endpoints, and CSS.

**Mitigation**: Phase per domain. Each phase independently verifiable (can render correctly). Start with simplest domains (Settings, Home) to build momentum.

### R2: Visual identity preservation during CSS framework migration
Current app uses raw CSS with custom properties (tokens.css, shell.css, app.css). Target uses Tailwind CSS + shadcn-vue. Must port every visual token exactly without changing the look.

**Mitigation**: Port theme tokens to Tailwind config first. Use CSS custom properties as bridge. Compare against existing stylesheet rules. Include a dedicated visual verification step in Polish phase.

### R3: Tauri coexistence during migration
Tauri config (`frontend/src-tauri/tauri.conf.json`) points `frontendDist` to `../build` and `beforeDevCommand` to `pnpm dev`. During migration, Vue app must work while SvelteKit app remains functional.

**Mitigation**: Create new Vue app in parallel structure (e.g., `frontend-vue/` or restructured `frontend/`). Update Tauri config to point to new build output after cutover phase. During development, use separate Vite dev server for Vue app.

### R4: Complex state migration
Current app has ~20+ Svelte stores with complex state management (communications, layoutEditor, navigation, settings, vault, notifications, sidebar, theme, uiState). These must be decomposed into Pinia (UI state) + TanStack Query (server state) with correct boundaries.

**Mitigation**: Phase 1 establishes the state management pattern. Each domain port explicitly redraws the boundary between server state (→ TanStack Query) and UI state (→ Pinia). Verification includes "no server data in Pinia" check.

### R5: Build and type safety
Target stack uses many new libraries (TanStack Query, TanStack Table, TanStack Virtual, shadcn-vue, TipTap, Vue Flow). TypeScript strict mode ensures type safety but library integration may have friction.

**Mitigation**: Phase 1 validates the full toolchain compiles. Each phase includes `pnpm build` and `pnpm check` (or equivalent). Library-specific issues documented and fixed per-phase.

## Dependencies

1. **Phase 1 (Foundation) must complete before any domain port** — all domains depend on platform layer (API client, auth), i18n, theme tokens, shared UI primitives.
2. **Phase 2 (Shell) must complete before visual domain ports** — all domains render inside the shell (sidebar, topbar, workspace).
3. **Domain phases are independent** — can be done in any order once 1-2 are done. Start with simplest (Settings, Home), end with most complex (Communications/Mail).
4. **Tauri config update** — only at final cutover phase.
5. **SvelteKit removal** — only after all domains are ported and verified.

## Open Questions (Assumed)

- **AQ1**: Vue app will be created in `frontend/` directory, replacing the SvelteKit `src/` contents. The old SvelteKit code will be kept in a backup directory or git branch until cutover. 
  - *Decision*: Actually, creating alongside is safer. Vue app goes in `frontend/` as the new primary, SvelteKit remains in `frontend/src-svelte/` during migration.
- **AQ2**: Tauri config will be updated at cutover phase to point to Vue build output.
- **AQ3**: Migration order: Foundation → Shell → Settings → Home → Persons → Projects → Tasks → Calendar → Documents → Notes → Knowledge → Review → Organizations → Agents → Timeline → Communications → Telegram → WhatsApp → Polish → Cutover.

## Memory Hits Applied

None — no memory directory found for this project.

## Tools/Skills Relied On

- `frontend-design` skill — for shared UI primitives design (Level 1 components)
- `codebase_search` — for finding existing patterns in Svelte code to port
- `new_task` — for dispatching parallel domain ports when dependencies allow

## Best Practices Applied

- Domain-Driven Frontend (mirrors backend domain boundaries)
- Strict TypeScript (no `any` without written explanation)
- Composition API with `<script setup>`
- Feature-first decomposition (no flat component folders)
- Query Layer abstraction (no direct fetch in components)
- Real-Time First (SSE/WS over polling)
- Virtualization for all large collections
- God file prevention (max 500 lines)
- TDD for new logic (write failing test first)
