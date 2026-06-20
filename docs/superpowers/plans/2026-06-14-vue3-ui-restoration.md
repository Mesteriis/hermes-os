# Vue 3 UI Restoration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore the post-migration Vue 3 Hermes Hub interface to the `7f1cf42f` visual baseline while keeping the UI fully interactive, token/theme-configurable, and compliant with SRP limits.

**Architecture:** Use a restoration kernel plus domain slices. First restore shell/theme/runtime foundations, then restore each screen as focused Vue domain components without introducing a widget/layout builder. Domain views stay composition roots; TanStack Query owns server-derived state; Pinia owns transient UI state only.

**Tech Stack:** Vue 3, TypeScript, Vite, Pinia, TanStack Query, Tauri 2, Tailwind CSS, CSS custom properties, Vitest.

---

## Ground Rules

- Do not introduce a widget system or layout builder.
- Do not implement mobile or tablet UI. Validate desktop window sizes only.
- Do not reintroduce bearer auth or `X-Hermes-Actor-Id`; use `X-Hermes-Secret`.
- Do not add inline `style=` attributes.
- Do not add new hardcoded visual constants in domain components. Use tokens or shared primitives.
- No component over `700` lines may remain in changed code. Components over `500` lines must be split before completion.
- This repository forbids commits unless the user explicitly asks. Treat any "Commit" step from generic process guidance as a local checkpoint: inspect `git diff`, do not run `git commit`.

## Reference Sources

- Visual baseline: Git commit `7f1cf42f` (`before Vue3`).
- Current Vue target: `frontend/src/`.
- Design spec: `docs/superpowers/specs/2026-06-14-vue3-ui-restoration-design.md`.
- Relevant ADRs:
  - `docs/adr/ADR-0004-tauri-desktop-shell.md`
  - `docs/adr/ADR-0026-desktop-first-responsive-ui.md`
  - `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md`
  - `docs/adr/ADR-0054-application-settings-store.md`
  - `docs/adr/ADR-0056-local-api-simplified-auth.md`
  - `docs/adr/ADR-0077-i18n-russian-english.md`
  - `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md`

## Current High-Risk Files

Current line-count scan shows these files need decomposition or careful handling:

- `frontend/src/integrations/telegram/components/TelegramMessageThread.vue` — 903 lines, violates hard limit.
- `frontend/src/domains/communications/views/CommunicationsPage.vue` — 890 lines, violates hard limit.
- `frontend/src/domains/settings/components/SidebarSettings.vue` — 660 lines, must be split before completion.
- `frontend/src/domains/settings/components/AppearanceSettings.vue` — 462 lines, warning zone.
- `frontend/src/domains/communications/components/ComposeDrawer.vue` — 447 lines, warning zone.
- `frontend/src/domains/review/views/ReviewPage.vue` — 470 lines, warning zone.
- `frontend/src/integrations/telegram/views/TelegramPage.vue` — 426 lines, warning zone.
- `frontend/src/domains/knowledge/views/KnowledgePage.vue` — 407 lines, warning zone.

## File Map

### Create

- `frontend/src/platform/config/env.ts` — typed env parsing and visible bootstrap errors.
- `frontend/src/platform/config/env.test.ts` — env parsing regression tests.
- `frontend/src/platform/bootstrap/api.ts` — initializes `ApiClient`.
- `frontend/src/platform/bootstrap/api.test.ts` — bootstrap/init regression tests.
- `frontend/src/platform/theme/settings.ts` — schema-versioned theme parser and allowlisted classes.
- `frontend/src/platform/theme/settings.test.ts` — theme parser/class tests.
- `frontend/src/platform/theme/persistence.ts` — application-settings persistence boundary with local fallback.
- `frontend/src/shared/ui/Surface.vue` — shared tokenized panel/surface primitive.
- `frontend/src/domains/communications/components/message-renderer/sanitizeEmailHtml.ts` — safe email HTML/text rendering utilities.
- `frontend/src/domains/communications/components/message-renderer/sanitizeEmailHtml.test.ts` — message renderer security tests.
- `frontend/src/domains/communications/components/CommunicationsActionBar.vue`
- `frontend/src/domains/communications/components/CommunicationsWorkbench.vue`
- `frontend/src/domains/communications/components/CommunicationsListPane.vue`
- `frontend/src/domains/communications/components/CommunicationsDetailPane.vue`
- `frontend/src/domains/communications/components/CommunicationsRailPane.vue`
- `frontend/src/integrations/telegram/components/thread/TelegramThreadHeader.vue`
- `frontend/src/integrations/telegram/components/thread/TelegramMessageList.vue`
- `frontend/src/integrations/telegram/components/thread/TelegramComposer.vue`
- `frontend/src/integrations/telegram/components/thread/TelegramSyncPanel.vue`
- `frontend/src/domains/settings/components/sidebar/SidebarNavigationList.vue`
- `frontend/src/domains/settings/components/sidebar/SidebarGroupEditor.vue`
- `frontend/src/domains/settings/components/sidebar/SidebarItemEditor.vue`
- `frontend/src/domains/settings/components/appearance/AppearanceHeader.vue`
- `frontend/src/domains/settings/components/appearance/BackgroundPicker.vue`
- `frontend/src/domains/settings/components/appearance/AccentPicker.vue`
- `frontend/src/domains/settings/components/appearance/ThemeRangeControl.vue`
- `frontend/src/domains/settings/components/appearance/SpacingDensityControl.vue`
- `frontend/scripts/check-component-lines.mjs` — SRP line-count gate.
- `frontend/scripts/capture-vue-baseline-screenshots.mjs` — current Vue screenshot capture.

### Modify

- `frontend/package.json` — add validation scripts.
- `frontend/src/main.ts` — initialize config/API before mount and show bootstrap errors.
- `frontend/src/config/index.ts` — replace with compatibility re-export from `platform/config/env`.
- `frontend/src/platform/api/ApiClient.ts` — reject empty secret during init and keep request header behavior.
- `frontend/src/platform/api/index.ts` — keep API exports stable if import paths change.
- `frontend/src/platform/sse/SseClient.ts` — align SSE URL env if it uses old names.
- `frontend/src/platform/theme/tokens.ts` — expand typed tokens for spacing density and theme classes.
- `frontend/src/shared/stores/theme.ts` — use schema-versioned settings and app-settings persistence.
- `frontend/src/style.css` — restore shell/theme CSS from `7f1cf42f`, expressed as tokens.
- `frontend/tailwind.config.ts` — keep Tailwind token aliases aligned with CSS custom properties.
- `frontend/src/app/shell/AppShell.vue`
- `frontend/src/app/shell/Sidebar.vue`
- `frontend/src/app/shell/Topbar.vue`
- `frontend/src/app/shell/NotificationsDrawer.vue`
- `frontend/src/domains/settings/components/AppearanceSettings.vue`
- `frontend/src/domains/settings/components/SidebarSettings.vue`
- `frontend/src/domains/communications/views/CommunicationsPage.vue`
- `frontend/src/domains/communications/components/MessageBodyTab.vue`
- `frontend/src/integrations/telegram/components/TelegramMessageThread.vue`
- `frontend/src/domains/calendar/views/CalendarPage.vue`
- `frontend/src/domains/tasks/views/TasksPage.vue`
- every domain view under `frontend/src/domains/*/views/*.vue` during screen restoration.

### Restore Assets

Copy these from `7f1cf42f:frontend/static/` into the current Vite static directory `frontend/public/`.

- `assets/hermes-logo.png`
- `assets/hermes-logo-mark.png`
- `assets/hermes-reference-avatar.png`
- `assets/shell-backgrounds/data-stream.png`
- `assets/shell-backgrounds/dna-blueprint.png`
- `assets/shell-backgrounds/eclipse-grid.png`
- `assets/shell-backgrounds/forest-network.png`
- `assets/shell-backgrounds/forest-stream.png`
- `assets/shell-backgrounds/knowledge-map.png`
- `assets/shell-backgrounds/network-mesh.png`
- `assets/shell-backgrounds/node-frame.png`
- `assets/shell-backgrounds/rune-gold.png`
- `assets/shell-backgrounds/rune-teal.png`

---

## Task 1: Baseline Evidence And Screenshot Setup

**Files:**
- Create: `frontend/scripts/capture-vue-baseline-screenshots.mjs`
- Modify: `frontend/package.json`
- Reference: `frontend/scripts/capture-layout-screenshots.mjs`

- [ ] **Step 1: Record current baseline sources**

Run:

```sh
git show --stat --oneline 7f1cf42f -- frontend | sed -n '1,120p'
git ls-tree -r --name-only 7f1cf42f frontend/static frontend/src/lib | sed -n '1,220p'
```

Expected: output includes old Svelte pages, CSS files, layout/theme files and shell assets.

- [ ] **Step 2: Create a Vue screenshot capture script**

Add `frontend/scripts/capture-vue-baseline-screenshots.mjs`:

