# Settings Redesign Design

## Purpose

The current Settings page works functionally, but the Accounts section looks like a dashboard of duplicated provider cards rather than a settings surface. Mail, Calendar and Contacts repeat the same Gmail/iCloud account across separate panels, technical account IDs get too much visual weight, and the horizontal settings tabs do not scale like an IDE preferences screen.

This redesign turns Settings into a calm desktop IDE-style control surface:

- left settings tree grouped by domain;
- central work area for the selected section;
- right inspector for selected objects and actions;
- provider integrations as the primary object in Accounts;
- service status columns instead of noisy badges.

## Relevant ADRs And Constraints

- `ADR-0003 SvelteKit Frontend`: frontend remains in the SvelteKit desktop SPA.
- `ADR-0004 Tauri Desktop Shell`: Settings is part of the desktop shell surface.
- `ADR-0026 Desktop First Responsive UI`: optimize for desktop and resizable laptop windows.
- `ADR-0031 Temporary Desktop Only UI Scope`: no mobile UI design, implementation or validation.
- `ADR-0054 Application Settings Store`: user-editable app settings remain declared non-secret settings.
- `ADR-0077 i18n Russian and English`: new user-visible strings must go through the existing i18n dictionaries.
- `ADR-0078 Frontend Component Decomposition`: Settings state belongs in `$lib/stores/settings.ts`; page widgets should stay readable and scoped.

Security constraints:

- never show secret values, OAuth tokens, app passwords or private provider credentials;
- show only non-secret provider metadata, account IDs, service states and vault binding status;
- destructive actions must stay in the inspector and require confirmation.

## Non-Goals

This design does not implement:

- mobile settings UI;
- new backend account setup APIs;
- new sync engine or sync-health backend fields;
- provider credential editing inline in the table;
- account deletion without confirmation;
- a full design-system rewrite;
- route-based SvelteKit settings pages.

## Approved Direction

Use a hybrid IDE settings model:

- the overall Settings screen behaves like IDE Preferences;
- left navigation is a tree, not horizontal tabs;
- normal settings are displayed as dense form/list sections;
- Accounts becomes `Sources -> Integrations`;
- the primary row is an integration/provider, such as Google Workspace, Apple/iCloud, Telegram or WhatsApp;
- connected services are shown as table columns, not badges;
- details, diagnostics and actions live in the right inspector.

The chosen visual direction is a Hermes-shell/IDE hybrid: the outer app keeps the Hermes dark glass environment, while the Settings work area is quieter, denser and more table-driven.

## Information Architecture

Initial left tree:

- `General`
  - `Application`
  - `Language`
- `Interface`
  - `Appearance`
  - `Sidebar`
- `Sources`
  - `Integrations`
  - `Sync`
  - `Vault`
- `Advanced`
  - `Developer`

Only sections backed by existing UI need to be interactive in the first implementation. Future sections may render disabled or hidden until backed by real content, but the implementation should not show fake functional pages.

`Integrations` replaces the current account dashboard layout. It consolidates:

- mail accounts;
- calendar accounts;
- contacts-capable provider accounts;
- Telegram accounts;
- WhatsApp/future communication providers.

## Integrations Table

The table is for scanning. It should stay compact and avoid large card surfaces.

Columns:

- `Integration`: provider display name and secondary identity;
- `Mail`: service state;
- `Calendar`: service state;
- `People`: contacts/people service state;
- `Updated`: latest known update timestamp for the integration;
- `Status`: integration-level summary.

Service state labels:

- `Ready`: service is configured and usable from available metadata;
- `Auth`: credential or authorization needs attention;
- `Disabled`: service exists but is disabled by user/config;
- `-`: service does not apply to the provider or is not configured;
- future sync states may add `Syncing`, `Error` or `Stale` when backend data exists.

The first implementation can derive `Ready` from existing metadata:

- Gmail provider account with connected services produces Mail, Calendar and People states when corresponding provider/calendar metadata exists;
- iCloud provider account with connected services produces Mail, Calendar and People states when corresponding provider/calendar metadata exists;
- Telegram appears as an integration row with message/account status and `-` for Mail/Calendar/People;
- WhatsApp appears as empty or configured based on existing provider account data.

## Inspector

Selecting a table row opens details in the right inspector without a route change or modal.

