# Unified Widget Layout Design

## Purpose

Hermes Hub currently renders each desktop view through hand-built layout markup and CSS grids in `frontend/src/routes/+page.svelte`. The product needs a single internal layout system that can preserve the current UI while making panels and cards manageable as widgets: add, hide, reorder, reset and save.

The goal is not to redesign the product surface. The first implementation must inventory the current views, turn their existing panels, cards, rails, metric strips and static blocks into widget definitions, and reproduce the current visual placement by default. Backend data wiring for newly formalized widgets is a separate future slice.

## Relevant ADRs And Constraints

- `ADR-0003 SvelteKit Frontend`: frontend work stays in the SvelteKit app.
- `ADR-0004 Tauri Desktop Shell`: product surface is desktop packaged by Tauri.
- `ADR-0026 Desktop First Responsive UI`: layout must preserve usability across desktop window sizes.
- `ADR-0031 Temporary Desktop Only UI Scope`: no mobile UI design, implementation or validation.
- `ADR-0054 Application Settings Store`: user-editable non-secret UI settings live in declared `application_settings` rows.

Current stabilization constraints also apply:

- CSS is loaded as `tokens.css` before `app.css`.
- No inline `style=` attributes and no embedded Svelte `<style>` blocks.
- Minimum supported desktop viewport is `800 x 600`; below that, the viewport guard blocks the app.
- Component refactoring is deferred until the layout foundation needs a small component boundary for correctness or until the later component refactor phase.

## Non-Goals

This design does not implement:

- mobile or tablet UI;
- free-form canvas positioning;
- manual resize handles in the first version;
- mouse drag-and-drop dependency in the first version;
- new backend data integrations for static widgets;
- AI-generated layout decisions;
- storing private content, message bodies, credentials or runtime selection state in layout settings;
- broad component extraction as the primary objective.

## Approved Direction

Use frontend-declared layout presets with backend-stored user overrides.

The frontend owns:

- layout archetypes;
- widget registry;
- default view presets and preset versions;
- layout resolver;
- edit-mode rendering behavior.

The backend owns durable persistence of declared non-secret layout settings through `application_settings`. The backend does not need to understand widget semantics in the first version; it stores and validates a typed JSON setting.

## Shell Boundary

The global app shell is outside the widget layout system:

- left sidebar navigation remains system UI;
- topbar/search/actions remain system UI;
- viewport guard remains system UI;
- future footer/system bars can be modeled separately.

The layout system starts below the topbar inside the active view content area.

## Layout Archetypes

All views use one of four approved internal layout archetypes under the same engine.

### Operational Board

Best for:

- `Home`;
- `Settings`;
- `AI Agents`.

Typical zones:

- `hero`;
- `metrics`;
- `main`;
- `rail`;
- `bottom`.

This archetype supports dashboard-like operational surfaces without becoming a free-form dashboard builder.

### Master-Detail Workbench

Best for:

- `Communications`;
- `Contacts`;
- `Documents`;
- `Notes`;
- `Tasks`.

Typical zones:

- `header`;
- `filters`;
- `list`;
- `detail`;
- `rail`.

This archetype preserves fast selection workflows: list or queue on the left, selected object in the center, contextual widgets on the right.

### Entity Workspace

Best for:

- `Projects`;
- future rich person, organization or document profile views.

Typical zones:

- `hero`;
- `metadata`;
- `tabs`;
- `main`;
- `rail`;
- `bottom`.

This archetype makes one entity the center of the workspace while exposing graph, evidence, health and related-object widgets.

### Canvas + Inspector

Best for:

- `Knowledge Graph`;
- `Calendar`;
- `Timeline`.

Typical zones:

- `toolbar`;
- `canvas`;
- `inspector`;
- `bottom`.

This archetype gives spatial surfaces a large work area plus a consistent selected-item inspector.

## Widget Registry

Every current top-level panel, card, rail block, metric strip and static block in `+page.svelte` must be represented as either a visible default widget instance or an explicit catalog-only widget.

Each widget definition should include:

```ts
type WidgetDefinition = {
	id: string;
	title: string;
	viewScope: string[];
	defaultZone: string;
	allowedZones: string[];
	minSize: {
		width: number;
		height: number;
	};
	defaultSizeIntent: 'auto' | 'compact' | 'normal' | 'wide' | 'tall' | 'large';
	priority: number;
	canHide: boolean;
	canAdd: boolean;
	dataMode: 'static' | 'existing_state' | 'api_backed';
};
```

`dataMode` is documentation and validation metadata for this slice. It prevents layout refactoring from becoming accidental backend rewiring.

## Full View Inventory Scope

The implementation must inventory all currently rendered views:

- `Home`;
- `Communications`;
- `Timeline`;
- `Contacts`;
- `Projects`;
- `Tasks`;
- `Calendar`;
- `Documents`;
- `Notes`;
- `Knowledge Graph`;
- `Telegram`;
- `WhatsApp`;
- `AI Agents`;
- `Settings`.