```js
import { mkdir, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import { chromium } from 'playwright'

const routes = [
  ['home', '/#/home'],
  ['communications', '/#/communications'],
  ['communications', 'telegram', '/#/communications'],
  ['communications', 'whatsapp', '/#/communications'],
  ['timeline', '/#/timeline'],
  ['persons', '/#/persons'],
  ['projects', '/#/projects'],
  ['tasks', '/#/tasks'],
  ['calendar', '/#/calendar'],
  ['documents', '/#/documents'],
  ['notes', '/#/notes'],
  ['knowledge', '/#/knowledge'],
  ['review', '/#/review'],
  ['agents', '/#/agents'],
  ['organizations', '/#/organizations'],
  ['settings', '/#/settings']
]

const viewports = [
  { id: '800x600', width: 800, height: 600 },
  { id: '1600x1000', width: 1600, height: 1000 }
]

const url = process.argv[2] ?? 'http://127.0.0.1:5174'
const outputDir =
  process.argv[3] ??
  path.join(os.tmpdir(), `hermes-vue-screens-${new Date().toISOString().replace(/[:.]/g, '-')}`)

await mkdir(outputDir, { recursive: true })

const browser = await chromium.launch()
const report = []

for (const viewport of viewports) {
  const viewportDir = path.join(outputDir, viewport.id)
  await mkdir(viewportDir, { recursive: true })

  for (const [id, route] of routes) {
    const page = await browser.newPage({ viewport: { width: viewport.width, height: viewport.height } })
    const consoleIssues = []
    page.on('console', (message) => {
      if (['error', 'warning'].includes(message.type())) {
        consoleIssues.push(`${message.type()}: ${message.text()}`)
      }
    })

    await page.goto(`${url}${route}`, { waitUntil: 'domcontentloaded' })
    await page.waitForTimeout(1000)
    const screenshot = path.join(viewportDir, `${id}.png`)
    await page.screenshot({ path: screenshot, fullPage: false })
    const state = await page.evaluate(() => {
      const body = document.body
      const root = document.querySelector('#app')
      const shell = document.querySelector('.desktop-shell')
      return {
        bodyText: body.textContent?.slice(0, 300) ?? '',
        hasRoot: root !== null,
        hasShell: shell !== null,
        horizontalOverflow: document.documentElement.scrollWidth > window.innerWidth + 1,
        verticalOverflow: document.documentElement.scrollHeight > window.innerHeight + 1
      }
    })
    report.push({ id, viewport: viewport.id, screenshot, consoleIssues, state })
    await page.close()
  }
}

await browser.close()
await writeFile(path.join(outputDir, 'report.json'), JSON.stringify(report, null, 2))
console.log(JSON.stringify({ outputDir, reportPath: path.join(outputDir, 'report.json') }, null, 2))
```

- [ ] **Step 3: Add screenshot script to package.json**

Modify `frontend/package.json` scripts:

```json
"screenshots:vue": "node scripts/capture-vue-baseline-screenshots.mjs"
```

Keep existing scripts unchanged.

- [ ] **Step 4: Run type validation**

Run:

```sh
cd frontend && pnpm lint:ts
```

Expected: `vue-tsc --noEmit` passes. If it fails, fix the script/package typing issue before continuing.

- [ ] **Step 5: Local checkpoint**

Run:

```sh
git diff -- frontend/package.json frontend/scripts/capture-vue-baseline-screenshots.mjs
git status --short
```

Expected: only the new script and package script change for this task. Do not commit unless the user explicitly asks.

---

## Task 2: Restore Static Shell Assets

**Files:**
- Create: `frontend/public/assets/...`
- Modify: `frontend/src/style.css`
- Reference: `7f1cf42f:frontend/static/assets/shell-backgrounds/*.png`

- [ ] **Step 1: Restore assets from Git**

Run:

```sh
mkdir -p frontend/public/assets/shell-backgrounds
git show 7f1cf42f:frontend/static/assets/hermes-logo.png > frontend/public/assets/hermes-logo.png
git show 7f1cf42f:frontend/static/assets/hermes-logo-mark.png > frontend/public/assets/hermes-logo-mark.png
git show 7f1cf42f:frontend/static/assets/hermes-reference-avatar.png > frontend/public/assets/hermes-reference-avatar.png
for name in data-stream dna-blueprint eclipse-grid forest-network forest-stream knowledge-map network-mesh node-frame rune-gold rune-teal; do
  git show "7f1cf42f:frontend/static/assets/shell-backgrounds/$name.png" > "frontend/public/assets/shell-backgrounds/$name.png"
done
```

Expected: all files exist under `frontend/public/assets/`.

- [ ] **Step 2: Verify assets are real images**

Run:

```sh
file frontend/public/assets/shell-backgrounds/*.png frontend/public/assets/hermes-logo*.png
```

Expected: each file reports `PNG image data`.

- [ ] **Step 3: Point shell background classes at restored paths**

In `frontend/src/style.css`, ensure background classes use these URLs:

```css
.shell-bg-network-mesh { --hh-shell-bg-image: url('/assets/shell-backgrounds/network-mesh.png'); }
.shell-bg-data-stream { --hh-shell-bg-image: url('/assets/shell-backgrounds/data-stream.png'); }
.shell-bg-node-frame { --hh-shell-bg-image: url('/assets/shell-backgrounds/node-frame.png'); }
.shell-bg-eclipse-grid { --hh-shell-bg-image: url('/assets/shell-backgrounds/eclipse-grid.png'); }
.shell-bg-dna-blueprint { --hh-shell-bg-image: url('/assets/shell-backgrounds/dna-blueprint.png'); }
.shell-bg-forest-network { --hh-shell-bg-image: url('/assets/shell-backgrounds/forest-network.png'); }
.shell-bg-forest-stream { --hh-shell-bg-image: url('/assets/shell-backgrounds/forest-stream.png'); }
.shell-bg-knowledge-map { --hh-shell-bg-image: url('/assets/shell-backgrounds/knowledge-map.png'); }
.shell-bg-rune-gold { --hh-shell-bg-image: url('/assets/shell-backgrounds/rune-gold.png'); }
.shell-bg-rune-teal { --hh-shell-bg-image: url('/assets/shell-backgrounds/rune-teal.png'); }
```

- [ ] **Step 4: Build asset bundle**

Run:

```sh
cd frontend && pnpm build
```

Expected: Vite build passes and does not report missing static assets.

- [ ] **Step 5: Local checkpoint**

Run:

```sh
git status --short frontend/public frontend/src/style.css
```

Expected: restored assets and any CSS path edits are visible. Do not commit unless the user explicitly asks.

---

## Task 3: Typed Env And API Bootstrap

**Files:**
- Create: `frontend/src/platform/config/env.ts`
- Create: `frontend/src/platform/config/env.test.ts`
- Create: `frontend/src/platform/bootstrap/api.ts`
- Create: `frontend/src/platform/bootstrap/api.test.ts`
- Modify: `frontend/src/config/index.ts`
- Modify: `frontend/src/platform/api/ApiClient.ts`
- Modify: `frontend/src/main.ts`

- [ ] **Step 1: Write env parser tests**

Create `frontend/src/platform/config/env.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { loadFrontendConfig } from './env'

describe('frontend env config', () => {
  it('uses Hermes env names and default backend URL', () => {
    const config = loadFrontendConfig({
      VITE_HERMES_LOCAL_API_SECRET: 'dev-secret'
    })

    expect(config.apiBaseUrl).toBe('http://127.0.0.1:8080')
    expect(config.apiSecret).toBe('dev-secret')
    expect(config.sseUrl).toBe('http://127.0.0.1:8080/api/events/stream')
  })

  it('rejects missing local API secret', () => {
    expect(() => loadFrontendConfig({})).toThrow('VITE_HERMES_LOCAL_API_SECRET is required')
  })

  it('accepts explicit Hermes backend URL', () => {
    const config = loadFrontendConfig({
      VITE_HERMES_API_BASE_URL: 'http://127.0.0.1:9090/',
      VITE_HERMES_LOCAL_API_SECRET: 'dev-secret'
    })

    expect(config.apiBaseUrl).toBe('http://127.0.0.1:9090')
    expect(config.sseUrl).toBe('http://127.0.0.1:9090/api/events/stream')
  })
})
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```sh
cd frontend && pnpm test:unit -- src/platform/config/env.test.ts
```

Expected: FAIL because `env.ts` does not exist.

- [ ] **Step 3: Implement env parser**

Create `frontend/src/platform/config/env.ts`:

```ts
export type FrontendConfig = {
  apiBaseUrl: string
  apiSecret: string
  sseUrl: string
}

type EnvSource = Record<string, string | boolean | undefined>

const DEFAULT_API_BASE_URL = 'http://127.0.0.1:8080'

export function loadFrontendConfig(env: EnvSource = import.meta.env): FrontendConfig {
  const apiBaseUrl = normalizeBaseUrl(
    stringValue(env.VITE_HERMES_API_BASE_URL) ?? DEFAULT_API_BASE_URL
  )
  const apiSecret = stringValue(env.VITE_HERMES_LOCAL_API_SECRET)

  if (!apiSecret) {
    throw new Error('VITE_HERMES_LOCAL_API_SECRET is required')
  }

  return {
    apiBaseUrl,
    apiSecret,
    sseUrl: stringValue(env.VITE_HERMES_SSE_URL) ?? `${apiBaseUrl}/api/events/stream`
  }
}

