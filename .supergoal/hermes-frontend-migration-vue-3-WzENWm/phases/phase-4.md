SUPERGOAL_PHASE_START
Phase: 4 of 15 — Settings Domain
Task: Port the Settings page with all panels to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 7
Evidence required: build output, settings domain file listing
Depends on phases: 1, 2, 3

## Why

Settings is the simplest domain and serves as the first real domain port to validate the architecture pattern (domain structure, TanStack Query, Pinia boundary, component hierarchy).

## Work

1. **Create settings domain structure** under `frontend/src/domains/settings/`:
   - `types/settings.ts` — TypeScript types for settings
   - `api/settings.ts` — API functions (getSettings, updateSettings, etc.)
   - `queries/useSettingsQuery.ts` — TanStack Query hooks
   - `stores/settings.ts` — Pinia store for UI state only (active section, action messages)
   - `views/SettingsPage.vue` — main settings page
   - `components/` — settings-specific components
   - `routes/index.ts` — route definitions for settings

2. **Port each settings panel:**
   - AppearanceSettings — theme toggle (light/dark)
   - LanguageSettings — locale switch (en/ru)
   - IntegrationsSettings — connected accounts list
   - SidebarSettings — sidebar configuration
   - ApplicationSettings — app-level settings
   - AI Settings panels (OverviewPanel, AIApiProvidersPanel, AIBuiltInProvidersPanel, AICliProvidersPanel, AIModelRoutingPanel, AIPromptStudioPanel, AIRunsHealthPanel, AISettingsControlCenter, AISettingsHeader, AISettingsRail, AISettingsStatus, AISettingsTabs)

3. **Register route** in `frontend/src/app/router.ts`

4. **Verify:**
   - Build passes
   - Each settings panel renders with correct data from API

## Acceptance criteria

- [ ] AC1: All settings panels render with correct data from backend API via TanStack Query
- [ ] AC2: Theme toggle (light/dark) works and persists across reload
- [ ] AC3: Language switch (en/ru) works and persists — i18n store updates reactively
- [ ] AC4: Integrations settings shows connected accounts list
- [ ] AC5: AI settings panels render
- [ ] AC6: `cd frontend && pnpm build` exits 0
- [ ] AC7: Settings page is accessible via route `/settings`

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output
- List of settings domain files
- Note on which API endpoints are used

## Notes

- Reference Svelte implementations in `frontend/src-svelte/lib/pages/settings/` and `frontend/src-svelte/lib/stores/settings.ts`
- API endpoints: study `frontend/src-svelte/lib/api/endpoints/settings.ts` and `frontend/src-svelte/lib/services/settings.ts`
- Keep each Vue component under 500 lines