Initial widget groups include the following current surface areas.

### Home

- home metrics;
- focus score;
- What's New;
- Today's Priorities;
- Upcoming;
- People You Talked To;
- System Status;
- Active Projects.

### Communications

- conversation list;
- selected message detail;
- sender/profile panel;
- summary;
- message metadata;
- related projects;
- active tasks;
- scoped Ask AI surface.

### Timeline

- timeline stream;
- filters;
- period summary;
- selected event context rail.

### Contacts

- contacts list;
- contact hero;
- contact information;
- about;
- relationship strength;
- recent interactions;
- active projects;
- AI summary;
- identity review;
- related documents;
- recent notes.

### Projects

- project hero;
- metadata strip;
- project switcher;
- section tabs;
- project summary;
- graph preview;
- timeline;
- recent communications;
- top documents;
- source evidence;
- open promises;
- project health;
- key people;
- related projects.

### Tasks

- task metrics;
- candidate review queue;
- active task list or table;
- AI refresh status;
- task detail or context rail;
- deadline and priority widgets.

### Calendar

- calendar toolbar;
- week grid or calendar canvas;
- event blocks;
- upcoming list or rail;
- source/status widgets.

### Documents

- source cards;
- document list or table;
- document detail or preview;
- processing jobs;
- failed job retry/status;
- related project/person context.

### Notes

- notes list;
- note detail;
- note metadata;
- source/filter widgets;
- related projects/documents.

### Knowledge Graph

- graph canvas;
- node inspector;
- graph summary;
- search results;
- evidence/context widgets.

### Telegram

- chat list;
- message thread;
- account/status rail;
- sync/import controls;
- selected chat metadata.

### WhatsApp

- session/status panel;
- chat/message surface;
- sync controls;
- account/session metadata.

### AI Agents

- runtime metrics;
- agent list;
- selected agent detail;
- run history;
- answer form;
- meeting prep/task extraction panels;
- citations.

### Settings

- settings metrics;
- application settings list/editor;
- accounts list;
- account setup cards;
- account detail/config status;
- security/runtime/status panels.

If a block currently exists in the UI, it must not disappear silently. It must be visible in the default preset or explicitly documented as catalog-only with a reason.

## Presets And Overrides

Each view has a versioned default preset declared in frontend code.

```ts
type LayoutPreset = {
	id: string;
	version: number;
	viewId: string;
	archetype: 'operational_board' | 'master_detail_workbench' | 'entity_workspace' | 'canvas_inspector';
	zones: LayoutZoneDefinition[];
	widgets: LayoutWidgetInstance[];
};
```

User settings store selected preset versions and overrides:

```ts
type LayoutSettings = {
	schemaVersion: 1;
	views: Record<string, ViewLayoutOverride>;
};

type ViewLayoutOverride = {
	presetId: string;
	presetVersion: number;
	hiddenWidgetIds: string[];
	zoneOverrides: Record<string, string>;
	orderOverrides: Record<string, string[]>;
	sizeIntentOverrides: Record<string, 'auto' | 'compact' | 'normal' | 'wide' | 'tall' | 'large'>;
};
```

The resolver applies overrides over the current frontend preset:

```text
widget registry + default preset + persisted overrides -> resolved layout
```

Rules:

- unknown widget IDs in persisted overrides are ignored;
- removed zones fall back to the widget default zone when possible;
- newly introduced default widgets appear in their default placement;
- hidden widgets stay in the Add Widget drawer;
- invalid persisted JSON falls back to defaults and surfaces a non-blocking warning.

## Persistence

Use a declared non-secret application setting, for example `frontend.layout`.

Rules:

- value kind is `json`;
- setting is allowlisted, not arbitrary;
- no secrets or private content are stored;
- no message bodies, document text, selected object IDs, search queries or runtime state are stored;
- Save writes only layout choices: preset selection, hidden widgets, zone moves, order overrides and future size intent.

Save flow:

```text
Edit Layout -> local draft overrides -> Save -> settings update endpoint -> reload resolved layout
```

If backend settings are unavailable, edit mode may still allow a local draft, but Save must fail visibly and not imply persistence.

## Edit Mode UX

Normal mode is passive. Widgets are not draggable or editable during ordinary work.

Edit mode is entered explicitly through a view-level action or command palette action.

Edit mode provides:

- widget frame chrome;
- widget title;
- zone label;
- reorder handle;
- `Move up`;
- `Move down`;
- `Move to zone`;
- `Hide`;
- `Add widget`;
- `Save`;
- `Cancel`;
- `Reset view layout`.

First version behavior:

- no mouse drag-and-drop dependency;
- no manual resize handles;
- keyboard-first reorder semantics;
- scoped Add Widget drawer for the current view;
- drawer shows only compatible widgets grouped by zone/archetype;
- hidden widgets can be re-added from the drawer;
- reset returns the current view to the current default preset version.