function stringValue(value: string | boolean | undefined): string | undefined {
  return typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined
}

function normalizeBaseUrl(value: string): string {
  return value.replace(/\/+$/, '')
}
```

- [ ] **Step 4: Re-export compatibility config**

Replace `frontend/src/config/index.ts` with:

```ts
import { loadFrontendConfig } from '../platform/config/env'

export const config = loadFrontendConfig()
```

- [ ] **Step 5: Add API bootstrap tests**

Create `frontend/src/platform/bootstrap/api.test.ts`:

```ts
import { beforeEach, describe, expect, it } from 'vitest'
import { ApiClient } from '../api/ApiClient'
import { initializeApiClient } from './api'

describe('initializeApiClient', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
  })

  it('initializes the singleton from config', () => {
    initializeApiClient({
      apiBaseUrl: 'http://127.0.0.1:8080',
      apiSecret: 'dev-secret',
      sseUrl: 'http://127.0.0.1:8080/api/events/stream'
    })

    expect(ApiClient.instance).toBeInstanceOf(ApiClient)
  })
})
```

- [ ] **Step 6: Update ApiClient test support and empty-secret guard**

Modify `frontend/src/platform/api/ApiClient.ts`:

```ts
static init(baseUrl: string, secret: string): ApiClient {
  if (secret.trim().length === 0) {
    throw new Error('X-Hermes-Secret cannot be empty')
  }
  ApiClient._instance = new ApiClient(baseUrl, secret)
  return ApiClient._instance
}

static resetForTests(): void {
  ApiClient._instance = null
}
```

Update `frontend/src/__tests__/apiClient.test.ts` to use `ApiClient.resetForTests()` instead of private field access.

- [ ] **Step 7: Implement API bootstrap**

Create `frontend/src/platform/bootstrap/api.ts`:

```ts
import { ApiClient } from '../api/ApiClient'
import type { FrontendConfig } from '../config/env'

export function initializeApiClient(config: FrontendConfig): ApiClient {
  return ApiClient.init(config.apiBaseUrl, config.apiSecret)
}
```

- [ ] **Step 8: Initialize before mount**

Modify `frontend/src/main.ts`:

```ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './app/App.vue'
import router from './app/router'
import { loadFrontendConfig } from './platform/config/env'
import { initializeApiClient } from './platform/bootstrap/api'
import './style.css'

const app = createApp(App)

try {
  initializeApiClient(loadFrontendConfig())
} catch (error) {
  document.body.innerHTML = `<main class="startup-error"><h1>Hermes Hub cannot start</h1><p>${escapeHtml(error instanceof Error ? error.message : 'Unknown startup error')}</p></main>`
  throw error
}

app.use(createPinia())
app.use(VueQueryPlugin)
app.use(router)
app.mount('#app')

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}
```

- [ ] **Step 9: Run targeted tests**

Run:

```sh
cd frontend && pnpm test:unit -- src/platform/config/env.test.ts src/platform/bootstrap/api.test.ts src/__tests__/apiClient.test.ts
```

Expected: all targeted tests pass.

- [ ] **Step 10: Run frontend build**

Run:

```sh
cd frontend && pnpm build
```

Expected: build passes.

---

## Task 4: Theme Settings Parser And CSS Class Contract

**Files:**
- Create: `frontend/src/platform/theme/settings.ts`
- Create: `frontend/src/platform/theme/settings.test.ts`
- Modify: `frontend/src/platform/theme/tokens.ts`
- Modify: `frontend/src/shared/stores/theme.ts`
- Modify: `frontend/src/domains/settings/components/AppearanceSettings.vue`

- [ ] **Step 1: Write theme parser tests**

Create `frontend/src/platform/theme/settings.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import {
  defaultThemeSettings,
  parseThemeSettings,
  shellAccentClass,
  shellBackgroundClass,
  shellBrightnessClass,
  shellPanelBlurClass,
  shellPanelOpacityClass,
  shellSpacingDensityClass
} from './settings'

describe('theme settings', () => {
  it('returns defaults for invalid values', () => {
    expect(parseThemeSettings(null)).toEqual(defaultThemeSettings())
    expect(parseThemeSettings({ schemaVersion: 99 })).toEqual(defaultThemeSettings())
  })

  it('keeps allowlisted values', () => {
    expect(parseThemeSettings({
      schemaVersion: 1,
      shellBackground: 'rune-teal',
      backgroundBrightness: 90,
      accentColor: 'cyan',
      panelOpacity: 50,
      panelBlur: 20,
      spacingDensity: 'compact'
    })).toEqual({
      schemaVersion: 1,
      shellBackground: 'rune-teal',
      backgroundBrightness: 90,
      accentColor: 'cyan',
      panelOpacity: 50,
      panelBlur: 20,
      spacingDensity: 'compact'
    })
  })

  it('returns allowlisted CSS classes', () => {
    const settings = parseThemeSettings({
      schemaVersion: 1,
      shellBackground: 'network-mesh',
      backgroundBrightness: 70,
      accentColor: 'violet',
      panelOpacity: 80,
      panelBlur: 12,
      spacingDensity: 'comfortable'
    })

    expect(shellBackgroundClass(settings)).toBe('shell-bg-network-mesh')
    expect(shellBrightnessClass(settings)).toBe('shell-bg-brightness-70')
    expect(shellAccentClass(settings)).toBe('theme-accent-violet')
    expect(shellPanelOpacityClass(settings)).toBe('panel-opacity-80')
    expect(shellPanelBlurClass(settings)).toBe('panel-blur-12')
    expect(shellSpacingDensityClass(settings)).toBe('spacing-density-comfortable')
  })
})
```

- [ ] **Step 2: Run test and verify failure**

Run:

```sh
cd frontend && pnpm test:unit -- src/platform/theme/settings.test.ts
```

Expected: FAIL because `settings.ts` does not exist.

- [ ] **Step 3: Implement theme settings**

Create `frontend/src/platform/theme/settings.ts`:

```ts
export const THEME_SCHEMA_VERSION = 1

export const shellBackgroundIds = [
  'none',
  'network-mesh',
  'data-stream',
  'node-frame',
  'eclipse-grid',
  'dna-blueprint',
  'forest-network',
  'forest-stream',
  'knowledge-map',
  'rune-gold',
  'rune-teal'
] as const

export const backgroundBrightnessValues = [30, 40, 50, 60, 70, 80, 90, 100] as const
export const accentColorIds = ['teal', 'cyan', 'blue', 'violet', 'amber', 'rose'] as const
export const panelOpacityValues = [40, 50, 60, 70, 80, 90, 100] as const
export const panelBlurValues = [0, 4, 8, 12, 16, 20, 24] as const
export const spacingDensityIds = ['compact', 'normal', 'comfortable'] as const

export type ShellBackgroundId = (typeof shellBackgroundIds)[number]
export type BackgroundBrightness = (typeof backgroundBrightnessValues)[number]
export type AccentColorId = (typeof accentColorIds)[number]
export type PanelOpacity = (typeof panelOpacityValues)[number]
export type PanelBlur = (typeof panelBlurValues)[number]
export type SpacingDensity = (typeof spacingDensityIds)[number]

export type ThemeSettings = {
  schemaVersion: typeof THEME_SCHEMA_VERSION
  shellBackground: ShellBackgroundId
  backgroundBrightness: BackgroundBrightness
  accentColor: AccentColorId
  panelOpacity: PanelOpacity
  panelBlur: PanelBlur
  spacingDensity: SpacingDensity
}

export function defaultThemeSettings(): ThemeSettings {
  return {
    schemaVersion: THEME_SCHEMA_VERSION,
    shellBackground: 'network-mesh',
    backgroundBrightness: 70,
    accentColor: 'teal',
    panelOpacity: 70,
    panelBlur: 12,
    spacingDensity: 'normal'
  }
}

export function parseThemeSettings(value: unknown): ThemeSettings {
  if (!isRecord(value) || value.schemaVersion !== THEME_SCHEMA_VERSION) {
    return defaultThemeSettings()
  }
  const defaults = defaultThemeSettings()
  return {
    schemaVersion: THEME_SCHEMA_VERSION,
    shellBackground: pick(value.shellBackground, shellBackgroundIds, defaults.shellBackground),
    backgroundBrightness: pick(value.backgroundBrightness, backgroundBrightnessValues, defaults.backgroundBrightness),
    accentColor: pick(value.accentColor, accentColorIds, defaults.accentColor),
    panelOpacity: pick(value.panelOpacity, panelOpacityValues, defaults.panelOpacity),
    panelBlur: pick(value.panelBlur, panelBlurValues, defaults.panelBlur),
    spacingDensity: pick(value.spacingDensity, spacingDensityIds, defaults.spacingDensity)
  }
}

export function shellBackgroundClass(settings: ThemeSettings): string {
  return `shell-bg-${settings.shellBackground}`
}

export function shellBrightnessClass(settings: ThemeSettings): string {
  return `shell-bg-brightness-${settings.backgroundBrightness}`
}

