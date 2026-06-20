# Unified Widget Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a unified internal widget layout system for every current Hermes Hub desktop view while preserving the existing normal-mode UI by default.

**Architecture:** Keep the global sidebar, topbar and viewport guard outside the widget system. Add a frontend-owned layout domain (`registry + presets + overrides -> resolved layout`) and persist user overrides as a declared non-secret `frontend.layout` JSON application setting. Implement edit mode incrementally after baseline screenshots, resolver tests and visual-preserving wrappers are in place.

**Tech Stack:** SvelteKit 2/Svelte 5, TypeScript, CSS in `tokens.css`/`app.css`, pnpm, Vitest for focused frontend layout tests, Rust backend application settings store, Browser QA at `800x600`.

---

## Preflight Rules

- Do not start implementation with unrelated unstaged changes mixed into the same commit.
- The current CSS stabilization work may already be dirty in the worktree. Either commit it first as its own change or implement this plan in a fresh worktree created from a clean base.
- Do not commit `.superpowers/` visual companion artifacts.
- Do not introduce inline `style=` or embedded `<style>` blocks.
- Do not add mouse drag-and-drop or manual resize handles in this version.
- Do not reconnect static widgets to new backend data in this layout slice.

## Target File Structure

Create:

- `frontend/src/lib/layout/types.ts` - layout domain types and constants.
- `frontend/src/lib/layout/registry.ts` - widget definitions for all current view blocks.
- `frontend/src/lib/layout/presets.ts` - versioned default presets for all current views.
- `frontend/src/lib/layout/resolver.ts` - pure resolver that applies overrides to presets.
- `frontend/src/lib/layout/settings.ts` - parse/serialize helpers for `frontend.layout`.
- `frontend/src/lib/layout/index.ts` - public exports for Svelte route usage.
- `frontend/src/lib/layout/resolver.test.ts` - resolver and settings tests.
- `frontend/scripts/capture-layout-screenshots.mjs` - Browser/Playwright screenshot helper for baseline and after evidence.

Modify:

- `frontend/package.json` - add `test:layout` and include it in `check`.
- `frontend/src/lib/api.ts` - add typed helpers for loading and saving `frontend.layout`.
- `frontend/src/routes/+page.svelte` - wire resolved layout state and edit mode controls while keeping current blocks in place.
- `frontend/src/lib/styles/app.css` - add widget frame, zone, edit mode, hidden notice and highlight/pulse classes.
- `backend/src/settings.rs` - declare `frontend.layout`.
- `backend/tests/settings.rs` - add settings declaration/update coverage for `frontend.layout`.
- `frontend/README.md` and `design-qa.md` - document commands and screenshot QA evidence after implementation.

Do not split the whole route into small Svelte components in this plan. Only add small layout helpers or wrappers if the implementation becomes unsafe without them.

---

### Task 1: Clean Implementation Starting Point

**Files:**
- Inspect only: repository root

- [ ] **Step 1: Verify current status**

Run:

```sh
git status --short
```

Expected before starting implementation:

```text
no unrelated dirty files, or only the committed CSS stabilization baseline expected by this branch
```

If unrelated dirty files are present, stop and ask the user whether to commit, stash or move to a fresh worktree. Do not run destructive git commands.

- [ ] **Step 2: Verify frontend baseline checks**

Run:

```sh
cd frontend && pnpm lint:styles
cd frontend && pnpm check
cd frontend && pnpm build
```

Expected:

```text
lint:styles exits 0
svelte-check found 0 errors and 0 warnings
vite build exits 0
```

- [ ] **Step 3: Commit only the baseline if needed**

If CSS stabilization changes are still uncommitted and belong to this work, commit them before layout work:

```sh
git add Makefile design-qa.md frontend/README.md frontend/package.json frontend/src/app.html frontend/src/routes/+layout.svelte frontend/src/routes/+page.svelte frontend/scripts/check-no-inline-styles.mjs frontend/src/lib/styles/tokens.css frontend/src/lib/styles/app.css
git commit -m "feat: stabilize frontend styling contract"
```

Expected:

```text
one commit containing only the styling contract and 800x600 guard work
```

If the files are already committed, skip this step.

---

### Task 2: Add Screenshot Capture Harness

**Files:**
- Create: `frontend/scripts/capture-layout-screenshots.mjs`
- Modify: `frontend/package.json`
- Output outside repo: `/tmp/hermes-layout-baseline-<timestamp>/`, `/tmp/hermes-layout-after-<timestamp>/`

- [ ] **Step 1: Add the screenshot script**

Create `frontend/scripts/capture-layout-screenshots.mjs`:

```js
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { chromium } from 'playwright';

const views = [
	['home', 'Home'],
	['communications', 'Communications'],
	['timeline', 'Timeline'],
	['contacts', 'Contacts'],
	['projects', 'Projects'],
	['tasks', 'Tasks'],
	['calendar', 'Calendar'],
	['documents', 'Documents'],
	['notes', 'Notes'],
	['knowledge-graph', 'Knowledge Graph'],
	['communications', 'telegram', 'Telegram'],
	['communications', 'whatsapp', 'WhatsApp'],
	['ai-agents', 'AI Agents'],
	['settings', 'Settings']
];

const mode = process.argv[2] ?? 'baseline';
if (!['baseline', 'after'].includes(mode)) {
	console.error('Usage: node scripts/capture-layout-screenshots.mjs baseline|after [url]');
	process.exit(1);
}

const url = process.argv[3] ?? 'http://localhost:5174/';
const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
const outputDir = path.join('/tmp', `hermes-layout-${mode}-${timestamp}`);

await mkdir(outputDir, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
const results = [];

page.on('console', (message) => {
	if (['warning', 'error'].includes(message.type())) {
		results.push({ type: 'console', level: message.type(), text: message.text() });
	}
});

await page.goto(url, { waitUntil: 'networkidle' });

for (const [id, label] of views) {
	const button = page.getByRole('button', { name: label, exact: true });
	await button.click();
	await page.waitForTimeout(100);
	const state = await page.evaluate(() => {
		const outliers = [];
		for (const element of document.querySelectorAll('body *')) {
			const rect = element.getBoundingClientRect();
			const style = getComputedStyle(element);
			if (style.display === 'none' || style.visibility === 'hidden' || rect.width === 0 || rect.height === 0) {
				continue;
			}
			if (rect.left < -1 || rect.right > window.innerWidth + 1) {
				outliers.push({
					tag: element.tagName.toLowerCase(),
					className: typeof element.className === 'string' ? element.className : '',
					left: Math.round(rect.left),
					right: Math.round(rect.right),
					text: (element.textContent ?? '').trim().replace(/\s+/g, ' ').slice(0, 80)
				});
			}
			if (outliers.length >= 10) break;
		}
		return {
			h1: document.querySelector('h1')?.textContent?.trim() ?? null,
			bodyScrollWidth: document.body.scrollWidth,
			documentScrollWidth: document.documentElement.scrollWidth,
			guardDisplay: getComputedStyle(document.querySelector('.viewport-guard')).display,
			outliers
		};
	});
	const screenshotPath = path.join(outputDir, `${id}.png`);
	await page.screenshot({ path: screenshotPath, fullPage: false });
	results.push({ type: 'view', id, label, screenshotPath, state });
}

await page.setViewportSize({ width: 799, height: 600 });
await page.waitForTimeout(50);
const widthGuard = await page.evaluate(() => getComputedStyle(document.querySelector('.viewport-guard')).display);

await page.setViewportSize({ width: 800, height: 599 });
await page.waitForTimeout(50);
const heightGuard = await page.evaluate(() => getComputedStyle(document.querySelector('.viewport-guard')).display);

results.push({ type: 'guard', widthGuard, heightGuard });

await writeFile(path.join(outputDir, 'summary.json'), JSON.stringify(results, null, 2));
await browser.close();

console.log(outputDir);
```

- [ ] **Step 2: Add script command**

Modify `frontend/package.json` scripts:

```json
{
	"scripts": {
		"capture:layout": "node scripts/capture-layout-screenshots.mjs"
	}
}
```

Preserve existing scripts. Do not remove `lint:styles`, `check`, or `build`.

- [ ] **Step 3: Verify the harness can run**

With the dev server already running at `http://localhost:5174/`, run:

```sh
cd frontend && pnpm capture:layout baseline http://localhost:5174/
```

Expected:

```text
/tmp/hermes-layout-baseline-<timestamp>
```

Open `/tmp/hermes-layout-baseline-<timestamp>/summary.json` and verify every view has:

```json
"documentScrollWidth": 800,
"bodyScrollWidth": 800,
"outliers": []
```

- [ ] **Step 4: Commit**

```sh
git add frontend/package.json frontend/scripts/capture-layout-screenshots.mjs
git commit -m "test: add layout screenshot harness"
```

---

### Task 3: Add Frontend Layout Test Runner

**Files:**
- Modify: `frontend/package.json`

- [ ] **Step 1: Add Vitest**

Run:

```sh
cd frontend && pnpm add -D vitest
```

Expected:

```text
devDependencies includes vitest
pnpm-lock.yaml is updated if present
```

- [ ] **Step 2: Add layout test script**

Modify `frontend/package.json` scripts:

```json
{
	"scripts": {
		"test:layout": "vitest run src/lib/layout",
		"check": "pnpm lint:styles && pnpm test:layout && svelte-kit sync && svelte-check --tsconfig ./tsconfig.json"
	}
}
```

Preserve existing scripts.

- [ ] **Step 3: Verify no tests exist yet**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected before test files exist:

```text
No test files found
```

This command may exit non-zero until the first test file is added in Task 4.

- [ ] **Step 4: Commit**

```sh
git add frontend/package.json pnpm-lock.yaml
git commit -m "test: add frontend layout test runner"
```

If no root `pnpm-lock.yaml` exists and the lockfile is `frontend/pnpm-lock.yaml`, stage that exact lockfile instead.

---

### Task 4: Define Layout Domain Types