## Responsive And Hidden Widgets

The supported minimum remains `800 x 600`.

When a widget cannot meet its declared minimum size in the current resolved zone:

- it is automatically hidden from the rendered grid;
- it does not occupy space;
- it does not create horizontal overflow;
- the view shows a compact notice near the top: `Hidden due to space: X, Y`;
- notice actions should help the user inspect, move or permanently hide the affected widgets.

The notice is required because silent disappearance would make the layout feel broken.

Default presets at `800 x 600` should preserve the same visible blocks as the current stabilized UI. Automatic hiding is for constrained user-customized arrangements, future narrower zones or widgets moved into incompatible zones; it is not a substitute for making the default layout fit.

## Widget Visual States

Each widget instance can have a visual attention state:

```ts
type WidgetHighlightState =
	| 'none'
	| 'border'
	| 'pulse-once'
	| 'pulse-continuous';
```

Rules:

- `border` highlights selected, focused or attention-targeted widgets;
- `pulse-once` is used for short events such as newly added widgets, successful save focus or command-palette navigation;
- `pulse-continuous` is reserved for active processes or unresolved attention states;
- highlight and pulse are implemented with CSS classes in `app.css`;
- states must not change widget dimensions;
- states must not cause layout shift;
- `prefers-reduced-motion` disables pulse animation and uses static border/accent styling instead.

## Implementation Slices

The implementation plan should split the work into these slices.

### 1. Inventory

Create the complete widget inventory and default view preset declarations for all current views. UI behavior should not change in this slice.

### 2. Resolver

Add pure TypeScript layout resolver logic:

```text
registry + presets + overrides -> resolved layout
```

Cover resolver behavior with targeted tests before wiring it into Svelte rendering.

### 3. Render Wrappers

Wrap current top-level blocks in widget frames and zones while preserving normal-mode visual output. This is the first significant frontend risk slice and requires screenshot comparison.

### 4. Edit Mode

Add Add/Hide/Reorder/Save/Cancel/Reset controls. Keep v1 keyboard-first and avoid mouse drag dependency.

### 5. Backend Setting

Declare and persist `frontend.layout` through the existing application settings boundary. Extend backend settings minimally only if the current declared-setting infrastructure cannot represent the JSON setting.

### 6. Responsive Auto-Hide

Implement per-widget minimum size checks, automatic hidden-due-to-space behavior and the top notice.

### 7. Visual States

Add border, pulse-once and pulse-continuous widget visual states with reduced-motion handling.

## Screenshot QA Gate

Visual validation is mandatory.

Before the first visual/layout implementation slice:

- capture baseline screenshots for all 14 views;
- use the current app state and current data;
- capture at `800 x 600`;
- optionally also capture `1600 x 1000` for broader desktop comparison.

After render wrappers/default presets are implemented:

- capture after screenshots for the same views;
- use the same viewport and data conditions;
- compare against baseline screenshots.

Screenshots should be stored outside the repository, for example:

```text
/tmp/hermes-layout-baseline-<timestamp>/
/tmp/hermes-layout-after-<timestamp>/
```

Do not commit screenshot artifacts unless explicitly requested.

Comparison criteria:

- same widgets are present;
- default order and zones match the current UI;
- no horizontal overflow;
- no text/control overlap;
- no unexpected missing blocks;
- edit chrome is absent in normal mode;
- hidden-due-to-space notice appears only when required.

The QA report must list screenshot paths and note any intentional or unresolved differences.

## Validation

Expected validation for implementation:

```sh
cd frontend && pnpm lint:styles
cd frontend && pnpm check
cd frontend && pnpm build
make frontend-check
git diff --check
```

If backend settings declarations or tests change, also run the targeted backend settings tests and the relevant repository Make target.

Browser QA must verify:

- app loads at `http://localhost:5174/`;
- no framework overlay;
- no relevant console warnings/errors;
- all 14 primary views render at `800 x 600`;
- below-minimum viewport guard still appears at `799 x 600` and `800 x 599`;
- edit mode can add, hide, reorder, cancel, reset and save;
- persisted overrides survive reload;
- invalid or unavailable persisted settings fall back safely.

## Acceptance Criteria

The first implementation is accepted when:

- all current views are represented by layout presets;
- all current panels, cards, rails, metric strips and static blocks are inventoried as widgets;
- default presets preserve the current visual placement;
- default presets at `800 x 600` do not hide widgets that are currently visible in the stabilized UI;
- normal mode has no edit chrome;
- edit mode supports Add, Hide, Reorder, Save, Cancel and Reset;
- user overrides persist through declared `application_settings`;
- no secrets or private content are stored in layout settings;
- widgets hidden due to space are listed in a compact top notice;
- widget border and pulse states work without layout shift;
- reduced-motion users do not receive pulse animation;
- `800 x 600` remains stable without horizontal overflow;
- baseline and after screenshots have been captured and compared;
- configured frontend checks pass.