export function shellAccentClass(settings: ThemeSettings): string {
  return `theme-accent-${settings.accentColor}`
}

export function shellPanelOpacityClass(settings: ThemeSettings): string {
  return `panel-opacity-${settings.panelOpacity}`
}

export function shellPanelBlurClass(settings: ThemeSettings): string {
  return `panel-blur-${settings.panelBlur}`
}

export function shellSpacingDensityClass(settings: ThemeSettings): string {
  return `spacing-density-${settings.spacingDensity}`
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function pick<const T extends readonly (string | number)[]>(
  value: unknown,
  allowed: T,
  fallback: T[number]
): T[number] {
  return allowed.includes(value as T[number]) ? (value as T[number]) : fallback
}
```

- [ ] **Step 4: Update theme store to use parser/classes**

Modify `frontend/src/shared/stores/theme.ts` so it imports `ThemeSettings`, `defaultThemeSettings`, `parseThemeSettings`, and class helpers from `platform/theme/settings`. Keep the same Pinia public API where possible, but rename old `shellBrightness` enum to numeric `backgroundBrightness`.

The computed class should be:

```ts
const shellThemeClass = computed<string>(() => {
  const settings = effectiveThemeSettings.value
  return [
    shellBackgroundClass(settings),
    shellBrightnessClass(settings),
    shellAccentClass(settings),
    shellPanelOpacityClass(settings),
    shellPanelBlurClass(settings),
    shellSpacingDensityClass(settings)
  ].join(' ')
})
```

- [ ] **Step 5: Add spacing density CSS variables**

In `frontend/src/style.css`, add:

```css
.spacing-density-compact {
  --hh-density-scale: 0.84;
  --hh-space-panel: 10px;
  --hh-space-section: 12px;
  --hh-space-control-x: 9px;
}

.spacing-density-normal {
  --hh-density-scale: 1;
  --hh-space-panel: 14px;
  --hh-space-section: 16px;
  --hh-space-control-x: 12px;
}

.spacing-density-comfortable {
  --hh-density-scale: 1.14;
  --hh-space-panel: 18px;
  --hh-space-section: 20px;
  --hh-space-control-x: 14px;
}
```

Replace repeated panel padding in restored/shared styles with `var(--hh-space-panel)` or `var(--hh-space-section)`.

- [ ] **Step 6: Add AppearanceSettings density control**

Add a focused `SpacingDensityControl.vue` later in Task 8. For this task, keep `AppearanceSettings.vue` compiling by wiring a simple selector against `theme.updateThemeDraft({ spacingDensity: id })`.

- [ ] **Step 7: Run targeted tests**

Run:

```sh
cd frontend && pnpm test:unit -- src/platform/theme/settings.test.ts
cd frontend && pnpm lint:ts
```

Expected: tests and typecheck pass.

---

## Task 4.5: Theme Persistence Boundary

**Files:**
- Create: `frontend/src/platform/theme/persistence.ts`
- Modify: `frontend/src/shared/stores/theme.ts`
- Modify: `frontend/src/domains/settings/api/settings.ts`

- [ ] **Step 1: Create persistence helper**

Create `frontend/src/platform/theme/persistence.ts`:

```ts
import {
  FRONTEND_THEME_SETTING_KEY,
  fetchApplicationSettings,
  saveApplicationSetting
} from '../../domains/settings/api/settings'
import { defaultThemeSettings, parseThemeSettings, type ThemeSettings } from './settings'

const LOCAL_STORAGE_KEY = 'hermes-theme-settings'

export async function loadPersistedThemeSettings(): Promise<ThemeSettings> {
  try {
    const response = await fetchApplicationSettings()
    const setting = response.items.find((item) => item.setting_key === FRONTEND_THEME_SETTING_KEY)
    if (setting) {
      return parseThemeSettings(setting.value)
    }
  } catch {
    return loadLocalThemeSettings()
  }

  return loadLocalThemeSettings()
}

export async function savePersistedThemeSettings(settings: ThemeSettings): Promise<ThemeSettings> {
  try {
    const saved = await saveApplicationSetting(FRONTEND_THEME_SETTING_KEY, settings)
    const parsed = parseThemeSettings(saved.value)
    saveLocalThemeSettings(parsed)
    return parsed
  } catch {
    saveLocalThemeSettings(settings)
    return settings
  }
}

export function loadLocalThemeSettings(): ThemeSettings {
  try {
    const raw = localStorage.getItem(LOCAL_STORAGE_KEY)
    return raw ? parseThemeSettings(JSON.parse(raw)) : defaultThemeSettings()
  } catch {
    return defaultThemeSettings()
  }
}

function saveLocalThemeSettings(settings: ThemeSettings): void {
  try {
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(settings))
  } catch {
    // localStorage may be unavailable; runtime theme still applies in memory.
  }
}
```

- [ ] **Step 2: Use persistence helper from theme store**

In `frontend/src/shared/stores/theme.ts`, add async actions:

```ts
async function hydrateThemeSettings(): Promise<void> {
  themeSettings.value = await loadPersistedThemeSettings()
}

async function saveThemeSettings(): Promise<void> {
  const next = themeDraft.value ?? themeSettings.value
  themeSettings.value = await savePersistedThemeSettings(next)
  themeDraft.value = null
}
```

Keep existing `updateThemeDraft`, `cancelThemeEditing`, label helpers and `shellThemeClass` public API.

- [ ] **Step 3: Hydrate theme before or during shell mount**

In `frontend/src/app/shell/AppShell.vue`, call theme hydration once:

```ts
import { onMounted } from 'vue'

onMounted(() => {
  void theme.hydrateThemeSettings()
})
```

- [ ] **Step 4: Run targeted checks**

Run:

```sh
cd frontend && pnpm lint:ts
```

Expected: pass. If typecheck reports an import cycle or undefined settings export, create `frontend/src/platform/settings/applicationSettingsClient.ts` with `fetchApplicationSettings`, `saveApplicationSetting` and `FRONTEND_THEME_SETTING_KEY`, then re-export those names from `frontend/src/domains/settings/api/settings.ts` so existing domain imports keep working.

---

## Task 5: Shell Visual Restoration Without Widget System

**Files:**
- Modify: `frontend/src/style.css`
- Modify: `frontend/src/app/shell/AppShell.vue`
- Modify: `frontend/src/app/shell/Sidebar.vue`
- Modify: `frontend/src/app/shell/Topbar.vue`
- Modify: `frontend/src/app/shell/NotificationsDrawer.vue`
- Create: `frontend/src/shared/ui/Surface.vue`

- [ ] **Step 1: Restore viewport guard semantics**

In `frontend/src/app/shell/AppShell.vue`, keep `.viewport-guard` as the class receiving theme and view classes. The rendered structure should remain:

```vue
<div class="viewport-guard" :class="[theme.shellThemeClass, nav.shellViewClass]">
  <div class="desktop-shell" :class="{ 'sidebar-rail': nav.isSidebarRail }">
    <Sidebar />
    <div class="workspace">
      <Topbar />
      <NotificationsDrawer />
      <main class="workspace-content">
        <RouterView />
      </main>
    </div>
  </div>
</div>
```

Remove widget/layout-editor classes from this restoration pass.

- [ ] **Step 2: Restore shell CSS from baseline as tokenized CSS**

In `frontend/src/style.css`, align `.desktop-shell` with the old shell:

```css
.desktop-shell {
  --hh-shell-bottom-inset: 16px;
  --hh-shell-bg-image: none;
  --hh-shell-bg-dim: 0.42;
  --hh-panel-alpha: 0.7;
  --hh-panel-alpha-low: 0.56;
  --hh-panel-blur: 12px;

  position: fixed;
  inset: 0;
  display: grid;
  grid-template-columns: var(--hh-shell-sidebar-width) minmax(var(--hh-shell-content-min-width), 1fr);
  gap: 16px;
  width: 100vw;
  max-width: 100vw;
  height: 100dvh;
  min-height: 0;
  overflow: hidden;
  padding: 0 14px var(--hh-shell-bottom-inset) 0;
  background:
    linear-gradient(rgba(2, 9, 11, var(--hh-shell-bg-dim)), rgba(2, 9, 11, var(--hh-shell-bg-dim))),
    var(--hh-shell-bg-image),
    radial-gradient(circle at 72% 2%, rgba(23, 122, 121, 0.14), transparent 34%),
    linear-gradient(180deg, rgba(7, 28, 32, 0.88), rgba(2, 9, 11, 0.98) 46%),
    var(--hh-color-bg);
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover, cover, auto, auto, auto;
}
```

- [ ] **Step 3: Add shared Surface primitive**

Create `frontend/src/shared/ui/Surface.vue`:

```vue
<script setup lang="ts">
type SurfaceTone = 'panel' | 'raised' | 'deep'

withDefaults(defineProps<{
  as?: string
  tone?: SurfaceTone
}>(), {
  as: 'section',
  tone: 'panel'
})
</script>

<template>
  <component :is="as" class="hh-surface" :class="`hh-surface--${tone}`">
    <slot />
  </component>
