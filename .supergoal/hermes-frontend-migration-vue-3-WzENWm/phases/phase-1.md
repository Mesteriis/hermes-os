SUPERGOAL_PHASE_START
Phase: 1 of 15 — Foundation
Task: Initialize Vue 3 + Vite project scaffold with all dependencies, platform layer, i18n, and theme config
Mandatory commands: cd frontend && pnpm install && pnpm build
Acceptance criteria: 9
Evidence required: build output, src/ listing, src-svelte/ existence, tailwind config
Depends on phases: none

## Why

All subsequent phases depend on the Vue project scaffold, build configuration, platform layer, i18n system, and theme tokens. This phase establishes the foundation that every domain builds upon.

## Work

1. **Restructure `frontend/` directory:**
   - Move existing SvelteKit `frontend/src/` to `frontend/src-svelte/`
   - Move existing SvelteKit `frontend/static/` to `frontend/static-svelte/`
   - Update `frontend/.gitignore` to exclude new Vue build artifacts while keeping SvelteKit artifacts ignored
   - Update `frontend/vite.config.ts` to remove the old SvelteKit static adapter config

2. **Initialize Vue 3 + Vite:**
   - Add Vue 3 core dependencies to `frontend/package.json`: `vue`, `vue-router`
   - Add Vite Vue plugin: `@vitejs/plugin-vue`
   - Create `frontend/vite.config.ts` with Vue plugin and Tauri-compatible dev server config (port 5173)
   - Create `frontend/tsconfig.json` with strict TypeScript, Vue compiler options
   - Create `frontend/index.html` as Vue entry point
   - Create `frontend/src/main.ts` — Vue app creation with router
   - Create `frontend/src/app/App.vue` — minimal root component with `<router-view>`
   - Create `frontend/src/app/router.ts` — Vue Router with empty route table (populated in Phase 2)

3. **Install and configure target dependencies:**
   - Add and install via pnpm: `pinia`, `@tanstack/vue-query`, `@tanstack/vue-table`, `@tanstack/vue-virtual`, `tailwindcss`, `postcss`, `autoprefixer`, `shadcn-vue` init, `@tiptap/vue-3`, `@vue-flow/core`, `date-fns`, `motion-v` (or motion/vue), `@iconify/vue`
   - Initialize Tailwind CSS: `frontend/tailwind.config.ts`, `frontend/postcss.config.js`
   - Initialize shadcn-vue components system
   - Configure Pinia plugin in `frontend/src/main.ts`
   - Configure TanStack Query (VueQuery) plugin in `frontend/src/main.ts`

4. **Port theme tokens:**
   - Read `frontend/src-svelte/lib/styles/tokens.css` (or the moved CSS files) and extract all CSS custom properties
   - Map every CSS custom property to Tailwind theme extension in `frontend/tailwind.config.ts`:
     - Colors (primary, surface, text, border, accent, danger, warning, success)
     - Spacing scale
     - Typography (font families, sizes, weights, line heights)
     - Border radius
     - Shadow/elevation tokens
     - Transition durations
   - Create `frontend/src/platform/theme/tokens.ts` — typed token constants for use outside Tailwind classes
   - Create CSS bridge file that imports Tailwind directives + applies Hermes custom properties at `:root`

5. **Port i18n system:**
   - Create `frontend/src/platform/i18n/ru.json` — copy from existing ru.json dictionary
   - Create `frontend/src/platform/i18n/en.json` — copy from existing en.json
   - Create `frontend/src/platform/i18n/index.ts` — Vue composable `useI18n()` that returns `t(key: string): string` function
   - The composable should work with a Pinia store for current locale (persisted to localStorage)
   - English strings are keys; `t()` returns the value from current locale dictionary or falls back to key
   - Create `frontend/src/platform/i18n/types.ts` — type for translation function

6. **Port API client:**
   - Create `frontend/src/platform/api/ApiClient.ts` — port from existing `frontend/src-svelte/lib/api/client.ts`
   - Same interface: `get<T>`, `post<T>`, `put<T>`, `patch<T>`, `delete<T>`
   - Same auth: reads `X-Hermes-Secret` from config/env
   - Create `frontend/src/platform/api/index.ts` — exports singleton instance
   - Create `frontend/src/platform/api/types.ts` — shared API types (ApiError, pagination types)
   - Create `frontend/src/config/index.ts` — `apiBaseUrl` from env/Vite import.meta.env

7. **Create SSE client foundation:**
   - Create `frontend/src/platform/sse/SseClient.ts` — EventSource wrapper with reconnect logic
   - Create `frontend/src/platform/sse/index.ts` — exports

8. **Configure linting for Vue/TypeScript:**
   - Add `vue-tsc` for type checking
   - Add ESLint with `@typescript-eslint` and `eslint-plugin-vue` if not present
   - Configure `frontend/package.json` scripts: `"lint:ts": "vue-tsc --noEmit"`, `"lint": "pnpm lint:ts"`
   - (The old Svelte lint scripts will be replaced at Cutover phase)

9. **Verify:**
   - Run `pnpm install` — all dependencies resolve
   - Run `pnpm build` — exits 0, produces `frontend/dist/` with Vue app
   - Run `pnpm lint:ts` — type check passes (or baseline acceptable errors)
   - Confirm `frontend/src-svelte/` directory exists with the old SvelteKit code

## Acceptance criteria (all must pass — verify each in transcript)

- [ ] AC1: `cd frontend && pnpm build` exits 0 with no errors
- [ ] AC2: `cd frontend && pnpm lint:ts` (vue-tsc) exits 0
- [ ] AC3: Vue app renders a blank page at `http://localhost:5173` (start dev server, verify)
- [ ] AC4: i18n system loads ru.json and renders Russian text in the test component
- [ ] AC5: ApiClient sends `X-Hermes-Secret` header correctly (inspect via curl or log)
- [ ] AC6: Tailwind theme in `tailwind.config.ts` matches all Hermes color tokens from tokens.css
- [ ] AC7: TypeScript strict mode enabled in tsconfig (`strict: true`, no `strict: false`)
- [ ] AC8: No `.svelte` or `.svelte-kit` files in the new `frontend/src/` directory
- [ ] AC9: Old SvelteKit code exists in `frontend/src-svelte/` and is excluded from the new build

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cd frontend && pnpm install && pnpm build`
- `cd frontend && npx vue-tsc --noEmit` (if configured, else skip)

## Evidence required in transcript

- `pnpm build` output — last 10 lines showing success
- File listing of `frontend/src/` (showing new Vue structure)
- File listing confirming `frontend/src-svelte/` exists
- Tailwind config snippet showing Hermes color tokens
- ApiClient class showing X-Hermes-Secret header

## Notes

- Reference the existing SvelteKit code in `frontend/src-svelte/` for token values and i18n dictionaries
- Do NOT delete the old SvelteKit code — it stays until Phase 15
- The Tauri config in `frontend/src-tauri/tauri.conf.json` is NOT modified in this phase
- If shadcn-vue init requires interactive prompts, use `yes "" | npx shadcn-vue init` or equivalent non-interactive mode
- For `@tanstack/vue-table`, install `@tanstack/table-core` as well
- Motion library: use `motion-v` (motion for Vue) — `npm install motion-v` or equivalent