**Files:**
- Create: `frontend/src/lib/layout/types.ts`
- Create: `frontend/src/lib/layout/index.ts`
- Test: `frontend/src/lib/layout/resolver.test.ts`

- [ ] **Step 1: Write failing type/export smoke test**

Create `frontend/src/lib/layout/resolver.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { LAYOUT_SCHEMA_VERSION } from './types';

describe('layout domain exports', () => {
	it('uses schema version 1 for the first persisted layout setting', () => {
		expect(LAYOUT_SCHEMA_VERSION).toBe(1);
	});
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
FAIL src/lib/layout/resolver.test.ts
Cannot find module './types'
```

- [ ] **Step 3: Add domain types**

Create `frontend/src/lib/layout/types.ts`:

```ts
export const LAYOUT_SCHEMA_VERSION = 1;

export const layoutArchetypes = [
	'operational_board',
	'master_detail_workbench',
	'entity_workspace',
	'canvas_inspector'
] as const;

export type LayoutArchetype = (typeof layoutArchetypes)[number];

export const widgetSizeIntents = ['auto', 'compact', 'normal', 'wide', 'tall', 'large'] as const;

export type WidgetSizeIntent = (typeof widgetSizeIntents)[number];

export const widgetHighlightStates = ['none', 'border', 'pulse-once', 'pulse-continuous'] as const;

export type WidgetHighlightState = (typeof widgetHighlightStates)[number];

export type WidgetDataMode = 'static' | 'existing_state' | 'api_backed';

export type LayoutViewId =
	| 'home'
	| 'communications'
	| 'timeline'
	| 'contacts'
	| 'projects'
	| 'tasks'
	| 'calendar'
	| 'documents'
	| 'notes'
	| 'knowledge-graph'
	| 'telegram'
	| 'whatsapp'
	| 'ai-agents'
	| 'settings';

export type WidgetMinimumSize = {
	width: number;
	height: number;
};

export type WidgetDefinition = {
	id: string;
	title: string;
	viewScope: LayoutViewId[];
	defaultZone: string;
	allowedZones: string[];
	minSize: WidgetMinimumSize;
	defaultSizeIntent: WidgetSizeIntent;
	priority: number;
	canHide: boolean;
	canAdd: boolean;
	dataMode: WidgetDataMode;
};

export type LayoutZoneDefinition = {
	id: string;
	title: string;
	minWidth: number;
	minHeight: number;
};

export type LayoutWidgetInstance = {
	widgetId: string;
	zoneId: string;
	order: number;
	sizeIntent: WidgetSizeIntent;
	highlight: WidgetHighlightState;
	visible: boolean;
};

export type LayoutPreset = {
	id: string;
	version: number;
	viewId: LayoutViewId;
	archetype: LayoutArchetype;
	zones: LayoutZoneDefinition[];
	widgets: LayoutWidgetInstance[];
};

export type ViewLayoutOverride = {
	presetId: string;
	presetVersion: number;
	hiddenWidgetIds: string[];
	zoneOverrides: Record<string, string>;
	orderOverrides: Record<string, string[]>;
	sizeIntentOverrides: Partial<Record<string, WidgetSizeIntent>>;
};

export type LayoutSettings = {
	schemaVersion: typeof LAYOUT_SCHEMA_VERSION;
	views: Partial<Record<LayoutViewId, ViewLayoutOverride>>;
};

export type ResolvedWidget = LayoutWidgetInstance & {
	definition: WidgetDefinition;
	isHiddenByUser: boolean;
};

export type ResolvedLayout = {
	preset: LayoutPreset;
	zones: LayoutZoneDefinition[];
	widgetsByZone: Record<string, ResolvedWidget[]>;
	hiddenByUser: ResolvedWidget[];
	ignoredWidgetIds: string[];
};
```

Create `frontend/src/lib/layout/index.ts`:

```ts
export * from './types';
```

- [ ] **Step 4: Run test**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
PASS src/lib/layout/resolver.test.ts
```

- [ ] **Step 5: Run frontend check**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 6: Commit**

```sh
git add frontend/src/lib/layout/types.ts frontend/src/lib/layout/index.ts frontend/src/lib/layout/resolver.test.ts frontend/package.json pnpm-lock.yaml
git commit -m "feat: add widget layout domain types"
```

---

### Task 5: Implement Layout Settings Parser

**Files:**
- Create: `frontend/src/lib/layout/settings.ts`
- Modify: `frontend/src/lib/layout/index.ts`
- Test: `frontend/src/lib/layout/resolver.test.ts`

- [ ] **Step 1: Add failing parser tests**

Append to `frontend/src/lib/layout/resolver.test.ts`:

```ts
import { defaultLayoutSettings, parseLayoutSettings } from './settings';

describe('layout settings parser', () => {
	it('returns defaults for missing or invalid values', () => {
		expect(parseLayoutSettings(null)).toEqual(defaultLayoutSettings());
		expect(parseLayoutSettings({ schemaVersion: 99, views: {} })).toEqual(defaultLayoutSettings());
		expect(parseLayoutSettings('bad')).toEqual(defaultLayoutSettings());
	});

	it('keeps valid view overrides', () => {
		const parsed = parseLayoutSettings({
			schemaVersion: 1,
			views: {
				home: {
					presetId: 'home-default',
					presetVersion: 1,
					hiddenWidgetIds: ['home-system-status'],
					zoneOverrides: { 'home-whats-new': 'rail' },
					orderOverrides: { main: ['home-priorities', 'home-whats-new'] },
					sizeIntentOverrides: { 'home-whats-new': 'wide' }
				}
			}
		});

		expect(parsed.views.home?.hiddenWidgetIds).toEqual(['home-system-status']);
		expect(parsed.views.home?.zoneOverrides['home-whats-new']).toBe('rail');
	});
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
FAIL
Cannot find module './settings'
```

- [ ] **Step 3: Add parser implementation**

Create `frontend/src/lib/layout/settings.ts`:

```ts
import {
	LAYOUT_SCHEMA_VERSION,
	type LayoutSettings,
	type LayoutViewId,
	type ViewLayoutOverride,
	type WidgetSizeIntent,
	widgetSizeIntents
} from './types';

const layoutViewIds = new Set<LayoutViewId>([
	'home',
	'communications',
	'timeline',
	'contacts',
	'projects',
	'tasks',
	'calendar',
	'documents',
	'notes',
	'knowledge-graph',
	'telegram',
	'whatsapp',
	'ai-agents',
	'settings'
]);

const sizeIntentSet = new Set<WidgetSizeIntent>(widgetSizeIntents);

export function defaultLayoutSettings(): LayoutSettings {
	return {
		schemaVersion: LAYOUT_SCHEMA_VERSION,
		views: {}
	};
}

export function parseLayoutSettings(value: unknown): LayoutSettings {
	if (!isRecord(value) || value.schemaVersion !== LAYOUT_SCHEMA_VERSION || !isRecord(value.views)) {
		return defaultLayoutSettings();
	}

	const views: LayoutSettings['views'] = {};
	for (const [viewId, override] of Object.entries(value.views)) {
		if (!layoutViewIds.has(viewId as LayoutViewId)) {
			continue;
		}
		const parsed = parseViewOverride(override);
		if (parsed) {
			views[viewId as LayoutViewId] = parsed;
		}
	}

	return {
		schemaVersion: LAYOUT_SCHEMA_VERSION,
		views
	};
}

function parseViewOverride(value: unknown): ViewLayoutOverride | null {
	if (!isRecord(value) || typeof value.presetId !== 'string' || !Number.isInteger(value.presetVersion)) {
		return null;
	}

	return {
		presetId: value.presetId,
		presetVersion: value.presetVersion,
		hiddenWidgetIds: stringArray(value.hiddenWidgetIds),
		zoneOverrides: stringRecord(value.zoneOverrides),
		orderOverrides: stringArrayRecord(value.orderOverrides),
		sizeIntentOverrides: sizeIntentRecord(value.sizeIntentOverrides)
	};
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function stringArray(value: unknown): string[] {
	return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : [];
}

function stringRecord(value: unknown): Record<string, string> {
	if (!isRecord(value)) {
		return {};
	}
	return Object.fromEntries(
		Object.entries(value).filter((entry): entry is [string, string] => typeof entry[1] === 'string')
	);
}

function stringArrayRecord(value: unknown): Record<string, string[]> {
	if (!isRecord(value)) {
		return {};
	}
	return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, stringArray(item)]));
}

function sizeIntentRecord(value: unknown): Partial<Record<string, WidgetSizeIntent>> {
	if (!isRecord(value)) {
		return {};
	}
	return Object.fromEntries(
		Object.entries(value).filter(
			(entry): entry is [string, WidgetSizeIntent] =>
				typeof entry[1] === 'string' && sizeIntentSet.has(entry[1] as WidgetSizeIntent)
		)
	);
}
```

Modify `frontend/src/lib/layout/index.ts`:

```ts
export * from './settings';
export * from './types';
```

- [ ] **Step 4: Run tests**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
PASS src/lib/layout/resolver.test.ts
```

- [ ] **Step 5: Commit**

```sh
git add frontend/src/lib/layout/settings.ts frontend/src/lib/layout/index.ts frontend/src/lib/layout/resolver.test.ts
git commit -m "feat: parse widget layout settings"
```

---

### Task 6: Implement Resolver

**Files:**
- Create: `frontend/src/lib/layout/resolver.ts`
- Modify: `frontend/src/lib/layout/index.ts`
- Test: `frontend/src/lib/layout/resolver.test.ts`

- [ ] **Step 1: Add failing resolver tests**

Append to `frontend/src/lib/layout/resolver.test.ts`:

```ts
import { resolveLayout } from './resolver';
import type { LayoutPreset, WidgetDefinition } from './types';

const testWidgets: WidgetDefinition[] = [
	{
		id: 'home-whats-new',
		title: "What's New",
		viewScope: ['home'],
		defaultZone: 'main',
		allowedZones: ['main', 'rail'],
		minSize: { width: 260, height: 160 },
		defaultSizeIntent: 'auto',
		priority: 10,
		canHide: true,
		canAdd: true,
		dataMode: 'static'
	},
	{
		id: 'home-priorities',
		title: "Today's Priorities",
		viewScope: ['home'],
		defaultZone: 'main',
		allowedZones: ['main'],
		minSize: { width: 260, height: 160 },
		defaultSizeIntent: 'auto',
		priority: 20,
		canHide: true,
		canAdd: true,
		dataMode: 'static'
	}
];

const testPreset: LayoutPreset = {
	id: 'home-default',
	version: 1,
	viewId: 'home',
	archetype: 'operational_board',
	zones: [
		{ id: 'main', title: 'Main', minWidth: 320, minHeight: 240 },
		{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 }
	],
	widgets: [
		{ widgetId: 'home-whats-new', zoneId: 'main', order: 2, sizeIntent: 'auto', highlight: 'none', visible: true },
		{ widgetId: 'home-priorities', zoneId: 'main', order: 1, sizeIntent: 'auto', highlight: 'none', visible: true }
	]
};

describe('resolveLayout', () => {
	it('sorts widgets by preset order when there are no overrides', () => {
		const resolved = resolveLayout(testPreset, testWidgets, undefined);
		expect(resolved.widgetsByZone.main.map((widget) => widget.widgetId)).toEqual([
			'home-priorities',
			'home-whats-new'
		]);
	});

	it('applies hidden, zone, order and size overrides', () => {
		const resolved = resolveLayout(testPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: ['home-priorities'],
			zoneOverrides: { 'home-whats-new': 'rail' },
			orderOverrides: { rail: ['home-whats-new'] },
			sizeIntentOverrides: { 'home-whats-new': 'wide' }
		});

		expect(resolved.widgetsByZone.main).toEqual([]);
		expect(resolved.widgetsByZone.rail.map((widget) => [widget.widgetId, widget.sizeIntent])).toEqual([
			['home-whats-new', 'wide']
		]);
		expect(resolved.hiddenByUser.map((widget) => widget.widgetId)).toEqual(['home-priorities']);
	});

	it('ignores illegal zone overrides', () => {
		const resolved = resolveLayout(testPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: [],
			zoneOverrides: { 'home-priorities': 'rail' },
			orderOverrides: {},
			sizeIntentOverrides: {}
		});

		expect(resolved.widgetsByZone.main.map((widget) => widget.widgetId)).toContain('home-priorities');
		expect(resolved.widgetsByZone.rail.map((widget) => widget.widgetId)).not.toContain('home-priorities');
	});
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
FAIL
Cannot find module './resolver'
```

- [ ] **Step 3: Add resolver implementation**

Create `frontend/src/lib/layout/resolver.ts`:

```ts
import type {
	LayoutPreset,
	ResolvedLayout,
	ResolvedWidget,
	ViewLayoutOverride,
	WidgetDefinition,
	WidgetSizeIntent
} from './types';

export function resolveLayout(
	preset: LayoutPreset,
	definitions: WidgetDefinition[],
	override: ViewLayoutOverride | undefined
): ResolvedLayout {
	const definitionsById = new Map(definitions.map((definition) => [definition.id, definition]));
	const zoneIds = new Set(preset.zones.map((zone) => zone.id));
	const widgetsByZone = Object.fromEntries(preset.zones.map((zone) => [zone.id, [] as ResolvedWidget[]]));
	const hiddenByUser: ResolvedWidget[] = [];
	const ignoredWidgetIds: string[] = [];
	const hiddenIds = new Set(override?.hiddenWidgetIds ?? []);

	for (const instance of preset.widgets) {
		const definition = definitionsById.get(instance.widgetId);
		if (!definition) {
			ignoredWidgetIds.push(instance.widgetId);
			continue;
		}

		const requestedZone = override?.zoneOverrides[instance.widgetId];
		const zoneId =
			requestedZone && zoneIds.has(requestedZone) && definition.allowedZones.includes(requestedZone)
				? requestedZone
				: instance.zoneId;
		const sizeIntent = safeSizeIntent(override?.sizeIntentOverrides[instance.widgetId], instance.sizeIntent);
		const resolved: ResolvedWidget = {
			...instance,
			zoneId,
			sizeIntent,
			definition,
			isHiddenByUser: hiddenIds.has(instance.widgetId)
		};

		if (resolved.isHiddenByUser || !instance.visible) {
			hiddenByUser.push(resolved);
			continue;
		}

		widgetsByZone[zoneId]?.push(resolved);
	}

	for (const zone of preset.zones) {
		widgetsByZone[zone.id] = sortZoneWidgets(widgetsByZone[zone.id] ?? [], override?.orderOverrides[zone.id]);
	}

	return {
		preset,
		zones: preset.zones,
		widgetsByZone,
		hiddenByUser,
		ignoredWidgetIds
	};
}

function safeSizeIntent(value: WidgetSizeIntent | undefined, fallback: WidgetSizeIntent): WidgetSizeIntent {
	return value ?? fallback;
}

function sortZoneWidgets(widgets: ResolvedWidget[], orderOverride: string[] | undefined): ResolvedWidget[] {
	if (!orderOverride?.length) {
		return [...widgets].sort((left, right) => left.order - right.order);
	}

	const order = new Map(orderOverride.map((widgetId, index) => [widgetId, index]));
	return [...widgets].sort((left, right) => {
		const leftOrder = order.get(left.widgetId) ?? Number.MAX_SAFE_INTEGER;
		const rightOrder = order.get(right.widgetId) ?? Number.MAX_SAFE_INTEGER;
		if (leftOrder !== rightOrder) {
			return leftOrder - rightOrder;
		}
		return left.order - right.order;
	});
}
```

Modify `frontend/src/lib/layout/index.ts`:

```ts
export * from './resolver';
export * from './settings';
export * from './types';
```

- [ ] **Step 4: Run tests**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
PASS src/lib/layout/resolver.test.ts
```

- [ ] **Step 5: Run full frontend check**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 6: Commit**

```sh
git add frontend/src/lib/layout/resolver.ts frontend/src/lib/layout/index.ts frontend/src/lib/layout/resolver.test.ts
git commit -m "feat: resolve widget layout overrides"
```

---

### Task 7: Declare Widget Registry And Presets

**Files:**
- Create: `frontend/src/lib/layout/registry.ts`
- Create: `frontend/src/lib/layout/presets.ts`
- Modify: `frontend/src/lib/layout/index.ts`
- Test: `frontend/src/lib/layout/resolver.test.ts`

- [ ] **Step 1: Add inventory completeness test**

Append to `frontend/src/lib/layout/resolver.test.ts`:

```ts
import { layoutPresets } from './presets';
import { widgetRegistry } from './registry';
import type { LayoutViewId } from './types';

const expectedViews: LayoutViewId[] = [
	'home',
	'communications',
	'timeline',
	'contacts',
	'projects',
	'tasks',
	'calendar',
	'documents',
	'notes',
	'knowledge-graph',
	'telegram',
	'whatsapp',
	'ai-agents',
	'settings'
];