</template>
```

Add global CSS:

```css
.hh-surface {
  border: 1px solid var(--hh-border-subtle);
  border-radius: var(--hh-radius-md);
  background: rgba(5, 22, 25, var(--hh-panel-alpha));
  backdrop-filter: blur(var(--hh-panel-blur));
  box-shadow: var(--hh-shadow-panel);
}

.hh-surface--raised {
  background: rgba(8, 29, 33, var(--hh-panel-alpha));
}

.hh-surface--deep {
  background: rgba(4, 18, 21, var(--hh-panel-alpha));
}
```

- [ ] **Step 4: Restore Sidebar logo and density**

Use the restored real asset for the brand mark:

```vue
<img class="sidebar-logo-image" src="/assets/hermes-logo-mark.png" alt="" aria-hidden="true" />
```

Keep the text label accessible as "Hermes / Memory System". Use tokenized colors and spacing.

- [ ] **Step 5: Run shell typecheck**

Run:

```sh
cd frontend && pnpm lint:ts
```

Expected: pass.

---

## Task 6: Validation Gate Upgrade

**Files:**
- Create: `frontend/scripts/check-component-lines.mjs`
- Modify: `frontend/package.json`
- Modify: `frontend/src/__tests__/placeholder.test.ts`

- [ ] **Step 1: Create SRP line-count script**

Create `frontend/scripts/check-component-lines.mjs`:

```js
import { readdir, readFile } from 'node:fs/promises'
import path from 'node:path'

const root = path.resolve('src')
const hardLimit = 700
const reviewLimit = 500

async function collectVueFiles(dir) {
  const entries = await readdir(dir, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...await collectVueFiles(full))
    } else if (entry.isFile() && entry.name.endsWith('.vue')) {
      files.push(full)
    }
  }
  return files
}

const files = await collectVueFiles(root)
const violations = []
const review = []

for (const file of files) {
  const source = await readFile(file, 'utf8')
  const lines = source.split('\n').length
  const relative = path.relative(process.cwd(), file)
  if (lines > hardLimit) {
    violations.push(`${relative}: ${lines} lines exceeds hard limit ${hardLimit}`)
  } else if (lines > reviewLimit) {
    review.push(`${relative}: ${lines} lines exceeds review limit ${reviewLimit}`)
  }
}

if (review.length > 0) {
  console.warn(review.join('\n'))
}

if (violations.length > 0) {
  console.error(violations.join('\n'))
  process.exit(1)
}
```

- [ ] **Step 2: Add scripts**

Modify `frontend/package.json`:

```json
"lint:srp": "node scripts/check-component-lines.mjs",
"lint:styles": "node scripts/check-no-inline-styles.mjs",
"test": "pnpm test:unit",
"validate": "pnpm lint:ts && pnpm lint:styles && pnpm lint:srp && pnpm test:unit && pnpm build"
```

Keep existing `lint`, `lint:ts`, `test:unit`, `build`.

- [ ] **Step 3: Remove placeholder assertions**

Delete `frontend/src/__tests__/placeholder.test.ts` after real tests from Tasks 3 and 4 pass.

- [ ] **Step 4: Run validation and expect SRP failures**

Run:

```sh
cd frontend && pnpm lint:srp
```

Expected: FAIL until `TelegramMessageThread.vue` and `CommunicationsPage.vue` are split. Keep this failure as the red test for SRP tasks.

---

## Task 7: Safe Communications Message Rendering

**Files:**
- Create: `frontend/src/domains/communications/components/message-renderer/sanitizeEmailHtml.ts`
- Create: `frontend/src/domains/communications/components/message-renderer/sanitizeEmailHtml.test.ts`
- Modify: `frontend/src/domains/communications/components/MessageBodyTab.vue`

- [ ] **Step 1: Write sanitizer tests**

Create `sanitizeEmailHtml.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { sanitizeEmailHtml, textToSafeHtml } from './sanitizeEmailHtml'

describe('email body sanitizer', () => {
  it('removes scripts and event handlers', () => {
    const html = sanitizeEmailHtml('<img src="x" onerror="alert(1)"><script>alert(1)</script>')
    expect(html).toContain('<img')
    expect(html).not.toContain('onerror')
    expect(html).not.toContain('<script')
  })

  it('removes javascript urls', () => {
    const html = sanitizeEmailHtml('<a href="javascript:alert(1)">open</a>')
    expect(html).toContain('open')
    expect(html).not.toContain('javascript:')
  })

  it('escapes plain text before adding line breaks', () => {
    expect(textToSafeHtml('<b>x</b>\nnext')).toBe('&lt;b&gt;x&lt;/b&gt;<br>next')
  })
})
```

- [ ] **Step 2: Run sanitizer tests and verify failure**

Run:

```sh
cd frontend && pnpm test:unit -- src/domains/communications/components/message-renderer/sanitizeEmailHtml.test.ts
```

Expected: FAIL because implementation is missing.

- [ ] **Step 3: Implement sanitizer**

Create `sanitizeEmailHtml.ts`:

```ts
const blockedTags = new Set(['script', 'style', 'iframe', 'object', 'embed', 'link', 'meta'])
const allowedUrlProtocols = new Set(['http:', 'https:', 'mailto:', 'tel:'])

export function sanitizeEmailHtml(input: string): string {
  const template = document.createElement('template')
  template.innerHTML = input
  sanitizeNode(template.content)
  return template.innerHTML
}

export function textToSafeHtml(input: string): string {
  return escapeHtml(input).replaceAll('\n', '<br>')
}

function sanitizeNode(node: Node): void {
  for (const child of [...node.childNodes]) {
    if (child.nodeType === Node.ELEMENT_NODE) {
      const element = child as Element
      const tagName = element.tagName.toLowerCase()
      if (blockedTags.has(tagName)) {
        element.remove()
        continue
      }
      sanitizeAttributes(element)
    }
    sanitizeNode(child)
  }
}

function sanitizeAttributes(element: Element): void {
  for (const attribute of [...element.attributes]) {
    const name = attribute.name.toLowerCase()
    const value = attribute.value.trim()
    if (name.startsWith('on') || name === 'srcdoc' || name === 'style') {
      element.removeAttribute(attribute.name)
      continue
    }
    if ((name === 'href' || name === 'src') && !isSafeUrl(value)) {
      element.removeAttribute(attribute.name)
    }
  }
}

function isSafeUrl(value: string): boolean {
  try {
    const parsed = new URL(value, 'https://hermes.local')
    return allowedUrlProtocols.has(parsed.protocol)
  } catch {
    return false
  }
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}
```

- [ ] **Step 4: Use sanitizer in MessageBodyTab**

In `MessageBodyTab.vue`, remove regex-only sanitizer logic and use:

```ts
import { sanitizeEmailHtml, textToSafeHtml } from './message-renderer/sanitizeEmailHtml'
```

When rendering HTML bodies, assign `sanitizeEmailHtml(bodyHtml)` to the isolated body container. When rendering plain text, assign `textToSafeHtml(bodyText)`.

- [ ] **Step 5: Render on mount and message change**

Use `watchEffect` or an explicit watcher so rendered content updates when message/body changes:

```ts
watchEffect(() => {
  renderShadowContent()
})
```

Guard DOM access so it only runs when refs are available.

- [ ] **Step 6: Run tests**

Run:

```sh
cd frontend && pnpm test:unit -- src/domains/communications/components/message-renderer/sanitizeEmailHtml.test.ts
cd frontend && pnpm lint:ts
```

Expected: pass.

---

## Task 8: Appearance Settings Decomposition

**Files:**
- Create: `frontend/src/domains/settings/components/appearance/AppearanceHeader.vue`
- Create: `frontend/src/domains/settings/components/appearance/BackgroundPicker.vue`
- Create: `frontend/src/domains/settings/components/appearance/AccentPicker.vue`
- Create: `frontend/src/domains/settings/components/appearance/ThemeRangeControl.vue`
- Create: `frontend/src/domains/settings/components/appearance/SpacingDensityControl.vue`
- Modify: `frontend/src/domains/settings/components/AppearanceSettings.vue`

- [ ] **Step 1: Create range control**

Create `ThemeRangeControl.vue`:

```vue
<script setup lang="ts">
defineProps<{
  id: string
  label: string
  description: string
  value: number
  min: number
  max: number
  step: number
  unit: string
}>()

defineEmits<{
  change: [value: number]
}>()
</script>

<template>
  <section class="appearance-section">
    <header>
      <div>
        <h3>{{ label }}</h3>
        <p>{{ description }}</p>
      </div>
      <strong>{{ value }}{{ unit }}</strong>
    </header>
    <input
      :id="id"
      type="range"
      :min="min"
      :max="max"
      :step="step"
      :value="value"
      @input="$emit('change', Number(($event.target as HTMLInputElement).value))"
    />
  </section>
</template>
```

- [ ] **Step 2: Create spacing density control**

Create `SpacingDensityControl.vue`:

```vue
<script setup lang="ts">
import type { SpacingDensity } from '../../../../platform/theme/settings'

defineProps<{ value: SpacingDensity }>()
defineEmits<{ change: [value: SpacingDensity] }>()

