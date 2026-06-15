# Vue 3 UI Restoration Design

## Purpose

Restore the Hermes Hub interface after the Vue 3 migration without reverting the
ADR-0093 frontend architecture. The visual baseline is Git commit `7f1cf42f`
(`before Vue3`). The restored UI must preserve the existing product surface,
work as a full interactive desktop application, and avoid single-responsibility
regressions.

This is not a redesign. It is a restoration and stabilization program.

## Approved Direction

Use a restoration kernel plus domain slices.

First restore shared foundations that affect every screen:

- app shell and viewport behavior;
- design tokens and runtime theme settings;
- shell assets and theme classes;
- API bootstrap and auth configuration;
- validation coverage.

Then restore each domain screen against the `7f1cf42f` baseline while keeping
Vue files focused and small.

## Relevant ADRs And Repository Constraints

- `ADR-0004`: Hermes uses a Tauri desktop shell.
- `ADR-0026`: UI is desktop-first and must remain usable across desktop window
  sizes.
- `ADR-0031`: mobile UI design, implementation and validation are out of scope.
- `ADR-0054`: user-editable non-secret runtime/UI settings belong in
  `application_settings`.
- `ADR-0056`: protected local APIs use router-level `X-Hermes-Secret`; do not
  reintroduce bearer tokens or `X-Hermes-Actor-Id`.
- `ADR-0077`: UI text is localized through the Russian/English dictionary
  boundary.
- `ADR-0093`: frontend platform is Vue 3 + TypeScript + Vite + Tauri 2, with
  domain-driven source layout, Pinia for transient UI state and TanStack Query
  for server-derived state.

## Non-Goals

This restoration does not introduce:

- a widget system or layout builder;
- mobile or tablet UI support;
- new product concepts or new routes;
- a broad backend domain expansion;
- provider adapter work;
- AI runtime redesign;
- visual redesign beyond matching the existing Hermes interface.

## Architecture

Keep the current Vue source layout:

```text
frontend/src/
├── app/
├── domains/
├── shared/
└── platform/
```

Domain views are composition roots. They assemble focused child components for
panels, rails, lists, detail panes, drawers, tabs and toolbars. Server-derived
state flows through TanStack Query composables. Pinia stores hold only transient
UI state such as active tabs, selected local controls, drawer state and draft
settings.

Shared components are allowed only when reuse is real across at least two
domains or when a shared primitive enforces a cross-cutting visual/system rule.

## SRP Gates

The following limits are hard acceptance criteria:

- `300+` lines: warning; review whether the component is mixing responsibilities.
- `500+` lines: must be decomposed before merge unless explicitly justified in
  the implementation plan.
- `700+` lines: not allowed.

Implementation must prioritize splitting current oversized Vue files while
restoring screens. Known high-risk areas include:

- Communications page and message rendering;
- Telegram message thread;
- Settings sidebar and appearance controls;
- any page that combines several panels and domain workflows in one SFC.

## Theme And Tokens

Theme is the single customization surface for visual primitives.

Runtime CSS custom properties are the source of truth:

- `--hh-color-*`;
- `--hh-space-*`;
- `--hh-radius-*`;
- `--hh-panel-alpha`;
- `--hh-panel-blur`;
- shell background and accent variables.

Typed TypeScript token definitions mirror allowlisted values for UI controls and
tests. Components consume tokens through CSS variables or shared primitives. New
domain code must not introduce hardcoded colors, opacity, padding, radii or
fallback palettes.

Appearance settings must control:

- shell background;
- background brightness;
- accent color;
- panel opacity;
- panel blur;
- spacing density or padding scale.

The old shell background assets from `7f1cf42f` are restored as real static
assets and referenced by theme classes. Theme settings are schema-versioned,
validated and persisted through the existing non-secret application settings
boundary when available. Local storage fallback is acceptable only as a degraded
mode when backend settings are unavailable, and the UI must not imply durable
backend persistence in that state.

## Runtime Safety

The restoration includes runtime correctness fixes from the Vue migration.

Required behavior:

- `ApiClient` is initialized during bootstrap before route-driven domain queries
  can run.
- Frontend env names match the repository Makefile and backend contract:
  `VITE_HERMES_API_BASE_URL` and `VITE_HERMES_LOCAL_API_SECRET`.
- The default backend URL is `http://127.0.0.1:8080`.
- Missing local API secret fails visibly instead of silently sending an empty
  `X-Hermes-Secret`.
- Protected requests use `X-Hermes-Secret` per `ADR-0056`.
- Server-derived data is loaded through TanStack Query composables, not direct
  API calls inside view components.
- Untrusted imported message bodies are rendered through a safe boundary; no
  regex-only sanitizer plus `innerHTML` path is accepted for email HTML or plain
  text.

## Screen Restoration Flow

Each screen is restored as a domain slice against `7f1cf42f`.

For every slice:

1. Inspect the old Svelte page, CSS and related assets.
2. Inspect the current Vue page, components, API/query/store files and tests.
3. Identify missing visible blocks, broken interactions, token leaks and runtime
   failures.
4. Restore the screen using focused Vue components and existing domain
   boundaries.
5. Preserve visible information architecture: app shell, sidebar groups, topbar,
   dense panels, communications workbench, settings sections, graph, calendar
   and timeline surfaces.
6. Compare screenshots at the same desktop viewport and fix mismatches.

Desktop validation targets:

- minimum supported viewport: `800 x 600`;
- at least one wider desktop viewport for layout density and shell treatment.

## Validation

Implementation validation must include:

```sh
git diff --check
cd frontend && pnpm lint:ts
cd frontend && pnpm test:unit
cd frontend && pnpm build
make frontend-check
```

Run broader repository validation, such as `make validate`, when backend,
settings, Makefile or shared contracts change.

Browser QA must verify:

- app boots without runtime crash;
- protected API requests include `X-Hermes-Secret`;
- restored screens render at `800 x 600`;
- restored screens render at a wider desktop viewport;
- no horizontal overflow;
- no clipped or overlapping text/control content;
- no missing baseline blocks unless explicitly documented;
- theme controls visibly affect accent color, spacing/padding density, panel
  opacity and panel blur;
- shell background assets load correctly;
- no relevant console errors are present.

Screenshot QA must compare current Vue screens with the `7f1cf42f` baseline for
the restored slices. Screenshot artifacts should stay outside the repository
unless explicitly requested.

## Acceptance Criteria

The work is complete when:

- the current Vue interface visually matches the `7f1cf42f` baseline for the
  restored screens within documented intentional differences;
- the interface is fully interactive, not merely visually patched;
- runtime bootstrap, API auth and critical domain data flows work;
- theme settings configure color/accent, spacing/padding density, opacity and
  blur through tokens;
- no widget/layout builder was introduced;
- SRP gates are satisfied, with no component over `700` lines;
- validation commands and browser QA have been run and reported;
- remaining risks, if any, are explicitly documented.