describe('default widget inventory', () => {
	it('declares one preset for every current view', () => {
		expect(layoutPresets.map((preset) => preset.viewId).sort()).toEqual([...expectedViews].sort());
	});

	it('has a widget definition for every preset instance', () => {
		const definitionIds = new Set(widgetRegistry.map((widget) => widget.id));
		const missing = layoutPresets.flatMap((preset) =>
			preset.widgets
				.filter((widget) => !definitionIds.has(widget.widgetId))
				.map((widget) => `${preset.viewId}:${widget.widgetId}`)
		);
		expect(missing).toEqual([]);
	});

	it('keeps all visible default widgets inside allowed zones', () => {
		const definitions = new Map(widgetRegistry.map((widget) => [widget.id, widget]));
		const illegal = layoutPresets.flatMap((preset) =>
			preset.widgets.flatMap((widget) => {
				const definition = definitions.get(widget.widgetId);
				return definition && definition.allowedZones.includes(widget.zoneId)
					? []
					: [`${preset.viewId}:${widget.widgetId}:${widget.zoneId}`];
			})
		);
		expect(illegal).toEqual([]);
	});
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
FAIL
Cannot find module './presets'
```

- [ ] **Step 3: Add registry and presets**

Create `frontend/src/lib/layout/registry.ts` with widget definitions for all current blocks. Start with this exact helper and list:

```ts
import type { LayoutViewId, WidgetDefinition } from './types';

function widget(
	id: string,
	title: string,
	viewScope: LayoutViewId[],
	defaultZone: string,
	allowedZones: string[],
	dataMode: WidgetDefinition['dataMode'] = 'static'
): WidgetDefinition {
	return {
		id,
		title,
		viewScope,
		defaultZone,
		allowedZones,
		minSize: { width: 220, height: 120 },
		defaultSizeIntent: 'auto',
		priority: 100,
		canHide: true,
		canAdd: true,
		dataMode
	};
}

export const widgetRegistry: WidgetDefinition[] = [
	widget('home-metrics', 'Home Metrics', ['home'], 'metrics', ['metrics']),
	widget('home-focus-score', 'Focus Score', ['home'], 'metrics', ['metrics']),
	widget('home-whats-new', "What's New", ['home'], 'main', ['main', 'rail']),
	widget('home-priorities', "Today's Priorities", ['home'], 'main', ['main']),
	widget('home-upcoming', 'Upcoming', ['home'], 'main', ['main', 'rail']),
	widget('home-people-talked-to', 'People You Talked To', ['home'], 'rail', ['rail']),
	widget('home-system-status', 'System Status', ['home'], 'rail', ['rail']),
	widget('home-active-projects', 'Active Projects', ['home'], 'bottom', ['bottom', 'main']),

	widget('communications-conversation-list', 'Conversation List', ['communications'], 'list', ['list']),
	widget('communications-message-detail', 'Message Detail', ['communications'], 'detail', ['detail']),
	widget('communications-sender-profile', 'Sender Profile', ['communications'], 'rail', ['rail']),
	widget('communications-summary', 'Summary', ['communications'], 'rail', ['rail']),
	widget('communications-message-metadata', 'Message Metadata', ['communications'], 'rail', ['rail']),
	widget('communications-related-projects', 'Related Projects', ['communications'], 'rail', ['rail']),
	widget('communications-active-tasks', 'Active Tasks', ['communications'], 'rail', ['rail']),
	widget('communications-ask-ai', 'Ask AI', ['communications'], 'detail', ['detail', 'rail'], 'existing_state'),

	widget('timeline-stream', 'Timeline Stream', ['timeline'], 'canvas', ['canvas']),
	widget('timeline-filters', 'Timeline Filters', ['timeline'], 'toolbar', ['toolbar']),
	widget('timeline-period-summary', 'Period Summary', ['timeline'], 'inspector', ['inspector']),
	widget('timeline-selected-event-context', 'Selected Event Context', ['timeline'], 'inspector', ['inspector']),

	widget('contacts-list', 'Contacts List', ['contacts'], 'list', ['list']),
	widget('contacts-hero', 'Contact Hero', ['contacts'], 'detail', ['detail']),
	widget('contacts-information', 'Contact Information', ['contacts'], 'detail', ['detail', 'rail']),
	widget('contacts-about', 'About', ['contacts'], 'detail', ['detail']),
	widget('contacts-relationship-strength', 'Relationship Strength', ['contacts'], 'detail', ['detail', 'rail']),
	widget('contacts-recent-interactions', 'Recent Interactions', ['contacts'], 'detail', ['detail']),
	widget('contacts-active-projects', 'Active Projects', ['contacts'], 'detail', ['detail', 'rail']),
	widget('contacts-ai-summary', 'AI Summary', ['contacts'], 'rail', ['rail']),
	widget('contacts-identity-review', 'Contact Identity Review', ['contacts'], 'rail', ['rail'], 'api_backed'),
	widget('contacts-related-documents', 'Related Documents', ['contacts'], 'rail', ['rail']),
	widget('contacts-recent-notes', 'Recent Notes', ['contacts'], 'rail', ['rail']),

	widget('projects-hero', 'Project Hero', ['projects'], 'hero', ['hero']),
	widget('projects-metadata-strip', 'Metadata Strip', ['projects'], 'metadata', ['metadata']),
	widget('projects-switcher', 'Project Switcher', ['projects'], 'tabs', ['tabs']),
	widget('projects-section-tabs', 'Section Tabs', ['projects'], 'tabs', ['tabs']),
	widget('projects-summary', 'Project Summary', ['projects'], 'main', ['main']),
	widget('projects-graph-preview', 'Knowledge Graph', ['projects'], 'main', ['main']),
	widget('projects-timeline', 'Project Timeline', ['projects'], 'main', ['main', 'rail']),
	widget('projects-recent-communications', 'Recent Communications', ['projects'], 'main', ['main', 'rail']),
	widget('projects-top-documents', 'Top Documents', ['projects'], 'main', ['main', 'rail']),
	widget('projects-source-evidence', 'Source Evidence', ['projects'], 'main', ['main']),
	widget('projects-open-promises', 'Open Promises', ['projects'], 'main', ['main']),
	widget('projects-health', 'Project Health', ['projects'], 'rail', ['rail']),
	widget('projects-key-people', 'Key People', ['projects'], 'rail', ['rail']),
	widget('projects-related-projects', 'Related Projects', ['projects'], 'rail', ['rail']),

	widget('tasks-metrics', 'Task Metrics', ['tasks'], 'header', ['header']),
	widget('tasks-candidate-review', 'Candidate Review Queue', ['tasks'], 'list', ['list'], 'api_backed'),
	widget('tasks-active-list', 'Active Tasks', ['tasks'], 'detail', ['detail'], 'api_backed'),
	widget('tasks-ai-refresh-status', 'AI Refresh Status', ['tasks'], 'rail', ['rail'], 'api_backed'),
	widget('tasks-context', 'Task Context', ['tasks'], 'rail', ['rail']),
	widget('tasks-deadlines-priority', 'Deadlines And Priority', ['tasks'], 'rail', ['rail']),

	widget('calendar-toolbar', 'Calendar Toolbar', ['calendar'], 'toolbar', ['toolbar']),
	widget('calendar-week-grid', 'Week Grid', ['calendar'], 'canvas', ['canvas']),
	widget('calendar-event-blocks', 'Event Blocks', ['calendar'], 'canvas', ['canvas']),
	widget('calendar-upcoming', 'Upcoming', ['calendar'], 'inspector', ['inspector']),
	widget('calendar-source-status', 'Source Status', ['calendar'], 'inspector', ['inspector']),

	widget('documents-source-cards', 'Source Cards', ['documents'], 'header', ['header']),
	widget('documents-list', 'Documents List', ['documents'], 'list', ['list']),
	widget('documents-detail-preview', 'Document Detail', ['documents'], 'detail', ['detail']),
	widget('documents-processing-jobs', 'Processing Jobs', ['documents'], 'rail', ['rail'], 'api_backed'),
	widget('documents-failed-retry-status', 'Failed Job Retry Status', ['documents'], 'rail', ['rail'], 'api_backed'),
	widget('documents-related-context', 'Related Context', ['documents'], 'rail', ['rail']),

	widget('notes-list', 'Notes List', ['notes'], 'list', ['list']),
	widget('notes-detail', 'Note Detail', ['notes'], 'detail', ['detail']),
	widget('notes-metadata', 'Note Metadata', ['notes'], 'rail', ['rail']),
	widget('notes-source-filters', 'Source Filters', ['notes'], 'header', ['header']),
	widget('notes-related-projects-documents', 'Related Projects And Documents', ['notes'], 'rail', ['rail']),

	widget('knowledge-toolbar', 'Graph Toolbar', ['knowledge-graph'], 'toolbar', ['toolbar']),
	widget('knowledge-graph-canvas', 'Graph Canvas', ['knowledge-graph'], 'canvas', ['canvas'], 'api_backed'),
	widget('knowledge-node-inspector', 'Node Inspector', ['knowledge-graph'], 'inspector', ['inspector'], 'api_backed'),
	widget('knowledge-graph-summary', 'Graph Summary', ['knowledge-graph'], 'inspector', ['inspector'], 'api_backed'),
	widget('knowledge-search-results', 'Search Results', ['knowledge-graph'], 'inspector', ['inspector'], 'api_backed'),
	widget('knowledge-evidence-context', 'Evidence', ['knowledge-graph'], 'inspector', ['inspector'], 'api_backed'),

	widget('telegram-chat-list', 'Telegram Chats', ['communications', 'telegram'], 'list', ['list'], 'api_backed'),
	widget('telegram-message-thread', 'Message Thread', ['communications', 'telegram'], 'detail', ['detail'], 'api_backed'),
	widget('telegram-account-status', 'Account Status', ['communications', 'telegram'], 'rail', ['rail'], 'api_backed'),
	widget('telegram-sync-controls', 'Sync Controls', ['communications', 'telegram'], 'rail', ['rail'], 'api_backed'),
	widget('telegram-selected-chat-metadata', 'Selected Chat Metadata', ['communications', 'telegram'], 'rail', ['rail']),

	widget('whatsapp-session-status', 'Session Status', ['communications', 'whatsapp'], 'header', ['header'], 'api_backed'),
	widget('whatsapp-chat-message-surface', 'Chat Message Surface', ['communications', 'whatsapp'], 'detail', ['detail'], 'api_backed'),
	widget('whatsapp-sync-controls', 'Sync Controls', ['communications', 'whatsapp'], 'rail', ['rail'], 'api_backed'),
	widget('whatsapp-account-session-metadata', 'Account Session Metadata', ['communications', 'whatsapp'], 'rail', ['rail']),

	widget('ai-runtime-metrics', 'Runtime Metrics', ['ai-agents'], 'metrics', ['metrics'], 'api_backed'),
	widget('ai-agent-list', 'Agent List', ['ai-agents'], 'main', ['main'], 'api_backed'),
	widget('ai-selected-agent-detail', 'Selected Agent Detail', ['ai-agents'], 'main', ['main'], 'api_backed'),
	widget('ai-run-history', 'Run History', ['ai-agents'], 'rail', ['rail'], 'api_backed'),
	widget('ai-answer-form', 'Answer Form', ['ai-agents'], 'main', ['main'], 'api_backed'),
	widget('ai-workflow-panels', 'Meeting Prep And Task Extraction', ['ai-agents'], 'rail', ['rail'], 'api_backed'),
	widget('ai-citations', 'Citations', ['ai-agents'], 'rail', ['rail'], 'api_backed'),

	widget('settings-metrics', 'Settings Metrics', ['settings'], 'metrics', ['metrics'], 'api_backed'),
	widget('settings-application-list-editor', 'Application Settings', ['settings'], 'main', ['main'], 'api_backed'),
	widget('settings-accounts-list', 'Accounts List', ['settings'], 'main', ['main'], 'api_backed'),
	widget('settings-account-setup-cards', 'Account Setup', ['settings'], 'rail', ['rail'], 'api_backed'),
	widget('settings-account-detail-status', 'Account Detail Status', ['settings'], 'rail', ['rail'], 'api_backed'),
	widget('settings-security-runtime-status', 'Security And Runtime Status', ['settings'], 'rail', ['rail'], 'api_backed')
];
```

Create `frontend/src/lib/layout/presets.ts` with default zones and preset widgets. Use helper functions to keep declarations readable:

```ts
import type { LayoutPreset, LayoutViewId, LayoutWidgetInstance } from './types';

function instance(widgetId: string, zoneId: string, order: number): LayoutWidgetInstance {
	return {
		widgetId,
		zoneId,
		order,
		sizeIntent: 'auto',
		highlight: 'none',
		visible: true
	};
}

const workbenchZones = [
	{ id: 'header', title: 'Header', minWidth: 560, minHeight: 72 },
	{ id: 'filters', title: 'Filters', minWidth: 560, minHeight: 48 },
	{ id: 'list', title: 'List', minWidth: 220, minHeight: 320 },
	{ id: 'detail', title: 'Detail', minWidth: 320, minHeight: 320 },
	{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 }
];

const boardZones = [
	{ id: 'hero', title: 'Hero', minWidth: 560, minHeight: 72 },
	{ id: 'metrics', title: 'Metrics', minWidth: 560, minHeight: 84 },
	{ id: 'main', title: 'Main', minWidth: 320, minHeight: 320 },
	{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 },
	{ id: 'bottom', title: 'Bottom', minWidth: 560, minHeight: 120 }
];

const entityZones = [
	{ id: 'hero', title: 'Hero', minWidth: 560, minHeight: 96 },
	{ id: 'metadata', title: 'Metadata', minWidth: 560, minHeight: 72 },
	{ id: 'tabs', title: 'Tabs', minWidth: 560, minHeight: 48 },
	{ id: 'main', title: 'Main', minWidth: 320, minHeight: 320 },
	{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 },
	{ id: 'bottom', title: 'Bottom', minWidth: 560, minHeight: 120 }
];

const canvasZones = [
	{ id: 'toolbar', title: 'Toolbar', minWidth: 560, minHeight: 56 },
	{ id: 'canvas', title: 'Canvas', minWidth: 360, minHeight: 360 },
	{ id: 'inspector', title: 'Inspector', minWidth: 220, minHeight: 240 },
	{ id: 'bottom', title: 'Bottom', minWidth: 560, minHeight: 120 }
];

function preset(
	viewId: LayoutViewId,
	archetype: LayoutPreset['archetype'],
	zones: LayoutPreset['zones'],
	widgets: LayoutWidgetInstance[]
): LayoutPreset {
	return {
		id: `${viewId}-default`,
		version: 1,
		viewId,
		archetype,
		zones,
		widgets
	};
}

export const layoutPresets: LayoutPreset[] = [
	preset('home', 'operational_board', boardZones, [
		instance('home-metrics', 'metrics', 10),
		instance('home-focus-score', 'metrics', 20),
		instance('home-whats-new', 'main', 10),
		instance('home-priorities', 'main', 20),
		instance('home-upcoming', 'main', 30),
		instance('home-people-talked-to', 'rail', 10),
		instance('home-system-status', 'rail', 20),
		instance('home-active-projects', 'bottom', 10)
	]),
	preset('communications', 'master_detail_workbench', workbenchZones, [
		instance('communications-conversation-list', 'list', 10),
		instance('communications-message-detail', 'detail', 10),
		instance('communications-ask-ai', 'detail', 20),
		instance('communications-sender-profile', 'rail', 10),
		instance('communications-summary', 'rail', 20),
		instance('communications-message-metadata', 'rail', 30),
		instance('communications-related-projects', 'rail', 40),
		instance('communications-active-tasks', 'rail', 50)
	]),
	preset('timeline', 'canvas_inspector', canvasZones, [
		instance('timeline-filters', 'toolbar', 10),
		instance('timeline-stream', 'canvas', 10),
		instance('timeline-period-summary', 'inspector', 10),
		instance('timeline-selected-event-context', 'inspector', 20)
	]),
	preset('contacts', 'master_detail_workbench', workbenchZones, [
		instance('contacts-list', 'list', 10),
		instance('contacts-hero', 'detail', 10),
		instance('contacts-information', 'detail', 20),
		instance('contacts-about', 'detail', 30),
		instance('contacts-relationship-strength', 'detail', 40),
		instance('contacts-recent-interactions', 'detail', 50),
		instance('contacts-active-projects', 'detail', 60),
		instance('contacts-ai-summary', 'rail', 10),
		instance('contacts-identity-review', 'rail', 20),
		instance('contacts-related-documents', 'rail', 30),
		instance('contacts-recent-notes', 'rail', 40)
	]),
	preset('projects', 'entity_workspace', entityZones, [
		instance('projects-hero', 'hero', 10),
		instance('projects-metadata-strip', 'metadata', 10),
		instance('projects-switcher', 'tabs', 10),
		instance('projects-section-tabs', 'tabs', 20),
		instance('projects-summary', 'main', 10),
		instance('projects-graph-preview', 'main', 20),
		instance('projects-timeline', 'main', 30),
		instance('projects-recent-communications', 'main', 40),
		instance('projects-top-documents', 'main', 50),
		instance('projects-source-evidence', 'main', 60),
		instance('projects-open-promises', 'main', 70),
		instance('projects-health', 'rail', 10),
		instance('projects-key-people', 'rail', 20),
		instance('projects-related-projects', 'rail', 30)
	]),
	preset('tasks', 'master_detail_workbench', workbenchZones, [
		instance('tasks-metrics', 'header', 10),
		instance('tasks-candidate-review', 'list', 10),
		instance('tasks-active-list', 'detail', 10),
		instance('tasks-ai-refresh-status', 'rail', 10),
		instance('tasks-context', 'rail', 20),
		instance('tasks-deadlines-priority', 'rail', 30)
	]),
	preset('calendar', 'canvas_inspector', canvasZones, [
		instance('calendar-toolbar', 'toolbar', 10),
		instance('calendar-week-grid', 'canvas', 10),
		instance('calendar-event-blocks', 'canvas', 20),
		instance('calendar-upcoming', 'inspector', 10),
		instance('calendar-source-status', 'inspector', 20)
	]),
	preset('documents', 'master_detail_workbench', workbenchZones, [
		instance('documents-source-cards', 'header', 10),
		instance('documents-list', 'list', 10),
		instance('documents-detail-preview', 'detail', 10),
		instance('documents-processing-jobs', 'rail', 10),
		instance('documents-failed-retry-status', 'rail', 20),
		instance('documents-related-context', 'rail', 30)
	]),
	preset('notes', 'master_detail_workbench', workbenchZones, [
		instance('notes-source-filters', 'header', 10),
		instance('notes-list', 'list', 10),
		instance('notes-detail', 'detail', 10),
		instance('notes-metadata', 'rail', 10),
		instance('notes-related-projects-documents', 'rail', 20)
	]),
	preset('knowledge-graph', 'canvas_inspector', canvasZones, [
		instance('knowledge-toolbar', 'toolbar', 10),
		instance('knowledge-graph-canvas', 'canvas', 10),
		instance('knowledge-node-inspector', 'inspector', 10),
		instance('knowledge-graph-summary', 'inspector', 20),
		instance('knowledge-search-results', 'inspector', 30),
		instance('knowledge-evidence-context', 'inspector', 40)
	]),
	preset('telegram', 'master_detail_workbench', workbenchZones, [
		instance('telegram-chat-list', 'list', 10),
		instance('telegram-message-thread', 'detail', 10),
		instance('telegram-account-status', 'rail', 10),
		instance('telegram-sync-controls', 'rail', 20),
		instance('telegram-selected-chat-metadata', 'rail', 30)
	]),
	preset('whatsapp', 'master_detail_workbench', workbenchZones, [
		instance('whatsapp-session-status', 'header', 10),
		instance('whatsapp-chat-message-surface', 'detail', 10),
		instance('whatsapp-sync-controls', 'rail', 10),
		instance('whatsapp-account-session-metadata', 'rail', 20)
	]),
	preset('ai-agents', 'operational_board', boardZones, [
		instance('ai-runtime-metrics', 'metrics', 10),
		instance('ai-agent-list', 'main', 10),
		instance('ai-selected-agent-detail', 'main', 20),
		instance('ai-answer-form', 'main', 30),
		instance('ai-run-history', 'rail', 10),
		instance('ai-workflow-panels', 'rail', 20),
		instance('ai-citations', 'rail', 30)
	]),
	preset('settings', 'operational_board', boardZones, [
		instance('settings-metrics', 'metrics', 10),
		instance('settings-application-list-editor', 'main', 10),
		instance('settings-accounts-list', 'main', 20),
		instance('settings-account-setup-cards', 'rail', 10),
		instance('settings-account-detail-status', 'rail', 20),
		instance('settings-security-runtime-status', 'rail', 30)
	])
];

export function findPresetForView(viewId: LayoutViewId): LayoutPreset | null {
	return layoutPresets.find((preset) => preset.viewId === viewId) ?? null;
}
```

Modify `frontend/src/lib/layout/index.ts`:

```ts
export * from './presets';
export * from './registry';
export * from './resolver';
export * from './settings';
export * from './types';
```

- [ ] **Step 4: Run tests**

Run:

```sh
cd frontend && pnpm test:layout
```

Expected:

```text
PASS src/lib/layout/resolver.test.ts
```

- [ ] **Step 5: Commit**

```sh
git add frontend/src/lib/layout/registry.ts frontend/src/lib/layout/presets.ts frontend/src/lib/layout/index.ts frontend/src/lib/layout/resolver.test.ts
git commit -m "feat: inventory current views as layout widgets"
```

---

### Task 8: Add Backend `frontend.layout` Setting

**Files:**
- Modify: `backend/src/settings.rs`
- Modify: `backend/tests/settings.rs`

- [ ] **Step 1: Add failing backend test**

Add to `backend/tests/settings.rs`:

```rust
#[tokio::test]
async fn application_settings_include_frontend_layout_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live frontend layout settings test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    database
        .settings_store()
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = database
        .settings_store()
        .list_settings()
        .await
        .expect("list settings");

    let layout_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.layout")
        .expect("frontend layout setting");

    assert_eq!(layout_setting.category, "frontend");
    assert_eq!(layout_setting.value_kind, "json");
    assert_eq!(layout_setting.value["schemaVersion"], json!(1));
    assert!(layout_setting.value["views"].is_object());
    assert!(layout_setting.is_editable);
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml application_settings_include_frontend_layout_against_postgres
```

Expected with `HERMES_TEST_DATABASE_URL` set:

```text
FAIL
frontend layout setting
```

If `HERMES_TEST_DATABASE_URL` is not set, the test will skip. In that case continue but also run `make backend-test` after implementation.

- [ ] **Step 3: Declare setting**

Add this `DeclaredApplicationSetting` inside `declared_application_settings()` in `backend/src/settings.rs`, after `frontend.actor_id`:

```rust
        DeclaredApplicationSetting {
            setting_key: "frontend.layout",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 1,
                "views": {}
            }),
            label: "Frontend layout",
            description: "Desktop widget layout preset selections and user overrides. Stores layout metadata only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "json",
                "schema_version": 1,
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
```

- [ ] **Step 4: Run backend tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml application_settings_include_frontend_layout_against_postgres
make backend-test
```

Expected:

```text
test passes or skips only when HERMES_TEST_DATABASE_URL is not set
make backend-test exits 0
```

- [ ] **Step 5: Commit**

```sh
git add backend/src/settings.rs backend/tests/settings.rs
git commit -m "feat: declare frontend layout setting"
```

---

### Task 9: Add Frontend API Helpers For Layout Setting

**Files:**
- Modify: `frontend/src/lib/api.ts`
- Test: covered by `pnpm check`

- [ ] **Step 1: Add layout API types and helpers**

Modify `frontend/src/lib/api.ts` by importing layout settings type near existing types:

```ts
import type { LayoutSettings } from '$lib/layout';
```

Add below `ApplicationSettingsResponse`:

```ts
export const FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout';
```

Add near existing settings helpers:

```ts
export function findFrontendLayoutSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_LAYOUT_SETTING_KEY) ?? null;
}

export async function saveFrontendLayoutSetting(
	baseUrl: string,
	token: string,
	actorId: string,
	value: LayoutSettings
): Promise<ApplicationSetting> {
	return saveApplicationSetting(baseUrl, token, actorId, FRONTEND_LAYOUT_SETTING_KEY, value);
}
```

- [ ] **Step 2: Run frontend checks**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 3: Commit**

```sh
git add frontend/src/lib/api.ts
git commit -m "feat: add frontend layout settings API helpers"
```

---

### Task 10: Wire Resolved Layout State Without Rendering Changes

**Files:**
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Import layout helpers**

Modify the `<script lang="ts">` imports in `frontend/src/routes/+page.svelte`:

```ts
import {
	defaultLayoutSettings,
	findPresetForView,
	layoutPresets,
	parseLayoutSettings,
	resolveLayout,
	widgetRegistry,
	type LayoutSettings,
	type ResolvedLayout
} from '$lib/layout';
```

`findPresetForView` is exported from `frontend/src/lib/layout/presets.ts` in Task 7.

- [ ] **Step 2: Add route state**

Add near settings state in `+page.svelte`:

```ts
let layoutSettings = $state<LayoutSettings>(defaultLayoutSettings());
let layoutError = $state('');

const activeLayout = $derived(resolveActiveLayout(currentView, layoutSettings));
```

Add helper function near other utility helpers:

```ts
function resolveActiveLayout(viewId: typeof currentView, settings: LayoutSettings): ResolvedLayout | null {
	const preset = findPresetForView(viewId);
	if (!preset) {
		return null;
	}
	return resolveLayout(preset, widgetRegistry, settings.views[viewId]);
}
```

- [ ] **Step 3: Apply loaded setting**

Inside `loadSettings()`, after `applicationSettings = settingsResponse.items;`, add:

```ts
const frontendLayoutSetting = settingsResponse.items.find(
	(setting) => setting.setting_key === 'frontend.layout'
);
layoutSettings = parseLayoutSettings(frontendLayoutSetting?.value ?? null);
```

In the `catch` branch, add:

```ts
layoutSettings = defaultLayoutSettings();
layoutError = error instanceof Error ? error.message : 'Unknown layout settings error';
```

- [ ] **Step 4: Keep output unchanged**

Do not render `activeLayout` in this task. The only user-visible behavior allowed in this task is unchanged current UI. Svelte and the current TypeScript config do not enable `noUnusedLocals`, so unused derived state is acceptable until wrapper rendering starts.

- [ ] **Step 5: Run checks**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 6: Commit**

```sh
git add frontend/src/routes/+page.svelte frontend/src/lib/layout/presets.ts
git commit -m "feat: load resolved widget layout state"
```

---

### Task 11: Add Widget Frame And Zone CSS

**Files:**
- Modify: `frontend/src/lib/styles/app.css`

- [ ] **Step 1: Add widget frame classes**

Add near existing `.panel` styles in `frontend/src/lib/styles/app.css`:

```css
.layout-zone {
	display: contents;
}

.widget-frame {
	min-width: 0;
	min-height: 0;
}

.widget-frame[data-widget-hidden='true'] {
	display: none;
}

.widget-frame.editing {
	position: relative;
	outline: 1px dashed rgba(45, 240, 206, 0.38);
	outline-offset: 3px;
}

.widget-edit-chrome {
	position: absolute;
	top: 8px;
	right: 8px;
	z-index: 4;
	display: none;
	gap: 6px;
	max-width: calc(100% - 16px);
}

.widget-frame.editing .widget-edit-chrome {
	display: flex;
}

.widget-edit-chrome button {
	display: grid;
	place-items: center;
	min-width: 28px;
	height: 28px;
	border: 1px solid rgba(45, 240, 206, 0.28);
	border-radius: var(--hh-radius-control);
	background: rgba(3, 20, 23, 0.94);
	color: var(--hh-color-text-soft);
}

.widget-hidden-notice {
	display: flex;
	flex-wrap: wrap;
	align-items: center;
	gap: 8px;
	max-width: 100%;
	border: 1px solid rgba(236, 183, 70, 0.24);
	border-radius: var(--hh-radius-control);
	background: rgba(104, 76, 14, 0.18);
	color: var(--hh-color-text-soft);
	padding: 8px 10px;
	font-size: 12px;
}

.widget-highlight-border {
	box-shadow: 0 0 0 1px var(--hh-focus-ring);
}

.widget-highlight-pulse-once,
.widget-highlight-pulse-continuous {
	box-shadow: 0 0 0 1px rgba(45, 240, 206, 0.48);
}

.widget-highlight-pulse-once {
	animation: widget-pulse 900ms ease-out 1;
}

.widget-highlight-pulse-continuous {
	animation: widget-pulse 1600ms ease-in-out infinite;
}

@keyframes widget-pulse {
	0% {
		box-shadow: 0 0 0 1px rgba(45, 240, 206, 0.52), 0 0 0 0 rgba(45, 240, 206, 0.28);
	}
	100% {
		box-shadow: 0 0 0 1px rgba(45, 240, 206, 0.52), 0 0 0 12px rgba(45, 240, 206, 0);
	}
}

@media (prefers-reduced-motion: reduce) {
	.widget-highlight-pulse-once,
	.widget-highlight-pulse-continuous {
		animation: none;
		box-shadow: 0 0 0 1px var(--hh-focus-ring);
	}
}
```

- [ ] **Step 2: Run style guard**

Run:

```sh
cd frontend && pnpm lint:styles
```

Expected:

```text
node scripts/check-no-inline-styles.mjs exits 0
```

- [ ] **Step 3: Commit**

```sh
git add frontend/src/lib/styles/app.css
git commit -m "style: add widget layout frame states"
```

---

### Task 12: Wrap Home View Without Visual Change

**Files:**
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Capture baseline before wrapping**

Run:

```sh
cd frontend && pnpm capture:layout baseline http://localhost:5174/
```

Expected:

```text
/tmp/hermes-layout-baseline-<timestamp>
```

Keep this path for the task final note.

- [ ] **Step 2: Add widget frame helper markup pattern**

In the Home view only, wrap each top-level current block with:

```svelte
<div class="widget-frame" data-widget-id="home-whats-new" data-widget-hidden="false">
</div>
```

Place the exact current block content inside the wrapper; do not rewrite its internals. Apply these wrappers:

- `home-metrics` around the metric grid;
- `home-whats-new` around What's New;
- `home-priorities` around Today's Priorities;
- `home-upcoming` around Upcoming;
- `home-people-talked-to` around People You Talked To;
- `home-system-status` around System Status;
- `home-active-projects` around Active Projects.

Do not wrap the global sidebar or topbar.

- [ ] **Step 3: Run checks**

Run:

```sh
cd frontend && pnpm check
cd frontend && pnpm build
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
vite build exits 0
```

- [ ] **Step 4: Capture after screenshots**

Run:

```sh
cd frontend && pnpm capture:layout after http://localhost:5174/
```

Expected:

```text
/tmp/hermes-layout-after-<timestamp>
```

Compare Home screenshot to baseline:

```sh
open /tmp/hermes-layout-baseline-<timestamp>/home.png
open /tmp/hermes-layout-after-<timestamp>/home.png
```

Expected:

```text
Home has the same widgets in the same visual order; no edit chrome appears
```

- [ ] **Step 5: Commit**

```sh
git add frontend/src/routes/+page.svelte
git commit -m "feat: wrap home widgets without visual change"
```

---

### Task 13: Wrap Remaining Views Without Visual Change

**Files:**
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Wrap remaining current blocks**

For each view, wrap every current top-level panel/card/rail block in `.widget-frame` with matching `data-widget-id`. Use IDs from `frontend/src/lib/layout/registry.ts`.

Required view coverage:

```text
communications
timeline
contacts
projects
tasks
calendar
documents
notes
knowledge-graph
telegram
whatsapp
ai-agents
settings
```

Do not change internal copy, loops, API calls, or static data.

- [ ] **Step 2: Run checks**

Run:

```sh
cd frontend && pnpm check
cd frontend && pnpm build
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
vite build exits 0
```

- [ ] **Step 3: Capture after screenshots**

Run:

```sh
cd frontend && pnpm capture:layout after http://localhost:5174/
```

Expected:

```text
/tmp/hermes-layout-after-<timestamp>
```

Inspect `/tmp/hermes-layout-after-<timestamp>/summary.json`.

Expected for every view:

```json
"documentScrollWidth": 800,
"bodyScrollWidth": 800,
"outliers": []
```

- [ ] **Step 4: Commit**

```sh
git add frontend/src/routes/+page.svelte
git commit -m "feat: wrap current view blocks as widgets"
```

---

### Task 14: Add Edit Mode State And View Controls

**Files:**
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `frontend/src/lib/styles/app.css`

- [ ] **Step 1: Add edit state**

In `+page.svelte`, add:

```ts
let isLayoutEditing = $state(false);
let layoutDraft = $state<LayoutSettings | null>(null);

const effectiveLayoutSettings = $derived(layoutDraft ?? layoutSettings);
```

Change `activeLayout` to use `effectiveLayoutSettings`.

- [ ] **Step 2: Add edit actions**

Add helper functions:

```ts
function startLayoutEditing() {
	layoutDraft = structuredClone(layoutSettings);
	isLayoutEditing = true;
	layoutError = '';
}

function cancelLayoutEditing() {
	layoutDraft = null;
	isLayoutEditing = false;
	layoutError = '';
}

function resetCurrentViewLayout() {
	if (!layoutDraft) {
		layoutDraft = structuredClone(layoutSettings);
	}
	layoutDraft.views[currentView] = undefined;
}
```

- [ ] **Step 3: Add header buttons**

In the view header action area for each view, add one reusable normal-mode button pattern:

```svelte
{#if !isLayoutEditing}
	<button type="button" class="ghost-button" onclick={startLayoutEditing}>
		<Icon icon="tabler:layout-dashboard" width="16" height="16" />
		Edit Layout
	</button>
{:else}
	<button type="button" class="ghost-button" onclick={cancelLayoutEditing}>Cancel</button>
	<button type="button" class="ghost-button" onclick={resetCurrentViewLayout}>Reset</button>
	<button type="button" class="primary-button" disabled>Save</button>
{/if}
```

Keep Save disabled until Task 17.

- [ ] **Step 4: Reflect editing class on frames**

Change widget frame markup:

```svelte
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-whats-new" data-widget-hidden="false">
```

- [ ] **Step 5: Run checks**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 6: Browser smoke**

At `http://localhost:5174/`, click `Edit Layout` on Home.

Expected:

```text
Home shows Cancel, Reset and disabled Save
No widgets move
No horizontal overflow at 800x600
```

- [ ] **Step 7: Commit**

```sh
git add frontend/src/routes/+page.svelte frontend/src/lib/styles/app.css
git commit -m "feat: add widget layout edit mode"
```

---

### Task 15: Add Hide And Reorder Draft Operations

**Files:**
- Modify: `frontend/src/routes/+page.svelte`
- Test: `frontend/src/lib/layout/resolver.test.ts`

- [ ] **Step 1: Add draft mutation helpers**

In `+page.svelte`, add:

```ts
function ensureCurrentViewOverride() {
	const preset = findPresetForView(currentView);
	if (!preset) {
		throw new Error(`No layout preset for ${currentView}`);
	}
	if (!layoutDraft) {
		layoutDraft = structuredClone(layoutSettings);
	}
	layoutDraft.views[currentView] ??= {
		presetId: preset.id,
		presetVersion: preset.version,
		hiddenWidgetIds: [],
		zoneOverrides: {},
		orderOverrides: {},
		sizeIntentOverrides: {}
	};
	return layoutDraft.views[currentView];
}

function hideWidget(widgetId: string) {
	const override = ensureCurrentViewOverride();
	if (!override.hiddenWidgetIds.includes(widgetId)) {
		override.hiddenWidgetIds = [...override.hiddenWidgetIds, widgetId];
	}
}

function moveWidgetInZone(widgetId: string, direction: -1 | 1) {
	const layout = activeLayout;
	if (!layout) return;
	const widget = Object.values(layout.widgetsByZone).flat().find((item) => item.widgetId === widgetId);
	if (!widget) return;
	const zoneWidgets = layout.widgetsByZone[widget.zoneId] ?? [];
	const ids = zoneWidgets.map((item) => item.widgetId);
	const index = ids.indexOf(widgetId);
	const nextIndex = index + direction;
	if (index < 0 || nextIndex < 0 || nextIndex >= ids.length) return;
	const nextIds = [...ids];
	[nextIds[index], nextIds[nextIndex]] = [nextIds[nextIndex], nextIds[index]];
	const override = ensureCurrentViewOverride();
	override.orderOverrides = { ...override.orderOverrides, [widget.zoneId]: nextIds };
}
```

- [ ] **Step 2: Add edit chrome helper**

Add this Svelte snippet near the top of the markup section, before `<svelte:head>`:

```svelte
{#snippet widgetEditChrome(widgetId: string)}
	{#if isLayoutEditing}
		<div class="widget-edit-chrome">
			<button type="button" title="Move widget up" onclick={() => moveWidgetInZone(widgetId, -1)}>
				<Icon icon="tabler:arrow-up" width="14" height="14" />
			</button>
			<button type="button" title="Move widget down" onclick={() => moveWidgetInZone(widgetId, 1)}>
				<Icon icon="tabler:arrow-down" width="14" height="14" />
			</button>
			<button type="button" title="Hide widget" onclick={() => hideWidget(widgetId)}>
				<Icon icon="tabler:eye-off" width="14" height="14" />
			</button>
		</div>
	{/if}
{/snippet}
```

- [ ] **Step 3: Add edit chrome to wrapped widgets**

Inside each `.widget-frame`, render the helper with the same literal ID used by the frame's `data-widget-id`:

```svelte
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-whats-new" data-widget-hidden={!isWidgetVisible('home-whats-new')}>
	{@render widgetEditChrome('home-whats-new')}
</div>
```

For example, the `data-widget-id="projects-health"` frame must contain:

```svelte
{@render widgetEditChrome('projects-health')}
```

Do not use dynamic inline styles.

- [ ] **Step 4: Hide frames based on resolved layout**

Add helper:

```ts
function isWidgetVisible(widgetId: string) {
	if (!activeLayout) return true;
	return Object.values(activeLayout.widgetsByZone).some((widgets) =>
		widgets.some((widget) => widget.widgetId === widgetId)
	);
}
```

Change frame attributes:

```svelte
data-widget-hidden={!isWidgetVisible('home-whats-new')}
```

- [ ] **Step 5: Run checks**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 6: Browser smoke**

At `800x600`, enter edit mode, hide one Home widget, cancel.

Expected:

```text
Widget disappears after Hide
Cancel restores it
No horizontal overflow
```

- [ ] **Step 7: Commit**

```sh
git add frontend/src/routes/+page.svelte
git commit -m "feat: add widget hide and reorder drafts"
```

---

### Task 16: Add Scoped Add Widget Drawer

**Files:**
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `frontend/src/lib/styles/app.css`

- [ ] **Step 1: Add drawer state and helpers**

In `+page.svelte`, add:

```ts
let isWidgetDrawerOpen = $state(false);

const addableWidgetsForCurrentView = $derived(
	widgetRegistry.filter((widget) => widget.viewScope.includes(currentView) && widget.canAdd)
);

function showWidget(widgetId: string) {
	const override = ensureCurrentViewOverride();
	override.hiddenWidgetIds = override.hiddenWidgetIds.filter((id) => id !== widgetId);
	isWidgetDrawerOpen = false;
}
```

- [ ] **Step 2: Add drawer markup near the end of `<main>`**

Add before `</main>`:

```svelte
{#if isLayoutEditing && isWidgetDrawerOpen}
	<div class="widget-drawer" role="dialog" aria-label="Add widget">
		<header>
			<h2>Add widget</h2>
			<button type="button" class="icon-button" onclick={() => (isWidgetDrawerOpen = false)} title="Close add widget drawer">
				<Icon icon="tabler:x" width="16" height="16" />
			</button>
		</header>
		<div class="widget-drawer-list">
			{#each addableWidgetsForCurrentView as widget}
				<button type="button" onclick={() => showWidget(widget.id)}>
					<strong>{widget.title}</strong>
					<span>{widget.defaultZone}</span>
				</button>
			{/each}
		</div>
	</div>
{/if}
```

- [ ] **Step 3: Wire Add Widget button**

Change edit header controls:

```svelte
<button type="button" class="ghost-button" onclick={() => (isWidgetDrawerOpen = true)}>Add widget</button>
```

- [ ] **Step 4: Add drawer CSS**

Add to `app.css`:

```css
.widget-drawer {
	position: fixed;
	top: 72px;
	right: 18px;
	z-index: 80;
	display: grid;
	gap: 12px;
	width: min(320px, calc(100vw - 36px));
	max-height: calc(100dvh - 96px);
	overflow: auto;
	border: 1px solid var(--hh-border-accent-soft);
	border-radius: var(--hh-radius-md);
	background: rgba(4, 18, 21, 0.98);
	box-shadow: var(--hh-shadow-modal);
	padding: 14px;
}

.widget-drawer header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 10px;
}

.widget-drawer-list {
	display: grid;
	gap: 8px;
}

.widget-drawer-list button {
	display: grid;
	gap: 4px;
	width: 100%;
	border: 1px solid rgba(111, 205, 195, 0.14);
	border-radius: var(--hh-radius-control);
	background: rgba(8, 29, 33, 0.94);
	color: var(--hh-color-text-soft);
	padding: 10px;
	text-align: left;
}

.widget-drawer-list span {
	color: var(--hh-color-text-muted);
	font-size: 11px;
}
```

- [ ] **Step 5: Run checks and smoke**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

Browser expected:

```text
Edit Layout -> Add widget opens drawer
Drawer lists current-view widgets only
Close button closes drawer
```

- [ ] **Step 6: Commit**

```sh
git add frontend/src/routes/+page.svelte frontend/src/lib/styles/app.css
git commit -m "feat: add scoped widget drawer"
```

---

### Task 17: Save Layout Overrides

**Files:**
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `frontend/src/lib/api.ts`

- [ ] **Step 1: Add save helper**

In `+page.svelte`, import:

```ts
saveFrontendLayoutSetting
```

from `$lib/api`.

Add:

```ts
let isLayoutSaving = $state(false);

async function saveLayoutDraft() {
	if (!layoutDraft) return;
	isLayoutSaving = true;
	layoutError = '';
	try {
		const updated = await saveFrontendLayoutSetting(apiBaseUrl, apiToken, actorId, layoutDraft);
		layoutSettings = parseLayoutSettings(updated.value);
		layoutDraft = null;
		isLayoutEditing = false;
		settingsActionMessage = 'Layout saved';
	} catch (error) {
		layoutError = error instanceof Error ? error.message : 'Unknown layout save error';
	} finally {
		isLayoutSaving = false;
	}
}
```

- [ ] **Step 2: Enable Save button**

Change Save button:

```svelte
<button type="button" class="primary-button" disabled={isLayoutSaving || !layoutDraft} onclick={() => void saveLayoutDraft()}>
	{isLayoutSaving ? 'Saving...' : 'Save'}
</button>
```

Render `layoutError` near other view errors:

```svelte
{#if layoutError}
	<p class="inline-error">{layoutError}</p>
{/if}
```

- [ ] **Step 3: Run checks**

Run:

```sh
cd frontend && pnpm check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 4: Browser smoke with backend**

With backend running and `frontend.layout` declared, run:

```sh
make backend-run-dev
```

In the browser:

```text
Open Home -> Edit Layout -> Hide System Status -> Save -> reload page
```

Expected:

```text
System Status remains hidden after reload
Settings API audit records the setting update without storing the setting value
```

- [ ] **Step 5: Commit**

```sh
git add frontend/src/routes/+page.svelte frontend/src/lib/api.ts
git commit -m "feat: persist widget layout overrides"
```

---

### Task 18: Add Hidden-Due-To-Space Notice

**Files:**
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `frontend/src/lib/styles/app.css`

- [ ] **Step 1: Add hidden-space state**

In `+page.svelte`, add:

```ts
let widgetsHiddenDueToSpace = $state<string[]>([]);

function hiddenDueToSpaceLabels() {
	return widgetsHiddenDueToSpace
		.map((widgetId) => widgetRegistry.find((widget) => widget.id === widgetId)?.title ?? widgetId)
		.join(', ');
}
```

Add deterministic measurement after render:

```ts
$effect(() => {
	if (!activeLayout) {
		widgetsHiddenDueToSpace = [];
		return;
	}

	const nextHidden: string[] = [];
	for (const widget of Object.values(activeLayout.widgetsByZone).flat()) {
		const element = document.querySelector<HTMLElement>(`[data-widget-id="${widget.widgetId}"]`);
		if (!element) {
			continue;
		}
		const rect = element.getBoundingClientRect();
		if (
			rect.width > 0 &&
			rect.height > 0 &&
			(rect.width < widget.definition.minSize.width || rect.height < widget.definition.minSize.height)
		) {
			nextHidden.push(widget.widgetId);
		}
	}
	widgetsHiddenDueToSpace = nextHidden;
});
```

Default presets at `800x600` must produce an empty `widgetsHiddenDueToSpace` array. If this effect hides current default widgets at `800x600`, fix the layout or widget `minSize`; do not accept the hiding as success.

- [ ] **Step 2: Add notice markup**

Render near the top of each active view content:

```svelte
{#if widgetsHiddenDueToSpace.length > 0}
	<div class="widget-hidden-notice">
		<strong>Hidden due to space:</strong>
		<span>{hiddenDueToSpaceLabels()}</span>
		<button type="button" class="link-button" onclick={() => (isWidgetDrawerOpen = true)}>Review widgets</button>
	</div>
{/if}
```

- [ ] **Step 3: Run `800x600` screenshot QA**

Run:

```sh
cd frontend && pnpm capture:layout after http://localhost:5174/
```

Expected in `summary.json`:

```json
"outliers": []
```

At default `800x600`, expected:

```text
widgetsHiddenDueToSpace is empty for default presets
```

- [ ] **Step 4: Commit**

```sh
git add frontend/src/routes/+page.svelte frontend/src/lib/styles/app.css
git commit -m "feat: surface widgets hidden due to space"
```

---

### Task 19: Complete Widget Highlight And Pulse States

**Files:**
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `frontend/src/lib/styles/app.css`

- [ ] **Step 1: Add highlight class helper**

In `+page.svelte`, add:

```ts
function widgetHighlightClass(widgetId: string) {
	const widget = activeLayout
		? Object.values(activeLayout.widgetsByZone).flat().find((item) => item.widgetId === widgetId)
		: null;
	if (!widget || widget.highlight === 'none') {
		return '';
	}
	return `widget-highlight-${widget.highlight}`;
}
```

- [ ] **Step 2: Apply to frames**

For each widget frame:

```svelte
<div
	class="widget-frame {widgetHighlightClass('home-whats-new')}"
	class:editing={isLayoutEditing}
	data-widget-id="home-whats-new"
	data-widget-hidden={!isWidgetVisible('home-whats-new')}
>
```

- [ ] **Step 3: Add single-pulse on newly shown widget**

When `showWidget(widgetId)` runs, set a temporary highlight override in route state:

```ts
let pulsingWidgetId = $state<string | null>(null);

function triggerWidgetPulse(widgetId: string) {
	pulsingWidgetId = widgetId;
	window.setTimeout(() => {
		if (pulsingWidgetId === widgetId) {
			pulsingWidgetId = null;
		}
	}, 900);
}
```

Then call `triggerWidgetPulse(widgetId)` inside `showWidget`.

Update `widgetHighlightClass`:

```ts
if (pulsingWidgetId === widgetId) {
	return 'widget-highlight-pulse-once';
}
```

- [ ] **Step 4: Run checks**

Run:

```sh
cd frontend && pnpm check
cd frontend && pnpm lint:styles
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
lint:styles exits 0
```

- [ ] **Step 5: Commit**

```sh
git add frontend/src/routes/+page.svelte frontend/src/lib/styles/app.css
git commit -m "feat: add widget highlight and pulse states"
```

---

### Task 20: Full Visual QA And Documentation

**Files:**
- Modify: `frontend/README.md`
- Modify: `design-qa.md`

- [ ] **Step 1: Capture final screenshots**

Run:

```sh
cd frontend && pnpm capture:layout after http://localhost:5174/
```

Expected:

```text
/tmp/hermes-layout-after-<timestamp>
```

- [ ] **Step 2: Run validation**

Run:

```sh
cd frontend && pnpm lint:styles
cd frontend && pnpm test:layout
cd frontend && pnpm check
cd frontend && pnpm build
make frontend-check
git diff --check
```

Expected:

```text
all commands exit 0
```

If backend settings changed in this implementation, also run:

```sh
make backend-test
```

Expected:

```text
backend tests exit 0
```

- [ ] **Step 3: Update docs**

Update `frontend/README.md` with:

```md
## Widget Layout

The desktop workspace uses frontend-declared widget presets and backend-stored user overrides in the declared `frontend.layout` application setting. The global sidebar, topbar and viewport guard are outside the widget layout system.

Validate layout work with:

```sh
pnpm test:layout
pnpm capture:layout baseline http://localhost:5174/
pnpm capture:layout after http://localhost:5174/
```
```

Update `design-qa.md` with:

```md
unified widget layout QA:
- baseline screenshots: `/tmp/hermes-layout-baseline-<timestamp>/`
- after screenshots: `/tmp/hermes-layout-after-<timestamp>/`
- viewport: `800x600`
- result: all 14 views render without horizontal outliers
```

- [ ] **Step 4: Commit**

```sh
git add frontend/README.md design-qa.md
git commit -m "docs: document widget layout QA"
```

---

### Task 21: Final Verification And Branch Finish

**Files:**
- Inspect all changed files

- [ ] **Step 1: Run final validation**

Run:

```sh
make frontend-check
cd frontend && pnpm build
git status --short
```

Expected:

```text
make frontend-check exits 0
pnpm build exits 0
git status shows no uncommitted implementation changes except intentional local artifacts outside git
```

- [ ] **Step 2: Confirm no forbidden CSS**

Run:

```sh
rg -n "<style|\\sstyle=" frontend/src frontend/static frontend/src-tauri
```

Expected:

```text
no matches
```

`rg` exits 1 when there are no matches; that is acceptable.

- [ ] **Step 3: Summarize screenshot evidence**

Record final paths:

```text
baseline: /tmp/hermes-layout-baseline-<timestamp>/
after: /tmp/hermes-layout-after-<timestamp>/
```

State whether every view passed:

```text
Home, Communications, Timeline, Contacts, Projects, Tasks, Calendar, Documents, Notes, Knowledge Graph, Telegram, WhatsApp, AI Agents, Settings: PASS
```

- [ ] **Step 4: Final commit if docs changed after QA**

If Step 3 required documentation changes:

```sh
git add design-qa.md frontend/README.md
git commit -m "docs: record widget layout verification"
```

If no files changed, skip.

---

## Self-Review Checklist For Implementers

Before marking the feature complete, verify:

- Every view in the spec has a preset.
- Every current top-level visible block is represented in `widgetRegistry`.
- Default `800x600` screenshots do not lose widgets that were visible before the layout migration.
- Hidden-due-to-space is not used to hide default visible widgets at `800x600`.
- Edit mode chrome never appears in normal mode.
- Save stores only `frontend.layout` metadata, not content or secrets.
- `pnpm lint:styles` catches any accidental inline styles.
- `prefers-reduced-motion` disables pulse animation.
- Baseline and after screenshot directories are included in the final report.