const options: Array<{ id: SpacingDensity; label: string }> = [
  { id: 'compact', label: 'Compact' },
  { id: 'normal', label: 'Normal' },
  { id: 'comfortable', label: 'Comfortable' }
]
</script>

<template>
  <section class="appearance-section">
    <header>
      <div>
        <h3>Spacing Density</h3>
        <p>Controls padding density across panels and controls.</p>
      </div>
    </header>
    <div class="density-option-grid">
      <button
        v-for="option in options"
        :key="option.id"
        type="button"
        class="density-option-btn"
        :class="{ active: value === option.id }"
        @click="$emit('change', option.id)"
      >
        {{ option.label }}
      </button>
    </div>
  </section>
</template>
```

- [ ] **Step 3: Create background picker**

Create `BackgroundPicker.vue`:

```vue
<script setup lang="ts">
import {
  shellBackgroundIds,
  type ShellBackgroundId
} from '../../../../platform/theme/settings'

defineProps<{ value: ShellBackgroundId }>()
defineEmits<{ change: [value: ShellBackgroundId] }>()

const labels: Record<ShellBackgroundId, string> = {
  none: 'Default',
  'network-mesh': 'Network Mesh',
  'data-stream': 'Data Stream',
  'node-frame': 'Node Frame',
  'eclipse-grid': 'Eclipse Grid',
  'dna-blueprint': 'DNA Blueprint',
  'forest-network': 'Forest Network',
  'forest-stream': 'Forest Stream',
  'knowledge-map': 'Knowledge Map',
  'rune-gold': 'Rune Gold',
  'rune-teal': 'Rune Teal'
}
</script>

<template>
  <section class="appearance-section">
    <header>
      <div>
        <h3>Shell Background</h3>
        <p>Background image for the desktop shell.</p>
      </div>
    </header>
    <div class="background-option-grid">
      <button
        v-for="id in shellBackgroundIds"
        :key="id"
        type="button"
        class="background-option-btn"
        :class="{ active: value === id }"
        :aria-pressed="value === id"
        @click="$emit('change', id)"
      >
        <span class="shell-bg-preview" :class="`shell-bg-${id}`" />
        <span>{{ labels[id] }}</span>
      </button>
    </div>
  </section>
</template>
```

- [ ] **Step 4: Create accent picker**

Create `AccentPicker.vue`:

```vue
<script setup lang="ts">
import { accentColorIds, type AccentColorId } from '../../../../platform/theme/settings'

defineProps<{ value: AccentColorId }>()
defineEmits<{ change: [value: AccentColorId] }>()

const labels: Record<AccentColorId, string> = {
  teal: 'Teal',
  cyan: 'Cyan',
  blue: 'Blue',
  violet: 'Violet',
  amber: 'Amber',
  rose: 'Rose'
}
</script>

<template>
  <section class="appearance-section">
    <header>
      <div>
        <h3>Accent Color</h3>
        <p>Application accent color used for highlights and active elements.</p>
      </div>
    </header>
    <div class="accent-option-grid">
      <button
        v-for="id in accentColorIds"
        :key="id"
        type="button"
        class="accent-option-btn"
        :class="{ active: value === id }"
        :aria-pressed="value === id"
        @click="$emit('change', id)"
      >
        <span class="accent-swatch" :class="`accent-swatch-${id}`" />
        <span>{{ labels[id] }}</span>
      </button>
    </div>
  </section>
</template>
```

- [ ] **Step 5: Reduce AppearanceSettings.vue to composition root**

Modify `AppearanceSettings.vue` to import the new controls and contain only store wiring:

```vue
<BackgroundPicker
  :value="theme.effectiveThemeSettings.shellBackground"
  @change="theme.updateThemeDraft({ shellBackground: $event })"
/>
<ThemeRangeControl
  id="panel-opacity"
  label="Panel Opacity"
  description="Controls panel and card transparency."
  :value="theme.effectiveThemeSettings.panelOpacity"
  :min="40"
  :max="100"
  :step="10"
  unit="%"
  @change="theme.updateThemeDraft({ panelOpacity: $event })"
/>
<SpacingDensityControl
  :value="theme.effectiveThemeSettings.spacingDensity"
  @change="theme.updateThemeDraft({ spacingDensity: $event })"
/>
```

- [ ] **Step 6: Run SRP and type checks**

Run:

```sh
cd frontend && pnpm lint:srp
cd frontend && pnpm lint:ts
```

Expected: `AppearanceSettings.vue` is below 300 lines. `lint:srp` may still fail because Communications and Telegram are not split yet.

---

## Task 9: Sidebar Settings Decomposition

**Files:**
- Create: `frontend/src/domains/settings/components/sidebar/SidebarNavigationList.vue`
- Create: `frontend/src/domains/settings/components/sidebar/SidebarGroupEditor.vue`
- Create: `frontend/src/domains/settings/components/sidebar/SidebarItemEditor.vue`
- Modify: `frontend/src/domains/settings/components/SidebarSettings.vue`

- [ ] **Step 1: Identify responsibilities**

Split the current `SidebarSettings.vue` into:

- `SidebarSettings.vue`: store wiring, save/reset/cancel actions, layout shell.
- `SidebarNavigationList.vue`: renders root items/groups preview.
- `SidebarGroupEditor.vue`: edits one group label/icon/items.
- `SidebarItemEditor.vue`: edits one sidebar item visibility/order controls.

- [ ] **Step 2: Create item editor interface**

Create `SidebarItemEditor.vue`:

```vue
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import type { SidebarItemId } from '../../../../shared/stores/sidebar'

defineProps<{
  itemId: SidebarItemId
  label: string
  icon: string
  hidden: boolean
}>()

defineEmits<{
  toggleHidden: [itemId: SidebarItemId]
  moveUp: [itemId: SidebarItemId]
  moveDown: [itemId: SidebarItemId]
}>()
</script>

<template>
  <div class="sidebar-item-editor">
    <Icon :icon="icon" />
    <span>{{ label }}</span>
    <button type="button" @click="$emit('moveUp', itemId)">Move up</button>
    <button type="button" @click="$emit('moveDown', itemId)">Move down</button>
    <button type="button" @click="$emit('toggleHidden', itemId)">
      {{ hidden ? 'Show' : 'Hide' }}
    </button>
  </div>
</template>
```

- [ ] **Step 3: Create navigation list component**

Create `SidebarNavigationList.vue`:

```vue
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import type { ResolvedSidebarRootEntry, SidebarItemId } from '../../../../shared/stores/sidebar'

defineProps<{
  entries: ResolvedSidebarRootEntry[]
  hiddenItemIds: SidebarItemId[]
}>()

defineEmits<{
  toggleHidden: [itemId: SidebarItemId]
  moveUp: [itemId: SidebarItemId]
  moveDown: [itemId: SidebarItemId]
}>()
</script>

<template>
  <div class="sidebar-navigation-list">
    <template v-for="entry in entries" :key="entry.rootId">
      <div v-if="entry.kind === 'item'" class="sidebar-preview-row">
        <Icon :icon="entry.item.icon" />
        <span>{{ entry.item.label }}</span>
      </div>
      <div v-else class="sidebar-preview-group">
        <header>
          <Icon :icon="entry.group.icon" />
          <strong>{{ entry.group.label }}</strong>
        </header>
        <SidebarItemEditor
          v-for="item in entry.group.items"
          :key="item.itemId"
          :item-id="item.itemId"
          :label="item.label"
          :icon="item.icon"
          :hidden="hiddenItemIds.includes(item.itemId)"
          @toggle-hidden="$emit('toggleHidden', $event)"
          @move-up="$emit('moveUp', $event)"
          @move-down="$emit('moveDown', $event)"
        />
      </div>
    </template>
  </div>
</template>
```

Import `SidebarItemEditor` at the top of this file after creating it:

```ts
import SidebarItemEditor from './SidebarItemEditor.vue'
```

- [ ] **Step 4: Create group editor component**

Create `SidebarGroupEditor.vue`:

```vue
<script setup lang="ts">
import type { SidebarNavGroup, SidebarItemId } from '../../../../shared/stores/sidebar'
import SidebarItemEditor from './SidebarItemEditor.vue'

defineProps<{
  group: SidebarNavGroup
  itemLabels: Record<string, { label: string; icon: string }>
  hiddenItemIds: SidebarItemId[]
}>()

defineEmits<{
  rename: [groupId: string, label: string]
  toggleHidden: [itemId: SidebarItemId]
  moveUp: [itemId: SidebarItemId]
  moveDown: [itemId: SidebarItemId]
}>()
</script>

<template>
  <section class="sidebar-group-editor">
    <label>
      <span>Group label</span>
      <input
        :value="group.label"
        @input="$emit('rename', group.id, ($event.target as HTMLInputElement).value)"
      />
    </label>
    <SidebarItemEditor
      v-for="itemId in group.itemIds"
      :key="itemId"
      :item-id="itemId"
      :label="itemLabels[itemId]?.label ?? itemId"
      :icon="itemLabels[itemId]?.icon ?? 'tabler:circle'"
      :hidden="hiddenItemIds.includes(itemId)"
      @toggle-hidden="$emit('toggleHidden', $event)"
      @move-up="$emit('moveUp', $event)"
      @move-down="$emit('moveDown', $event)"
    />
  </section>