Inspector sections:

- integration title and secondary identity;
- service list with per-service state and short explanation;
- next action or warning if anything needs attention;
- actions;
- non-secret metadata.

Actions:

- `Reconnect` for OAuth/app-password backed providers;
- `Run sync now` only when a real sync command exists, otherwise omit or disable with clear copy;
- `View vault binding` for non-secret secret-reference metadata only;
- `Remove integration` as a destructive action requiring confirmation.

The inspector owns dangerous and detailed actions. The table rows should not expose destructive controls.

## Add Integration Flow

The global `Add integration` action opens the existing account wizard in provider-selection mode.

Rules:

- do not ask for email before provider choice;
- Google starts OAuth directly after provider selection;
- iCloud/IMAP continue to use provider-specific credential forms;
- after successful setup, Settings reloads and selects the created integration where possible.

This design does not require backend changes for the first pass.

## Data Model For Frontend

Create a view model layer rather than building table logic directly in Svelte markup.

Suggested shape:

```ts
type IntegrationServiceState = 'ready' | 'auth' | 'disabled' | 'not_applicable' | 'unknown';

type IntegrationService = {
  id: 'mail' | 'calendar' | 'people' | 'messages' | string;
  label: string;
  state: IntegrationServiceState;
  description: string;
};

type IntegrationViewModel = {
  integrationId: string;
  providerKind: string;
  title: string;
  subtitle: string;
  updatedAt: string | null;
  status: 'connected' | 'needs_action' | 'empty' | 'partial';
  services: IntegrationService[];
  accounts: ProviderAccount[];
  calendarAccounts: CalendarAccount[];
  metadata: Record<string, string>;
};
```

The first implementation should derive this from existing frontend data:

- `providerAccounts`;
- `calendarAccounts`;
- existing `connected_services` config;
- existing provider kind labels/icons;
- existing account update timestamps.

No backend schema change is required unless implementation discovers current API data is insufficient for the approved first-pass states.

## Component Boundaries

Expected frontend structure:

- `SettingsPage.svelte`: owns Settings shell layout, left tree selection and section switching.
- `IntegrationsSettings.svelte`: renders the integrations table and inspector.
- settings store/service helper: builds `IntegrationViewModel[]` from current store data.
- existing account wizard store remains the entry point for add/reconnect flows.

The current `AccountsSettings.svelte` can either be renamed to `IntegrationsSettings.svelte` or replaced with a new widget, depending on which is cleaner during implementation.

Keep the change scoped to Settings. Do not refactor unrelated pages or global shell components unless a small shared CSS/helper change is required.

## Visual Rules

- Use existing Hermes tokens, panel radius and typography scale.
- Make Settings quieter than dashboard pages: fewer nested cards, fewer glows, less background competition.
- Avoid service badges in integration rows.
- Use compact service columns with restrained state labels.
- Keep technical IDs secondary; show them as metadata, not large pill/code blocks.
- Use icons from the existing icon stack rather than custom SVGs.
- Do not create cards inside cards.
- Text must fit within desktop resizing constraints without overlap.

## Error Handling

Errors should be local and actionable:

- table-level load error if accounts/settings cannot load;
- row-level degraded status if one service needs attention;
- inspector warning explaining the selected integration issue;
- account setup failures stay in the existing setup flow and Settings action/error message area.

A degraded service should not mark the entire integration as fully broken when other services can still work.

## Testing And Validation

Implementation should include or update frontend tests for:

- deriving integrations from Gmail, iCloud, Telegram and empty WhatsApp data;
- service columns for Mail, Calendar and People;
- contacts source no longer duplicated as a separate card section;
- selecting an integration changes inspector content;
- add integration action still opens the account wizard;
- i18n dictionary coverage for new Settings strings.

Expected validation commands:

- `cd frontend && pnpm check`;
- relevant frontend tests if available;
- `git diff --check`;
- live desktop smoke at `http://127.0.0.1:5174` with the Settings screen open.

Backend validation is not required for a frontend-only implementation unless backend code is changed.

## Open Follow-Ups

Future slices may add:

- real sync health, last sync result and sync progress fields;
- per-service enable/disable toggles;
- provider-specific diagnostics;
- account removal backend flow;
- Settings search across all sections;
- keyboard navigation within the settings tree and integration table.