</template>
```

- [ ] **Step 5: Wire SidebarSettings.vue**

Modify `SidebarSettings.vue` so it imports and renders:

```vue
<SidebarNavigationList
  :entries="sidebar.sidebarRootEntries"
  :hidden-item-ids="sidebar.sidebarHiddenNavItems"
  @toggle-hidden="toggleSidebarItemHidden"
  @move-up="moveSidebarItemUp"
  @move-down="moveSidebarItemDown"
/>
<SidebarGroupEditor
  v-for="group in sidebar.effectiveSidebarSettings.groups"
  :key="group.id"
  :group="group"
  :item-labels="sidebarItemLabels"
  :hidden-item-ids="sidebar.sidebarHiddenNavItems"
  @rename="renameSidebarGroup"
  @toggle-hidden="toggleSidebarItemHidden"
  @move-up="moveSidebarItemUp"
  @move-down="moveSidebarItemDown"
/>
```

Keep mutation functions in `SidebarSettings.vue` or in the existing sidebar store. Do not pass the whole Pinia store into child components.

- [ ] **Step 6: Run checks**

Run:

```sh
cd frontend && pnpm lint:ts
cd frontend && pnpm lint:srp
```

Expected: `SidebarSettings.vue` is below 300 lines. `lint:srp` still fails only for remaining unsplit files.

---

## Task 10: Communications Page Decomposition

**Files:**
- Create: `frontend/src/domains/communications/components/CommunicationsActionBar.vue`
- Create: `frontend/src/domains/communications/components/CommunicationsWorkbench.vue`
- Create: `frontend/src/domains/communications/components/CommunicationsListPane.vue`
- Create: `frontend/src/domains/communications/components/CommunicationsDetailPane.vue`
- Create: `frontend/src/domains/communications/components/CommunicationsRailPane.vue`
- Modify: `frontend/src/domains/communications/views/CommunicationsPage.vue`
- Modify: `frontend/src/domains/communications/queries/useCommunicationsQuery.ts`
- Modify: `frontend/src/domains/communications/stores/communications.ts`

- [ ] **Step 1: Write target component contract**

`CommunicationsPage.vue` must only:

- call domain query composables;
- hold transient tab/filter state;
- call mutation handlers;
- pass data and callbacks to child panes.

It must not render mail row details, message body tabs, account setup content, context rail internals or compose drawer internals.

- [ ] **Step 2: Create workbench shell**

Create `CommunicationsWorkbench.vue`:

```vue
<script setup lang="ts">
defineProps<{ isLoading: boolean; hasError: boolean }>()
</script>

<template>
  <section class="communications-workbench">
    <slot name="list" />
    <slot name="detail" />
    <slot name="rail" />
  </section>
</template>
```

CSS:

```css
.communications-workbench {
  display: grid;
  grid-template-columns: 350px minmax(430px, 1fr) 320px;
  gap: var(--hh-layout-gap);
  min-height: var(--hh-widget-workbench-large);
}
```

- [ ] **Step 3: Move list area**

Create `CommunicationsListPane.vue` that wraps `CommunicationsConversationList.vue`, `MailList.vue`, filter tabs and loading/empty/error state. Props must be typed from `domains/communications/types/communications.ts`.

- [ ] **Step 4: Move detail area**

Create `CommunicationsDetailPane.vue` that owns selected message detail composition with `MailViewer.vue` and message tabs. It receives selected message and emits message actions.

- [ ] **Step 5: Move rail area**

Create `CommunicationsRailPane.vue` that composes `CommunicationsContextRail.vue` and `CommunicationsContextInspector.vue`.

- [ ] **Step 6: Reduce page**

Refactor `CommunicationsPage.vue` to under 500 lines before moving to the next task. Target under 300 lines by keeping it as a composition root. It should render:

```vue
<CommunicationsActionBar ... />
<CommunicationsWorkbench :is-loading="isMailListLoading" :has-error="Boolean(mailListError)">
  <template #list>
    <CommunicationsListPane ... />
  </template>
  <template #detail>
    <CommunicationsDetailPane ... />
  </template>
  <template #rail>
    <CommunicationsRailPane ... />
  </template>
</CommunicationsWorkbench>
```

- [ ] **Step 7: Run SRP gate**

Run:

```sh
cd frontend && pnpm lint:srp
```

Expected: `CommunicationsPage.vue` no longer appears over the hard limit.

- [ ] **Step 8: Run functional checks**

Run:

```sh
cd frontend && pnpm lint:ts
cd frontend && pnpm test:unit -- src/domains/communications/components/message-renderer/sanitizeEmailHtml.test.ts
```

Expected: pass.

---

## Task 11: Telegram Thread Decomposition

**Files:**
- Create: `frontend/src/integrations/telegram/components/thread/TelegramThreadHeader.vue`
- Create: `frontend/src/integrations/telegram/components/thread/TelegramMessageList.vue`
- Create: `frontend/src/integrations/telegram/components/thread/TelegramComposer.vue`
- Create: `frontend/src/integrations/telegram/components/thread/TelegramSyncPanel.vue`
- Modify: `frontend/src/integrations/telegram/components/TelegramMessageThread.vue`
- Modify: `frontend/src/integrations/telegram/views/TelegramPage.vue`

- [ ] **Step 1: Define split responsibilities**

Target files:

- `TelegramMessageThread.vue`: composition root for selected chat thread.
- `TelegramThreadHeader.vue`: selected chat title, metadata, actions.
- `TelegramMessageList.vue`: virtual/list rendering of messages.
- `TelegramComposer.vue`: message input, send action, disabled/error states.
- `TelegramSyncPanel.vue`: sync/import controls and status.

- [ ] **Step 2: Create message list component**

Create `TelegramMessageList.vue`:

```vue
<script setup lang="ts">
import type { TelegramMessage } from '../../types/telegram'

defineProps<{
  messages: TelegramMessage[]
  selectedMessageId: string | null
}>()

defineEmits<{
  selectMessage: [messageId: string]
}>()
</script>

<template>
  <div class="telegram-message-list">
    <article
      v-for="message in messages"
      :key="message.message_id"
      class="telegram-message-row"
      :class="{ active: selectedMessageId === message.message_id }"
      @click="$emit('selectMessage', message.message_id)"
    >
      <p>{{ message.text }}</p>
      <time>{{ message.occurred_at ?? message.projected_at }}</time>
    </article>
  </div>
</template>
```

- [ ] **Step 3: Move composer**

Create `TelegramComposer.vue`:

```vue
<script setup lang="ts">
defineProps<{
  draft: string
  disabled: boolean
  error: string | null
}>()

defineEmits<{
  'update:draft': [value: string]
  send: []
}>()
</script>

<template>
  <form class="telegram-composer" @submit.prevent="$emit('send')">
    <textarea
      :value="draft"
      :disabled="disabled"
      rows="2"
      aria-label="Telegram message draft"
      @input="$emit('update:draft', ($event.target as HTMLTextAreaElement).value)"
    />
    <button type="submit" :disabled="disabled || draft.trim().length === 0">
      Send
    </button>
    <p v-if="error" class="telegram-composer-error">{{ error }}</p>
  </form>
</template>
```

- [ ] **Step 4: Reduce thread root**

Refactor `TelegramMessageThread.vue` to compose the four children and keep only cross-child state wiring. It must be under 500 lines before moving to the next task.

- [ ] **Step 5: Run SRP and type checks**

Run:

```sh
cd frontend && pnpm lint:srp
cd frontend && pnpm lint:ts
```

Expected: no hard-limit violation from Telegram thread.

---

## Task 12: TanStack Query Boundary Repair

**Files:**
- Modify: `frontend/src/domains/calendar/queries/useCalendarEventsQuery.ts`
- Modify: `frontend/src/domains/calendar/views/CalendarPage.vue`
- Modify: `frontend/src/domains/tasks/queries/useTasksQuery.ts`
- Modify: `frontend/src/domains/tasks/views/TasksPage.vue`

- [ ] **Step 1: Inspect direct API calls**

Run:

```sh
rg -n "await fetch|ApiClient|fetch[A-Z]|create[A-Z]|search[A-Z]" frontend/src/domains/*/views frontend/src/domains/*/components
```

Expected: identify direct server calls in views/components. Start with Calendar and Tasks because they are known ADR-0093 drifts.

- [ ] **Step 2: Move Calendar server calls into query composables**

In `useCalendarEventsQuery.ts`, add query/mutation composables with stable keys:

```ts
export const calendarQueryKeys = {
  all: ['calendar'] as const,
  sources: () => [...calendarQueryKeys.all, 'sources'] as const,
  weeklyBrief: () => [...calendarQueryKeys.all, 'weekly-brief'] as const,
  search: (query: string) => [...calendarQueryKeys.all, 'search', query] as const
}
```

Use `useQuery` and `useMutation`; invalidation must happen through `useQueryClient()`.

- [ ] **Step 3: Move Task server calls into query composables**

In `useTasksQuery.ts`, add keys for tasks, decisions, obligations and review queues. Mutations invalidate the relevant keys.

- [ ] **Step 4: Thin views**

Update `CalendarPage.vue` and `TasksPage.vue` so they call composables and keep only UI state locally.

- [ ] **Step 5: Run type checks**

Run:

```sh
cd frontend && pnpm lint:ts
```

Expected: pass.

---

## Task 13: Remaining Domain Visual Restoration

**Files:**
- Modify: `frontend/src/domains/home/views/HomePage.vue`
- Modify: `frontend/src/domains/personas/views/PersonsPage.vue`
- Modify: `frontend/src/domains/projects/views/ProjectsPage.vue`
- Modify: `frontend/src/domains/calendar/views/CalendarPage.vue`
- Modify: `frontend/src/domains/documents/views/DocumentsPage.vue`
- Modify: `frontend/src/domains/notes/views/NotesPage.vue`
- Modify: `frontend/src/domains/knowledge/views/KnowledgePage.vue`
- Modify: `frontend/src/domains/review/views/ReviewPage.vue`
- Modify: `frontend/src/domains/agents/views/AgentsPage.vue`
- Modify: `frontend/src/domains/organizations/views/OrganizationsPage.vue`
- Modify: `frontend/src/integrations/whatsapp/views/WhatsAppPage.vue`
- Modify: matching component files under each domain.

- [ ] **Step 1: Inspect old and current files for every remaining domain**

Run this command block:

```sh
for domain in home persons projects calendar documents notes knowledge review agents organizations whatsapp; do
  page_name="$(python3 - <<PY
domain = "$domain"
mapping = {
    "home": "Home",
    "persons": "Persons",
    "projects": "Projects",
    "calendar": "Calendar",
    "documents": "Documents",
    "notes": "Notes",
    "knowledge": "Knowledge",
    "review": "Review",
    "agents": "Agents",
    "organizations": "Organizations",
    "whatsapp": "WhatsApp",
}
print(mapping[domain])
PY
)"
  echo "== $domain =="
  git show "7f1cf42f:frontend/src/lib/pages/$domain/${page_name}Page.svelte" | sed -n '1,260p'
  git show "7f1cf42f:frontend/src/lib/pages/$domain/$domain.css" | sed -n '1,260p' || true
  sed -n '1,260p' "frontend/src/domains/$domain/views/${page_name}Page.vue"
  find "frontend/src/domains/$domain/components" -maxdepth 1 -type f -print | sort
done
```

Expected: output shows old Svelte page evidence, old CSS evidence when the file exists, current Vue page, and current component files for all listed domains.

- [ ] **Step 2: Restore visible block inventory**

For each domain, list baseline blocks in the implementation notes before editing. Example for Home:

```text
Home visible blocks:
- metrics
- focus score
- What's New
- Today's Priorities
- Upcoming
- People You Talked To
- System Status
- Active Projects
```

Do not silently drop a block. If current backend data is unavailable, render a safe loading/empty/error state using realistic structure, not placeholder prose.

- [ ] **Step 3: Apply tokenized surface classes**

Use `Surface.vue` or shared CSS classes for panels. Replace domain-local hardcoded panel colors/padding with:

```css
background: rgba(5, 22, 25, var(--hh-panel-alpha));
backdrop-filter: blur(var(--hh-panel-blur));
padding: var(--hh-space-panel);
border-color: var(--hh-border-subtle);
```

- [ ] **Step 4: Keep SRP limits while restoring**

If a domain view exceeds 300 lines after restoration, split immediately into child components in that domain's `components/` directory.

- [ ] **Step 5: Run checks after each domain**

Run after each restored domain:

```sh
cd frontend && pnpm lint:ts
cd frontend && pnpm lint:srp
```

Expected: typecheck passes; no component exceeds 700 lines.

---

## Task 14: I18n Contract Pass

**Files:**
- Modify: `frontend/src/platform/i18n/index.ts`
- Modify: `frontend/src/platform/i18n/en.json`
- Modify: `frontend/src/platform/i18n/ru.json`
- Modify: all restored Vue components with user-visible strings.

- [ ] **Step 1: Verify current ADR-0077 drift**

Run:

```sh
sed -n '1,120p' docs/adr/ADR-0077-i18n-russian-english.md
sed -n '1,80p' frontend/src/platform/i18n/index.ts
wc -l frontend/src/platform/i18n/en.json frontend/src/platform/i18n/ru.json
```

Expected: current implementation defaults to Russian and ships populated `en.json`, which differs from ADR-0077.

- [ ] **Step 2: Choose ADR-compatible behavior**

Unless a new ADR is written, restore ADR-0077 behavior:

```ts
function loadLocale(): Locale {
  try {
    const stored = localStorage.getItem('hh-locale')
    if (stored === 'ru' || stored === 'en') return stored
  } catch {
    // localStorage unavailable
  }
  return 'en'
}
```

Keep `ru.json` as the Russian translation dictionary. Reduce `en.json` to `{}` only if tests and current UI fallback behavior confirm English string keys work as intended.

- [ ] **Step 3: Wrap restored visible text**

Every new visible string in restored components must go through `t('English string key')` or a verified equivalent composable.

- [ ] **Step 4: Run i18n smoke**

Run:

```sh
cd frontend && pnpm lint:ts
```

Expected: pass.

---

## Task 15: Browser QA And Theme Interaction

**Files:**
- Modify: `frontend/scripts/capture-vue-baseline-screenshots.mjs` only when the screenshot report misses a required route or viewport.
- Production files changed by this task must be limited to defects directly found during browser QA.

- [ ] **Step 1: Start frontend with correct env**

Run from repo root:

```sh
make docker-env
make frontend-dev
```

Expected: Vite starts on `http://127.0.0.1:5174` or the configured `HERMES_FRONTEND_PORT`.

- [ ] **Step 2: Open in browser**

Use the in-app browser or provided browser tool to open:

```text
http://127.0.0.1:5174
```

Expected: app boots without the startup error state when `docker/.env` has `HERMES_LOCAL_API_SECRET`.

- [ ] **Step 3: Capture screenshots**

Run:

```sh
cd frontend && pnpm screenshots:vue http://127.0.0.1:5174
```

Expected: script prints `outputDir` and `reportPath`.

- [ ] **Step 4: Verify theme controls**

In Settings > Appearance:

- change accent color and confirm active nav, focus rings and highlights update;
- change panel opacity and confirm panels/cards become more or less transparent;
- change panel blur and confirm background blur changes;
- change spacing density and confirm panel/control padding changes;
- change shell background and confirm real image assets load.

- [ ] **Step 5: Verify desktop viewports**

Use browser viewport sizes:

- `800 x 600`
- `1600 x 1000`

Expected:

- no horizontal overflow;
- no clipped controls;
- no overlapping text;
- no missing primary shell sections;
- no mobile layout claims.

---

## Task 16: Final Validation Gate

**Files:**
- All changed files.

- [ ] **Step 1: Run diff whitespace check**

Run:

```sh
git diff --check
```

Expected: no whitespace errors.

- [ ] **Step 2: Run frontend validation**

Run:

```sh
cd frontend && pnpm lint:ts
cd frontend && pnpm lint:styles
cd frontend && pnpm lint:srp
cd frontend && pnpm test:unit
cd frontend && pnpm build
cd frontend && pnpm validate
```

Expected: all commands pass.

- [ ] **Step 3: Run root validation**

If only frontend source/package files changed:

```sh
make frontend-check
```

If Makefile, backend settings, backend API contracts or shared repository contracts changed:

```sh
make validate
```

Expected: relevant command passes.

- [ ] **Step 4: Audit line counts**

Run:

```sh
find frontend/src -type f \( -name '*.vue' -o -name '*.ts' \) -print0 | xargs -0 wc -l | sort -nr | sed -n '1,40p'
```

Expected: no `.vue` file over 700 lines. Any `.vue` file over 500 lines has an explicit documented reason or is split before completion.

- [ ] **Step 5: Final status**

Run:

```sh
git status --short
```

Expected: changed files match the restoration scope. No `.superpowers/`, screenshots, temporary browser artifacts or local secrets are staged or committed.

---

## Self-Review Checklist

- Spec coverage:
  - visual baseline `7f1cf42f`: covered by Tasks 1, 2, 5, 13, 15;
  - full interactivity/runtime safety: covered by Tasks 3, 7, 12, 15;
  - no widget system: encoded in Ground Rules and shell/domain tasks;
  - configurable colors/padding/opacity/blur: covered by Tasks 4, 5, 8, 15;
  - SRP limits: covered by Tasks 6, 8, 9, 10, 11, 13, 16;
  - validation: covered by Tasks 1, 3, 4, 6, 7, 15, 16.
- Placeholder scan: no task uses `TBD`, `TODO`, or "implement later".
- Type consistency: theme settings use `ThemeSettings`; API bootstrap uses `FrontendConfig`; auth uses `X-Hermes-Secret`.
- Repository policy: plan avoids requiring actual commits because user has not requested commits.
