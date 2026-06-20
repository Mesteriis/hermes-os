# Documentation Alignment Architecture Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove architecture stop-factors that block documentation-aligned feature work before adding mail or Telegram parity functionality.

**Architecture:** Documentation remains the source of truth. The first executable stream eliminates God Files and God Components, then implementation can proceed through capability-gated mail and Telegram slices without expanding monoliths. Existing public routes stay stable only when the current documentation allows them; backward compatibility is not a constraint.

**Tech Stack:** Rust 1.88 backend with Axum/SQLx, SvelteKit 2/Svelte 5 frontend with pnpm, Tauri 2 desktop shell, Makefile validation gates.

---

## Governing Documents

- `AGENTS.md`
- `docs/adr/ADR-0001-event-sourcing-as-system-spine.md`
- `docs/adr/ADR-0002-rust-backend.md`
- `docs/adr/ADR-0003-sveltekit-frontend.md`
- `docs/adr/ADR-0004-tauri-desktop-shell.md`
- `docs/adr/ADR-0055-full-email-provider-networking.md`
- `docs/adr/ADR-0067-calendar-multi-provider-architecture.md`
- `docs/adr/ADR-0073-backend-module-organization.md`
- `docs/adr/ADR-0078-frontend-component-decomposition.md`
- `docs/adr/ADR-0079-script-logic-decomposition.md`
- `docs/adr/ADR-0080-mail-background-sync-progress-local-trash.md`
- `docs/adr/ADR-0083-telegram-live-user-client-runtime.md`
- `docs/adr/ADR-0091-telegram-production-client-capability-model.md`
- `docs/adr/ADR-0092-mail-provider-capability-tiers.md`
- `docs/telegram/README.md`
- `docs/calendar/architecture.md`
- `docs/calendar/README.md`
- `docs/mail/README.md`

## Current Stop-Factors

- Backend source files over 700 lines still exist outside completed backend decomposition slices.
- Frontend Svelte God Components have been decomposed by completed tasks; do not reintroduce components over 500 lines.
- Frontend CSS God Files have been decomposed by completed tasks; do not reintroduce CSS files over 700 lines.
- Frontend TypeScript service/store files have been decomposed by completed tasks; do not reintroduce files over 700 lines.
- Feature parity work for mail and Telegram must not add code to these files until the relevant file/component is decomposed.

### Task 1: Mail Handler God File Decomposition

**Files:**
- Modify: `backend/src/domains/communications/handlers/mod.rs`
- Create: `backend/src/domains/communications/handlers/account_management.rs`
- Create: `backend/src/domains/communications/handlers/account_setup.rs`
- Create: `backend/src/domains/communications/handlers/account_support.rs`
- Create: `backend/src/domains/communications/handlers/communication_messages.rs`
- Create: `backend/src/domains/communications/handlers/communication_queries.rs`
- Create: `backend/src/domains/communications/handlers/finance_analytics.rs`
- Create: `backend/src/domains/communications/handlers/legal_export.rs`
- Create: `backend/src/domains/communications/handlers/message_actions.rs`
- Create: `backend/src/domains/communications/handlers/sending.rs`
- Create: `backend/src/domains/communications/handlers/templates_status.rs`
- Create: `backend/src/domains/communications/handlers/workflow_state.rs`

- [x] **Step 1: Verify the stop-factor fails**

Run:

```sh
test "$(wc -l < backend/src/domains/communications/handlers/mod.rs | tr -d ' ')" -le 700
```

Expected before refactor: FAIL because the file has 3000 lines.

- [x] **Step 2: Split bounded handler groups**

Move contiguous handler groups into sibling modules and keep `pub(crate)` route handlers re-exported from `mod.rs`.

The resulting `backend/src/domains/communications/handlers/mod.rs` owns module assembly and imports only. Handler modules own one bounded route group each.

- [x] **Step 3: Verify the target files are below threshold**

Run:

```sh
find backend/src/domains/communications/handlers -maxdepth 1 -type f -name '*.rs' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
```

Expected after refactor: PASS.

- [x] **Step 4: Validate formatting and compilation**

Run:

```sh
make backend-fmt-check
make backend-check
```

Expected after refactor: PASS.

- [x] **Step 5: Commit**

Run:

```sh
git add backend/src/domains/communications/handlers docs/superpowers/plans/2026-06-13-documentation-alignment-architecture-gates.md IMPLEMENTATION_STATUS.md
git commit -m "refactor: split mail handler modules"
```

### Task 2: Account Setup Modal Component Decomposition

**Files:**
- Modify: `frontend/src/lib/components/shared/AccountSetupModal.svelte`
- Create: `frontend/src/lib/components/account-setup/MailAccountSetup.svelte`
- Create: `frontend/src/lib/components/account-setup/MailAccountWizard.svelte`
- Create: `frontend/src/lib/components/account-setup/CalendarAccountSetup.svelte`
- Create: `frontend/src/lib/components/account-setup/CalendarAccountWizard.svelte`
- Create: `frontend/src/lib/components/account-setup/TelegramAccountSetup.svelte`
- Create: `frontend/src/lib/components/account-setup/TelegramAccountWizard.svelte`
- Create: `frontend/src/lib/components/account-setup/TelegramQrLoginPanel.svelte`
- Create: `frontend/src/lib/components/account-setup/WhatsappAccountSetup.svelte`
- Create: `frontend/src/lib/components/account-setup/WhatsappAccountWizard.svelte`
- Modify: `frontend/src/lib/services/accounts.ts`

- [x] **Step 1: Verify current component threshold failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/components/shared/AccountSetupModal.svelte | tr -d ' ')" -le 500
```

Expected before refactor: FAIL because the component has 1219 lines.

- [x] **Step 2: Reuse typed account setup service helpers**

Reuse existing typed IMAP, calendar and Telegram setup helpers from `accounts.ts` and `telegram.ts`; export account wizard boundary types from `accounts.ts` so Svelte components do not redefine them locally. Existing frontend service tests cover these helpers.

- [x] **Step 3: Extract wizard components**

Each setup component owns one provider family, its local state and side effects. `AccountSetupModal.svelte` remains responsible only for modal framing, target selection and close behavior.

- [x] **Step 4: Validate component thresholds and frontend checks**

Run:

```sh
test "$(wc -l < frontend/src/lib/components/shared/AccountSetupModal.svelte | tr -d ' ')" -le 500
find frontend/src/lib/components/account-setup frontend/src/lib/components/shared/AccountSetupModal.svelte -type f -name '*.svelte' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 500 { print; failed=1 } END { exit failed ? 1 : 0 }'
pnpm --dir frontend test:unit
make lint-frontend
```

Expected after this task: no account setup component exceeds 500 lines; remaining unrelated oversized components are recorded in `IMPLEMENTATION_STATUS.md`.

### Task 3: Telegram Page Decomposition Before Telegram Parity Work

**Files:**
- Modify: `frontend/src/lib/pages/telegram/TelegramPage.svelte`
- Create: `frontend/src/lib/pages/telegram/widgets/TelegramCommandHeader.svelte`
- Create: `frontend/src/lib/pages/telegram/widgets/TelegramActionRail.svelte`
- Create: `frontend/src/lib/pages/telegram/widgets/TelegramStatusMessages.svelte`

- [x] **Step 1: Verify current component threshold failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/pages/telegram/TelegramPage.svelte | tr -d ' ')" -le 500
```

Expected before refactor: FAIL because the component has 842 lines.

- [x] **Step 2: Extract runtime status, capability panel and account action panels**

Keep Telegram actions capability-gated per ADR-0091 and `docs/telegram/`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/pages/telegram/TelegramPage.svelte | tr -d ' ')" -le 500
find frontend/src/lib/pages/telegram -type f -name '*.svelte' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 500 { print; failed=1 } END { exit failed ? 1 : 0 }'
pnpm --dir frontend test:unit
make lint-frontend
```

### Task 4: AI Settings Control Center Decomposition

**Files:**
- Modify: `frontend/src/lib/pages/settings/widgets/AISettingsControlCenter.svelte`
- Create focused AI settings panel components under `frontend/src/lib/pages/settings/widgets/`

- [x] **Step 1: Verify current component threshold failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/pages/settings/widgets/AISettingsControlCenter.svelte | tr -d ' ')" -le 500
```

Expected before refactor: FAIL because the component has 622 lines.

- [x] **Step 2: Extract AI settings sections**

Move header, tabs, status, overview, provider panels, routing, prompt studio, runs and rail UI into focused widgets. Keep provider secret handling and remote-consent behavior behind the existing `aiSettings` store/service boundary.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/pages/settings/widgets/AISettingsControlCenter.svelte | tr -d ' ')" -le 500
find frontend/src/lib/pages/settings/widgets -maxdepth 1 -type f -name 'AI*.svelte' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 500 { print; failed=1 } END { exit failed ? 1 : 0 }'
pnpm --dir frontend test:unit
make lint-frontend
```

### Task 5: Communications Message Detail Decomposition

**Files:**
- Modify: `frontend/src/lib/pages/communications/widgets/CommunicationsMessageDetail.svelte`
- Create focused message detail tab components under `frontend/src/lib/pages/communications/widgets/`

- [x] **Step 1: Verify current component threshold failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/pages/communications/widgets/CommunicationsMessageDetail.svelte | tr -d ' ')" -le 500
```

Expected before refactor: FAIL because the component has 550 lines.

- [x] **Step 2: Extract message detail tabs**

Move message body rendering, attachment list, header table, related actions/results and timeline UI into focused widgets. Keep selected message derivation and communication callbacks in the existing parent component.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/pages/communications/widgets/CommunicationsMessageDetail.svelte | tr -d ' ')" -le 500
find frontend/src/lib/pages/communications -type f -name '*.svelte' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 500 { print; failed=1 } END { exit failed ? 1 : 0 }'
find frontend/src -type f -name '*.svelte' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 500 { print; failed=1 } END { exit failed ? 1 : 0 }'
make lint-frontend
make lint-architecture
```

### Task 6: Communications CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/pages/communications/CommunicationsPage.svelte`
- Modify: `frontend/src/lib/pages/telegram/TelegramPage.svelte`
- Modify: `frontend/src/lib/pages/whatsapp/WhatsAppPage.svelte`
- Create focused communication CSS files under `frontend/src/lib/pages/communications/`

- [x] **Step 1: Verify communications CSS ownership failure**

Run:

```sh
! rg "^(\.communications-page|\.conversation-list-head|\.message-context-tabs|\.related-link-list|\.inspector-header)" frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns communications page, conversation list, message tabs, related list and inspector selectors.

- [x] **Step 2: Extract communication workspace CSS**

Move communication workspace, message detail and inspector selectors into communication-owned CSS chunks. Keep each new CSS file below 700 lines and import the shared communication workspace CSS from Communications, Telegram and WhatsApp pages because all three currently use the same `.communications-grid` shell classes.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.communications-page|\.conversation-list-head|\.message-context-tabs|\.related-link-list|\.inspector-header)" frontend/src/lib/pages/pages.css
find frontend/src/lib/pages/communications -maxdepth 1 -type f -name '*.css' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
make lint-frontend
make lint-architecture
```

### Task 7: Telegram CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/pages/telegram/TelegramPage.svelte`
- Modify: `frontend/src/lib/components/account-setup/TelegramAccountSetup.svelte`
- Modify: `frontend/src/lib/pages/communications/communications.css`
- Create focused Telegram CSS files under `frontend/src/lib/pages/telegram/`
- Create Telegram QR setup CSS under `frontend/src/lib/components/account-setup/`

- [x] **Step 1: Verify Telegram CSS ownership failure**

Run:

```sh
! rg "^(\.telegram-|\.telegram-grid|\.telegram-page|\.communications-grid \.conversation-list > button\.telegram-chat-row)" frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns Telegram page, list, thread, rail and Telegram QR setup selectors.

- [x] **Step 2: Extract Telegram CSS chunks**

Move Telegram page/list/thread/rail selectors into focused page-owned CSS chunks. Move Telegram QR-specific setup selectors to account setup CSS. Keep the shared Telegram/WhatsApp grid compatibility selectors in the communication workspace CSS because both pages still use the same `.communications-grid` shell.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.telegram-|\.telegram-grid|\.telegram-page|\.communications-grid \.conversation-list > button\.telegram-chat-row)" frontend/src/lib/pages/pages.css
find frontend/src/lib/pages/telegram frontend/src/lib/components/account-setup -maxdepth 1 -type f -name '*.css' -print0 \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
make lint-frontend
make lint-architecture
```

### Task 8: Account Setup and Shared Setup CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/components/shared/AccountSetupModal.svelte`
- Modify: `frontend/src/lib/components/shared/ComposeDrawer.svelte`
- Modify setup/status consumers that use shared setup form, setup state or form status classes
- Create: `frontend/src/lib/components/account-setup/accountSetup.css`
- Create: `frontend/src/lib/components/shared/accountModal.css`
- Create: `frontend/src/lib/components/shared/setupControls.css`
- Create: `frontend/src/lib/components/shared/composeDrawer.css`

- [x] **Step 1: Verify account/setup CSS ownership failure**

Run:

```sh
! rg "^(\.account-modal|\.provider-tabs|\.account-wizard-tabs|\.wizard-progress|\.wizard-step|\.wizard-choice|\.wizard-back|\.qr-login-panel|\.qr-svg|\.qr-login-copy|\.qr-skeleton|\.setup-form|\.setup-summary-card|\.setup-state|\.form-status|\.send-review-modal|\.send-review-grid)" frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns account modal, setup wizard, QR login, shared setup form/status and compose review selectors.

- [x] **Step 2: Extract account setup and shared setup CSS chunks**

Move account setup wizard and QR selectors to account setup CSS. Move modal, shared setup controls/status and compose send-review selectors to focused shared component CSS chunks. Keep existing semantic class names, and remove unused legacy wizard selectors instead of moving dead CSS.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.account-modal|\.provider-tabs|\.account-wizard-tabs|\.wizard-progress|\.wizard-step|\.wizard-choice|\.wizard-back|\.qr-login-panel|\.qr-svg|\.qr-login-copy|\.qr-skeleton|\.setup-form|\.setup-summary-card|\.setup-state|\.form-status|\.send-review-modal|\.send-review-grid)" frontend/src/lib/pages/pages.css
printf '%s\0' frontend/src/lib/components/account-setup/accountSetup.css frontend/src/lib/components/shared/accountModal.css frontend/src/lib/components/shared/setupControls.css frontend/src/lib/components/shared/composeDrawer.css frontend/src/lib/components/account-setup/telegramQr.css \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
make lint-frontend
make lint-architecture
```

### Task 9: Settings and HermesSelect CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/pages/settings/SettingsPage.svelte`
- Modify: `frontend/src/lib/pages/settings/widgets/AppearanceSettings.svelte`
- Modify: `frontend/src/lib/pages/settings/widgets/IntegrationsSettings.svelte`
- Modify: `frontend/src/lib/pages/settings/widgets/AISettingsControlCenter.svelte`
- Modify: `frontend/src/lib/components/shared/HermesSelect.svelte`
- Create: `frontend/src/lib/pages/settings/settings.css`
- Create: `frontend/src/lib/pages/settings/appearance.css`
- Create: `frontend/src/lib/pages/settings/integrations.css`
- Create: `frontend/src/lib/pages/settings/aiSettings.css`
- Create: `frontend/src/lib/components/shared/hermesSelect.css`

- [x] **Step 1: Verify settings CSS ownership failure**

Run:

```sh
! rg "^(\.(settings-|setting-|appearance-|background-|accent-|bg-preview-|brightness-|integrations-|integration-|mail-settings-import-panel|danger-button|ai-settings-|ai-overview-|ai-provider-|ai-panel-|ai-route-|ai-control-|ai-wizard-|ai-search-box|ai-prompt-|ai-consent-|hermes-select))" frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns settings workbench, appearance, integrations, AI settings and shared HermesSelect selectors.

- [x] **Step 2: Extract settings and shared select CSS chunks**

Move settings page/workbench controls, appearance settings, integrations table/inspector, AI settings control center and HermesSelect CSS into owner files. Keep Agents AI workflow and Telegram/WhatsApp AI analysis selectors in root for later owner-specific extraction because they are not settings-owned.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.(settings-|setting-|appearance-|background-|accent-|bg-preview-|brightness-|integrations-|integration-|mail-settings-import-panel|danger-button|ai-settings-|ai-overview-|ai-provider-|ai-panel-|ai-route-|ai-control-|ai-wizard-|ai-search-box|ai-prompt-|ai-consent-|hermes-select))" frontend/src/lib/pages/pages.css
printf '%s\0' frontend/src/lib/pages/settings/settings.css frontend/src/lib/pages/settings/appearance.css frontend/src/lib/pages/settings/integrations.css frontend/src/lib/pages/settings/aiSettings.css frontend/src/lib/components/shared/hermesSelect.css \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
make lint-frontend
make lint-architecture
```

### Task 10: Agents CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/pages/agents/AgentsPage.svelte`
- Create: `frontend/src/lib/pages/agents/agents.css`

- [x] **Step 1: Verify agents CSS ownership failure**

Run:

```sh
! rg "^(\.(agent-main|agents-layout|agent-metrics|agent-grid|agent-card|agent-detail|agent-detail-grid|ai-workflow-|ai-result-|citation-|spark-chart))" frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns Agents page layout, agent cards/detail, AI workflow and citation selectors.

- [x] **Step 2: Extract agents CSS chunk**

Move Agents page layout and widget selectors into `frontend/src/lib/pages/agents/agents.css` and import it from `AgentsPage.svelte`. Keep shared `.agents-page` grid shell rules in root for the later common page-shell split.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.(agent-main|agents-layout|agent-metrics|agent-grid|agent-card|agent-detail|agent-detail-grid|ai-workflow-|ai-result-|citation-|spark-chart))" frontend/src/lib/pages/pages.css
test "$(wc -l < frontend/src/lib/pages/agents/agents.css | tr -d ' ')" -le 700
make lint-frontend
make lint-architecture
```

### Task 11: Calendar CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/pages/calendar/CalendarPage.svelte`
- Create: `frontend/src/lib/pages/calendar/calendar.css`

- [x] **Step 1: Verify calendar CSS ownership failure**

Run:

```sh
! rg "^(\.(calendar-layout|week-board|week-header|time-grid|event-block|now-line|event-list|event-row|new-event-form|brief-section|brief-participants))" frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns Calendar layout, week board, event rows, new event form and event brief selectors.

- [x] **Step 2: Extract calendar CSS chunk**

Move Calendar page layout, week board, event list, new event form and event detail selectors into `frontend/src/lib/pages/calendar/calendar.css` and import it from `CalendarPage.svelte`. Keep shared `.calendar-page` page-shell rules in root for the later common page-shell split.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.(calendar-layout|week-board|week-header|time-grid|event-block|now-line|event-list|event-row|new-event-form|brief-section|brief-participants))" frontend/src/lib/pages/pages.css
test "$(wc -l < frontend/src/lib/pages/calendar/calendar.css | tr -d ' ')" -le 700
make lint-frontend
make lint-architecture
```

### Task 12: Documents and Notes CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/lib/pages/documents/DocumentsPage.svelte`
- Modify: `frontend/src/lib/pages/notes/NotesPage.svelte`
- Create: `frontend/src/lib/pages/documents/documents.css`
- Create: `frontend/src/lib/pages/notes/notes.css`

- [x] **Step 1: Verify Documents/Notes CSS ownership failure**

Run:

```sh
! rg "^(\.(documents-layout|notes-layout|document-source-cards|document-main-list|notes-main-list|docs-layout|docs-table|category-grid|notes-list))" frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns active Documents/Notes layout and overflow selectors plus dead legacy docs/notes selectors.

- [x] **Step 2: Extract Documents/Notes CSS chunks and remove dead legacy selectors**

Move active Documents and Notes page layout/overflow selectors into page-owned CSS files and import them from their page components. Remove unused legacy `docs-*`, `category-grid`, `notes-list` and `notes-main` selectors that have no Svelte consumers. Remove transferred Documents/Notes layout selectors from `app.css` media/layout groups so app-level CSS no longer owns these page layouts.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.(documents-layout|notes-layout|document-source-cards|document-main-list|notes-main-list|docs-layout|docs-table|category-grid|notes-list))" frontend/src/lib/pages/pages.css
! rg "\b(docs-layout|documents-layout|notes-layout|notes-list)\b" frontend/src/lib/styles/app.css
printf '%s\0' frontend/src/lib/pages/documents/documents.css frontend/src/lib/pages/notes/notes.css \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
make lint-frontend
make lint-architecture
```

### Task 13: Projects CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/lib/pages/projects/ProjectsPage.svelte`
- Create: `frontend/src/lib/pages/projects/projects.css`

- [x] **Step 1: Verify Projects CSS ownership failure**

Run:

```sh
! rg "^(\.(project-meta-strip|project-hero|project-logo|project-empty-state|project-switcher|project-dashboard-grid|graph-card-large))" frontend/src/lib/pages/pages.css
! rg "\b(project-hero|project-empty-state|project-dashboard-grid|project-meta-strip|project-switcher|project-side)\b" frontend/src/lib/styles/app.css
```

Expected before refactor: FAIL because root `pages.css` and app-level `app.css` own Projects page layout, hero, switcher, dashboard and project rail selectors.

- [x] **Step 2: Extract Projects CSS chunk**

Move Projects page layout, hero, metadata strip, switcher, dashboard and `graph-card-large` selectors into `frontend/src/lib/pages/projects/projects.css` and import it from `ProjectsPage.svelte`. Keep shared cross-page `doc-mini` and `graph-center` rules in shared/global files because they are used by both Projects and Knowledge.

- [x] **Step 3: Validate**

Run:

```sh
! rg "^(\.(project-meta-strip|project-hero|project-logo|project-empty-state|project-switcher|project-dashboard-grid|graph-card-large))" frontend/src/lib/pages/pages.css
! rg "\b(project-hero|project-empty-state|project-dashboard-grid|project-meta-strip|project-switcher|project-side)\b" frontend/src/lib/styles/app.css
test "$(wc -l < frontend/src/lib/pages/projects/projects.css | tr -d ' ')" -le 700
make lint-frontend
make lint-architecture
```

### Task 14: Tasks CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/lib/pages/tasks/TasksPage.svelte`
- Create: `frontend/src/lib/pages/tasks/tasks.css`

- [x] **Step 1: Verify Tasks CSS ownership failure**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(tasks-page|tasks-layout|task-table|task-table-head|task-context-review-|task-group)' frontend/src/lib/pages/pages.css
```

Expected before refactor: FAIL because root `pages.css` owns Tasks page shell, layout, task table, task review panel and task group selectors.

- [x] **Step 2: Extract Tasks CSS chunk**

Move Tasks page shell, layout, task table, task context review and task group selectors into `frontend/src/lib/pages/tasks/tasks.css` and import it from `TasksPage.svelte`. Remove the Tasks layout selector from shared `app.css` layout groups so the owner file preserves the previous effective grid behavior without relying on global cascade order. Keep shared `task-row`, `task-stack` and task action selectors in shared CSS because they are also used outside the Tasks page or remain part of the shared panel split.

- [x] **Step 3: Validate**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(tasks-page|tasks-layout|task-table|task-table-head|task-context-review-|task-group)' frontend/src/lib/pages/pages.css
! rg "\btasks-layout\b" frontend/src/lib/styles/app.css
test "$(wc -l < frontend/src/lib/pages/tasks/tasks.css | tr -d ' ')" -le 700
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 15: Persons CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/lib/pages/persons/PersonsPage.svelte`
- Create: `frontend/src/lib/pages/persons/persons.css`

- [x] **Step 1: Verify Persons CSS ownership failure**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(persons-page|persons-layout|person-detail|person-hero|person-cards|identity-|relationship-review-|identity-trace-target)' frontend/src/lib/pages/pages.css
! rg -n '\b(persons-layout|person-hero)\b' frontend/src/lib/styles/app.css
```

Expected before refactor: FAIL because root `pages.css` and app-level `app.css` own Persons page shell, layout, hero, dossier card layout, identity review and relationship review selectors.

- [x] **Step 2: Extract Persons CSS chunk**

Move Persons page shell, 12-column layout, detail, hero, person cards, identity candidate review, identity trace assignment and relationship review selectors into `frontend/src/lib/pages/persons/persons.css` and import it from `PersonsPage.svelte`. Remove the Persons layout and hero selectors from shared `app.css`, preserving the previous effective cascade in the owner file.

- [x] **Step 3: Validate**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(persons-page|persons-layout|person-detail|person-hero|person-cards|identity-|relationship-review-|identity-trace-target)' frontend/src/lib/pages/pages.css
! rg -n '\b(persons-layout|person-hero)\b' frontend/src/lib/styles/app.css
test "$(wc -l < frontend/src/lib/pages/persons/persons.css | tr -d ' ')" -le 700
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 16: Timeline CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/lib/pages/timeline/TimelinePage.svelte`
- Create: `frontend/src/lib/pages/timeline/timeline.css`

- [x] **Step 1: Verify Timeline CSS ownership failure**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(timeline-page|timeline-layout|large-timeline|timeline-event-row|rail-dot|timeline-slider)' frontend/src/lib/pages/pages.css
! rg -n '\btimeline-layout\b' frontend/src/lib/styles/app.css
```

Expected before refactor: FAIL because root `pages.css` owns Timeline page shell, active stream selectors and dead `timeline-slider` selectors, while `app.css` owns the effective Timeline layout grid.

- [x] **Step 2: Extract Timeline CSS chunk**

Move active Timeline page shell, 12-column layout, stream row and rail-dot selectors into `frontend/src/lib/pages/timeline/timeline.css` and import it from `TimelinePage.svelte`. Remove unused `timeline-slider` CSS instead of moving dead selectors.

- [x] **Step 3: Validate**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(timeline-page|timeline-layout|large-timeline|timeline-event-row|rail-dot|timeline-slider)' frontend/src/lib/pages/pages.css
! rg -n '\btimeline-layout\b' frontend/src/lib/styles/app.css
! rg -n 'timeline-slider' frontend/src
test "$(wc -l < frontend/src/lib/pages/timeline/timeline.css | tr -d ' ')" -le 700
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 17: Organizations CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/lib/pages/organizations/OrganizationsPage.svelte`
- Create: `frontend/src/lib/pages/organizations/organizations.css`

- [x] **Step 1: Verify Organizations CSS ownership failure**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(organizations-page|org-layout|org-list-panel|org-detail-panel|org-detail-grid)' frontend/src/lib/pages/pages.css
! rg -n '\borg-layout\b' frontend/src/lib/styles/app.css
```

Expected before refactor: FAIL because root `pages.css` owns Organizations page shell, list/detail panel selectors and detail grid selectors, while `app.css` owns the effective Organizations layout grid.

- [x] **Step 2: Extract Organizations CSS chunk**

Move Organizations page shell, 12-column layout, list panel, detail panel/grid and person mini selectors into `frontend/src/lib/pages/organizations/organizations.css` and import it from `OrganizationsPage.svelte`. Remove the Organizations layout selector from shared `app.css`, preserving the previous effective cascade in the owner file.

- [x] **Step 3: Validate**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(organizations-page|org-layout|org-list-panel|org-detail-panel|org-detail-grid)' frontend/src/lib/pages/pages.css
! rg -n '\borg-layout\b' frontend/src/lib/styles/app.css
test "$(wc -l < frontend/src/lib/pages/organizations/organizations.css | tr -d ' ')" -le 700
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 18: Knowledge and Review CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/lib/pages/knowledge/KnowledgePage.svelte`
- Modify: `frontend/src/lib/pages/review/ReviewPage.svelte`
- Create: `frontend/src/lib/pages/knowledge/knowledge.css`
- Create: `frontend/src/lib/pages/review/review.css`

- [x] **Step 1: Verify Knowledge/Review CSS ownership failure**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(knowledge-page|knowledge-layout|knowledge-side-rail|graph-filter-tabs|graph-workbench|graph-search-form|graph-search-strip|graph-picker|graph-result-row|knowledge-canvas|graph-edge-layer|graph-node|graph-state-card|graph-loading-overlay|graph-status-bar|polygraph-review-panel|polygraph-state|polygraph-list|polygraph-item|polygraph-actions)' frontend/src/lib/pages/pages.css
! rg -n '(^|[,{[:space:]])\.(review-page|review-overview|review-metrics|review-board|review-queue-panel|review-empty|review-list|review-item|review-actions)' frontend/src/lib/pages/pages.css
! rg -n '\b(knowledge-layout|graph-filter-tabs|graph-toolbar)\b' frontend/src/lib/styles/app.css
```

Expected before refactor: FAIL because root `pages.css` owns Knowledge Graph, Polygraph review and Review queue selectors, while `app.css` owns Knowledge layout, filters and toolbar responsive behavior.

- [x] **Step 2: Extract Knowledge and Review CSS chunks**

Move Knowledge page shell, 12-column layout, graph filters, graph workbench, graph canvas and Knowledge Polygraph selectors into `frontend/src/lib/pages/knowledge/knowledge.css`. Move Review page shell, metrics and queue board selectors into `frontend/src/lib/pages/review/review.css`. Keep shared `graph-strip-message`, `doc-mini`, `graph-center` and `evidence-row` in shared/root CSS until a dedicated shared component CSS split; remove unused `knowledge-core` CSS.

- [x] **Step 3: Validate**

Run:

```sh
! rg -n '(^|[,{[:space:]])\.(knowledge-page|knowledge-layout|knowledge-side-rail|graph-filter-tabs|graph-workbench|graph-search-form|graph-search-strip|graph-picker|graph-result-row|knowledge-canvas|graph-edge-layer|graph-node|graph-state-card|graph-loading-overlay|graph-status-bar|polygraph-review-panel|polygraph-state|polygraph-list|polygraph-item|polygraph-actions)' frontend/src/lib/pages/pages.css
! rg -n '(^|[,{[:space:]])\.(review-page|review-overview|review-metrics|review-board|review-queue-panel|review-empty|review-list|review-item|review-actions)' frontend/src/lib/pages/pages.css
! rg -n '\b(knowledge-layout|graph-filter-tabs|graph-toolbar)\b' frontend/src/lib/styles/app.css
printf '%s\0' frontend/src/lib/pages/knowledge/knowledge.css frontend/src/lib/pages/review/review.css \
  | xargs -0 wc -l \
  | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
test "$(wc -l < frontend/src/lib/pages/pages.css | tr -d ' ')" -le 700
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 19: Sidebar Settings CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/components/shell/sidebar.css`
- Modify: `frontend/src/lib/pages/settings/widgets/SidebarSettings.svelte`
- Create: `frontend/src/lib/pages/settings/widgets/sidebarSettings.css`

- [x] **Step 1: Verify sidebar settings CSS ownership failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/components/shell/sidebar.css | tr -d ' ')" -le 700
! rg -n '(^|[,{[:space:]])\.(sidebar-settings-panel|sidebar-settings-actions|sidebar-group-create|sidebar-config-|sidebar-preview-list|sidebar-settings-summary)' frontend/src/lib/components/shell/sidebar.css
```

Expected before refactor: FAIL because `sidebar.css` has 841 lines and owns Settings sidebar configuration selectors.

- [x] **Step 2: Extract Sidebar Settings CSS chunk**

Move Settings sidebar configuration panel, action, group creation, config item, preview and summary selectors into `frontend/src/lib/pages/settings/widgets/sidebarSettings.css` and import it from `SidebarSettings.svelte`. Keep shell sidebar, rail, navigation and responsive shell selectors in `frontend/src/lib/components/shell/sidebar.css`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/components/shell/sidebar.css | tr -d ' ')" -le 700
test "$(wc -l < frontend/src/lib/pages/settings/widgets/sidebarSettings.css | tr -d ' ')" -le 700
! rg -n '(^|[,{[:space:]])\.(sidebar-settings-panel|sidebar-settings-actions|sidebar-group-create|sidebar-config-|sidebar-preview-list|sidebar-settings-summary)' frontend/src/lib/components/shell/sidebar.css
rg -n '(^|[,{[:space:]])\.(sidebar-settings-panel|sidebar-settings-actions|sidebar-group-create|sidebar-config-|sidebar-preview-list|sidebar-settings-summary)' frontend/src/lib/pages/settings/widgets/sidebarSettings.css
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 20: Shared Panels CSS Ownership Split

**Files:**
- Modify: `frontend/src/lib/components/shared/panels.css`
- Modify: `frontend/src/routes/+layout.svelte`
- Modify: `frontend/src/lib/components/shared/LayoutEditControls.svelte`
- Modify: `frontend/src/lib/components/shared/WidgetEditChrome.svelte`
- Modify: `frontend/src/lib/components/shared/WidgetSettingsDrawer.svelte`
- Modify: `frontend/src/lib/components/shared/AddWidgetDrawer.svelte`
- Modify: `frontend/src/lib/components/shared/HealthStrip.svelte`
- Modify: `frontend/src/lib/components/shared/DraftStrip.svelte`
- Modify: `frontend/src/lib/pages/home/HomePage.svelte`
- Modify: `frontend/src/lib/pages/projects/projects.css`
- Modify: `frontend/src/lib/pages/documents/documents.css`
- Modify: `frontend/src/lib/pages/tasks/tasks.css`
- Modify: `frontend/src/lib/pages/calendar/calendar.css`
- Modify: `frontend/src/lib/pages/persons/persons.css`
- Modify: `frontend/src/lib/pages/communications/communications.css`
- Create: `frontend/src/lib/components/shared/layoutEditControls.css`
- Create: `frontend/src/lib/components/shared/widgetEditChrome.css`
- Create: `frontend/src/lib/components/shared/widgetSettingsDrawer.css`
- Create: `frontend/src/lib/components/shared/addWidgetDrawer.css`
- Create: `frontend/src/lib/components/shared/healthStrip.css`
- Create: `frontend/src/lib/components/shared/draftStrip.css`
- Create: `frontend/src/lib/pages/home/home.css`

- [x] **Step 1: Verify shared panel CSS ownership failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/components/shared/panels.css | tr -d ' ')" -le 1400
! rg -n '(^|[,{[:space:]])\.(layout-edit-controls|widget-edit-chrome|widget-config-button|widget-grid-|widget-drawer|widget-drawer-list|layout-widget-|widget-surface-slider|health-strip|health-chip|draft-strip|draft-chip|draft-open-button|draft-delete-button)' frontend/src/lib/components/shared/panels.css
! rg -n '(^|[,{[:space:]])\.(panel-opacity-|widget-panel-opacity-|panel-blur-|widget-panel-blur-)' frontend/src/lib/components/shared/panels.css
```

Expected before refactor: FAIL because `panels.css` has 1780 lines and owns shared component/editor selectors plus duplicate theme class selectors.

- [x] **Step 2: Extract component-owned and page-owned CSS**

Move layout edit controls, widget edit chrome, widget settings drawer, add widget drawer, health strip and draft strip selectors into component-owned CSS files imported by their Svelte owners. Move Home, Projects, Documents, Tasks, Calendar, Persons and Communications page-owned selectors into page CSS. Keep only shared panel/widget primitives in `panels.css`, and load it from the root layout instead of indirectly through `WidgetEditChrome`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/components/shared/panels.css | tr -d ' ')" -le 700
find frontend/src/lib/components/shared frontend/src/lib/pages -type f -name '*.css' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
! rg -n '(^|[,{[:space:]])\.(layout-edit-controls|widget-edit-chrome|widget-config-button|widget-grid-|widget-drawer|widget-drawer-list|layout-widget-|widget-surface-slider|health-strip|health-chip|draft-strip|draft-chip|draft-open-button|draft-delete-button)' frontend/src/lib/components/shared/panels.css
! rg -n '(^|[,{[:space:]])\.(hero-row|home-metrics|score-ring|feed-list|task-stack|schedule-list|person-list|status-list|full-band|project-card-row|compact-project|new-tile|communication-empty-page|radial-graph|graph-chip|timeline-mini|person-compact|persons-list-panel|inline-metrics|table-head|task-row|task-actions|chart-panel|donut|bar-row|source-footer|source-badge|source-strip|source-card|doc-row|chip|search-hint)' frontend/src/lib/components/shared/panels.css
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 21: App Global CSS Shell/Theme Split

**Files:**
- Modify: `frontend/src/lib/styles/app.css`
- Modify: `frontend/src/routes/+layout.svelte`
- Create: `frontend/src/lib/styles/shell.css`
- Create: `frontend/src/lib/styles/shellTheme.css`

- [x] **Step 1: Verify app CSS ownership failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/styles/app.css | tr -d ' ')" -le 700
! rg -n '(^|[,{[:space:]])\.(desktop-shell|viewport-guard|shell-bg-|theme-accent-|panel-opacity-|widget-panel-opacity-|panel-blur-|widget-panel-blur-)' frontend/src/lib/styles/app.css
```

Expected before refactor: FAIL because `app.css` has 973 lines and owns shell layout, viewport guard, shell background, accent and panel theme selectors.

- [x] **Step 2: Extract shell and theme global CSS chunks**

Move shell layout, viewport guard and shell responsive rules into `frontend/src/lib/styles/shell.css`. Move shell background, brightness, accent and panel surface variable classes into `frontend/src/lib/styles/shellTheme.css`. Import both from `+layout.svelte` before `app.css`, with `shellTheme.css` loaded after `shell.css` so selected theme classes override shell defaults.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/styles/app.css | tr -d ' ')" -le 700
test "$(wc -l < frontend/src/lib/styles/shell.css | tr -d ' ')" -le 700
test "$(wc -l < frontend/src/lib/styles/shellTheme.css | tr -d ' ')" -le 700
! rg -n '(^|[,{[:space:]])\.(desktop-shell|viewport-guard|shell-bg-|theme-accent-|panel-opacity-|widget-panel-opacity-|panel-blur-|widget-panel-blur-)' frontend/src/lib/styles/app.css
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 22: Communications Store Decomposition

**Files:**
- Modify: `frontend/src/lib/stores/communications.ts`
- Create: `frontend/src/lib/stores/communications/state.ts`
- Create: `frontend/src/lib/stores/communications/loaders.ts`
- Create: `frontend/src/lib/stores/communications/compose.ts`
- Create: `frontend/src/lib/stores/communications/actions.ts`
- Create: `frontend/src/lib/stores/communications/selectors.ts`
- Create: `frontend/src/lib/stores/communications/formatters.ts`

- [x] **Step 1: Verify communications store failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/stores/communications.ts | tr -d ' ')" -le 700
test -f frontend/src/lib/stores/communications/state.ts
test -f frontend/src/lib/stores/communications/actions.ts
```

Expected before refactor: FAIL because `communications.ts` has 899 lines and mixes state ownership, loading, sync, compose, message actions and workflow helpers in one store file.

- [x] **Step 2: Extract bounded store modules**

Keep `$lib/stores/communications` as the public import path by replacing `communications.ts` with a facade. Move store state and derived stores into `state.ts`, load/sync/resource functions into `loaders.ts`, compose and draft commands into `compose.ts`, selected-message/workflow commands into `actions.ts`, shared selectors into `selectors.ts`, and exported formatting helpers into `formatters.ts`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/stores/communications.ts | tr -d ' ')" -le 700
find frontend/src/lib/stores/communications -type f -name '*.ts' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
pnpm --dir frontend test:unit src/lib/stores/communications.test.ts src/lib/stores/uiState.test.ts
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 23: Accounts Service Decomposition

**Files:**
- Modify: `frontend/src/lib/services/accounts.ts`
- Create: `frontend/src/lib/services/accounts/calendar.ts`
- Create: `frontend/src/lib/services/accounts/drawer.ts`
- Create: `frontend/src/lib/services/accounts/labels.ts`
- Create: `frontend/src/lib/services/accounts/mailImport.ts`
- Create: `frontend/src/lib/services/accounts/mailSetup.ts`
- Create: `frontend/src/lib/services/accounts/mailWizard.ts`
- Create: `frontend/src/lib/services/accounts/shared.ts`
- Create: `frontend/src/lib/services/accounts/telegram.ts`
- Create: `frontend/src/lib/services/accounts/types.ts`

- [x] **Step 1: Verify accounts service failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/services/accounts.ts | tr -d ' ')" -le 700
test -f frontend/src/lib/services/accounts/mailSetup.ts
test -f frontend/src/lib/services/accounts/telegram.ts
```

Expected before refactor: FAIL because `accounts.ts` had 1011 lines and mixed account drawer state, mail setup, mail import/export, mail wizard, calendar setup, Telegram wizard helpers and account labels in one service file.

- [x] **Step 2: Extract bounded service modules**

Keep `$lib/services/accounts` as the public import path by replacing `accounts.ts` with a facade. Move calendar setup helpers into `calendar.ts`, drawer state helpers into `drawer.ts`, account labels into `labels.ts`, mail import/export/account management into `mailImport.ts`, mail setup API helpers into `mailSetup.ts`, mail wizard presets and inference into `mailWizard.ts`, common string normalization into `shared.ts`, Telegram wizard helpers into `telegram.ts`, and shared account setup boundary types into `types.ts`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/services/accounts.ts | tr -d ' ')" -le 700
find frontend/src/lib/services/accounts -type f -name '*.ts' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
pnpm --dir frontend test:unit src/lib/services/accounts.test.ts
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 24: Communications Service Decomposition

**Files:**
- Modify: `frontend/src/lib/services/communications.ts`
- Create: `frontend/src/lib/services/communications/actions.ts`
- Create: `frontend/src/lib/services/communications/compose.ts`
- Create: `frontend/src/lib/services/communications/constants.ts`
- Create: `frontend/src/lib/services/communications/formatters.ts`
- Create: `frontend/src/lib/services/communications/loaders.ts`
- Create: `frontend/src/lib/services/communications/related.ts`
- Create: `frontend/src/lib/services/communications/rendering.ts`
- Create: `frontend/src/lib/services/communications/resources.ts`
- Create: `frontend/src/lib/services/communications/types.ts`
- Create: `frontend/src/lib/services/communications/workbench.ts`
- Create: `frontend/src/lib/services/communications/workflow.ts`

- [x] **Step 1: Verify communications service failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/services/communications.ts | tr -d ' ')" -le 700
test -f frontend/src/lib/services/communications/rendering.ts
test -f frontend/src/lib/services/communications/compose.ts
```

Expected before refactor: FAIL because `communications.ts` had 1437 lines and mixed communications loading, sync, drafts, send, workflow actions, mail resources, workbench selectors, related-message selectors, message rendering and UI label helpers in one service file.

- [x] **Step 2: Extract bounded service modules**

Keep `$lib/services/communications` as the public import path by replacing `communications.ts` with a facade. Move message actions into `actions.ts`, compose/draft/send helpers into `compose.ts`, shared constants into `constants.ts`, UI label/badge helpers into `formatters.ts`, data loaders and sync helpers into `loaders.ts`, related message selectors into `related.ts`, sanitized mail rendering and remote image proxy helpers into `rendering.ts`, resource rail loaders/summaries into `resources.ts`, shared service types into `types.ts`, workbench/account selectors into `workbench.ts`, and workflow command helpers into `workflow.ts`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/services/communications.ts | tr -d ' ')" -le 700
find frontend/src/lib/services/communications -type f -name '*.ts' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
pnpm --dir frontend test:unit src/lib/services/communications.test.ts src/lib/stores/communications.test.ts src/lib/stores/uiState.test.ts
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 25: Telegram Service Decomposition

**Files:**
- Modify: `frontend/src/lib/services/telegram.ts`
- Create: `frontend/src/lib/services/telegram/automation.ts`
- Create: `frontend/src/lib/services/telegram/calls.ts`
- Create: `frontend/src/lib/services/telegram/constants.ts`
- Create: `frontend/src/lib/services/telegram/fixtures.ts`
- Create: `frontend/src/lib/services/telegram/lifecycle.ts`
- Create: `frontend/src/lib/services/telegram/messages.ts`
- Create: `frontend/src/lib/services/telegram/parsing.ts`
- Create: `frontend/src/lib/services/telegram/runtime.ts`
- Create: `frontend/src/lib/services/telegram/selection.ts`
- Create: `frontend/src/lib/services/telegram/types.ts`
- Create: `frontend/src/lib/services/telegram/wizard.ts`
- Create: `frontend/src/lib/services/telegram/workspace.ts`

- [x] **Step 1: Verify Telegram service failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/services/telegram.ts | tr -d ' ')" -le 700
test -f frontend/src/lib/services/telegram/workspace.ts
test -f frontend/src/lib/services/telegram/wizard.ts
test -f frontend/src/lib/services/telegram/messages.ts
```

Expected before refactor: FAIL because `telegram.ts` had 1584 lines and mixed workspace loading, account lifecycle, runtime sync, QR wizard setup, fixtures, manual send, automation, calls, selection helpers and Telegram workbench model helpers in one service file.

- [x] **Step 2: Extract bounded service modules**

Keep `$lib/services/telegram` as the public import path by replacing `telegram.ts` with a facade. Move automation template/policy/dry-run helpers into `automation.ts`, call/transcript helpers into `calls.ts`, constants into `constants.ts`, fixture/manual-send commands into `fixtures.ts`, account lifecycle helpers into `lifecycle.ts`, chat/message/attachment/link selectors into `messages.ts`, JSON parsing helpers into `parsing.ts`, runtime sync/media commands into `runtime.ts`, selected chat/call form mapping into `selection.ts`, exported UI model types into `types.ts`, QR/account wizard helpers into `wizard.ts`, and workspace loading/runtime status assembly into `workspace.ts`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/services/telegram.ts | tr -d ' ')" -le 700
find frontend/src/lib/services/telegram -type f -name '*.ts' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
pnpm --dir frontend test:unit src/lib/services/telegram.test.ts
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 26: API Contract Types Decomposition

**Files:**
- Modify: `frontend/src/lib/api/types.ts`
- Create: `frontend/src/lib/api/types/accounts.ts`
- Create: `frontend/src/lib/api/types/ai.ts`
- Create: `frontend/src/lib/api/types/calendar.ts`
- Create: `frontend/src/lib/api/types/communication.ts`
- Create: `frontend/src/lib/api/types/contradictions.ts`
- Create: `frontend/src/lib/api/types/decisions.ts`
- Create: `frontend/src/lib/api/types/documents.ts`
- Create: `frontend/src/lib/api/types/graph.ts`
- Create: `frontend/src/lib/api/types/mail.ts`
- Create: `frontend/src/lib/api/types/obligations.ts`
- Create: `frontend/src/lib/api/types/organizations.ts`
- Create: `frontend/src/lib/api/types/persons.ts`
- Create: `frontend/src/lib/api/types/projects.ts`
- Create: `frontend/src/lib/api/types/relationships.ts`
- Create: `frontend/src/lib/api/types/settings.ts`
- Create: `frontend/src/lib/api/types/tasks.ts`
- Create: `frontend/src/lib/api/types/telegram.ts`
- Create: `frontend/src/lib/api/types/vault.ts`
- Create: `frontend/src/lib/api/types/whatsapp.ts`

- [x] **Step 1: Verify API contract type failure**

Run:

```sh
test "$(wc -l < frontend/src/lib/api/types.ts | tr -d ' ')" -le 700
test -d frontend/src/lib/api/types
find frontend/src/lib/api -maxdepth 2 -type f -name '*.ts' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
```

Expected before refactor: FAIL because `types.ts` had 2615 lines and mixed all API contract types in one source file.

- [x] **Step 2: Extract bounded contract modules**

Keep `$lib/api/types` and `$lib/api` as public import paths by replacing `types.ts` with a facade. Move contract types into bounded modules for account setup, AI, calendar, communication, graph, mail, persona, projects, tasks, Telegram, WhatsApp and related domains. Preserve the exported type/const name set from the original committed file.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < frontend/src/lib/api/types.ts | tr -d ' ')" -le 700
find frontend/src/lib/api/types -type f -name '*.ts' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
comm -3 <(git show HEAD:frontend/src/lib/api/types.ts | rg '^export (type|const) ' | sed -E 's/^export (type|const) ([A-Za-z0-9_]+).*/\1 \2/' | sort) <(rg '^export (type|const) ' frontend/src/lib/api/types.ts frontend/src/lib/api/types/*.ts | sed -E 's/^.*:export (type|const) ([A-Za-z0-9_]+).*/\1 \2/' | sort)
pnpm --dir frontend lint:ts
make lint-frontend
make lint-architecture
```

### Task 27: Telegram TDLib JSON Boundary Decomposition

**Files:**
- Modify: `backend/src/integrations/telegram/tdjson.rs`
- Create: `backend/src/integrations/telegram/tdjson/client.rs`
- Create: `backend/src/integrations/telegram/tdjson/identifiers.rs`
- Create: `backend/src/integrations/telegram/tdjson/library_paths.rs`
- Create: `backend/src/integrations/telegram/tdjson/parsing.rs`
- Create: `backend/src/integrations/telegram/tdjson/qr_login.rs`
- Create: `backend/src/integrations/telegram/tdjson/qr_login_support.rs`
- Create: `backend/src/integrations/telegram/tdjson/requests.rs`
- Create: `backend/src/integrations/telegram/tdjson/snapshots.rs`

- [x] **Step 1: Verify TDLib JSON boundary failure**

Run:

```sh
test "$(wc -l < backend/src/integrations/telegram/tdjson.rs | tr -d ' ')" -le 700
test -d backend/src/integrations/telegram/tdjson
```

Expected before refactor: FAIL because `tdjson.rs` had 2361 lines and mixed TDLib dynamic library loading, FFI client wrapper, QR-login worker state, request construction, response parsing, snapshot DTOs and tests.

- [x] **Step 2: Extract bounded TDLib modules**

Keep `crate::integrations::telegram::tdjson` as the public crate-local import path by replacing `tdjson.rs` with a facade. Move FFI/client ownership into `client.rs`, library candidate discovery into `library_paths.rs`, TDLib request builders into `requests.rs`, response parsing and error helpers into `parsing.rs`, QR-login orchestration into `qr_login.rs`, QR-login state/response helpers into `qr_login_support.rs`, shared path identifiers into `identifiers.rs`, and TDLib snapshot DTOs into `snapshots.rs`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < backend/src/integrations/telegram/tdjson.rs | tr -d ' ')" -le 700
find backend/src/integrations/telegram/tdjson -type f -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
cargo fmt --manifest-path backend/Cargo.toml --check
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml tdjson
make backend-validate
make lint-architecture
```

### Task 28: Telegram Client Store Boundary Decomposition

**Files:**
- Modify: `backend/src/integrations/telegram/client.rs`
- Create: `backend/src/integrations/telegram/client/accounts.rs`
- Create: `backend/src/integrations/telegram/client/chats.rs`
- Create: `backend/src/integrations/telegram/client/errors.rs`
- Create: `backend/src/integrations/telegram/client/identifiers.rs`
- Create: `backend/src/integrations/telegram/client/messages.rs`
- Create: `backend/src/integrations/telegram/client/models.rs`
- Create: `backend/src/integrations/telegram/client/projection.rs`
- Create: `backend/src/integrations/telegram/client/rows.rs`
- Create: `backend/src/integrations/telegram/client/store.rs`
- Create: `backend/src/integrations/telegram/client/tests.rs`
- Create: `backend/src/integrations/telegram/client/validation.rs`
- Create: `backend/src/integrations/telegram/client/vault.rs`

- [x] **Step 1: Verify Telegram client boundary failure**

Run:

```sh
test "$(wc -l < backend/src/integrations/telegram/client.rs | tr -d ' ')" -le 700
test -d backend/src/integrations/telegram/client
```

Expected before refactor: FAIL because `client.rs` had 1793 lines and mixed provider account setup, credential binding, lifecycle state changes, chat/message ingestion, projection, row mapping, identifiers, validation, error types and tests.

- [x] **Step 2: Extract bounded client modules**

Keep `crate::integrations::telegram::client` as the public crate-local import path by replacing `client.rs` with a facade. Move account setup/lifecycle and credential binding into `accounts.rs`, chat operations into `chats.rs`, message ingestion/query helpers into `messages.rs`, public DTOs into `models.rs`, projection into `projection.rs`, row mappers into `rows.rs`, stable identifiers into `identifiers.rs`, validation helpers into `validation.rs`, secret-vault boundary into `vault.rs`, and tests into `tests.rs`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < backend/src/integrations/telegram/client.rs | tr -d ' ')" -le 700
find backend/src/integrations/telegram/client -type f -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
cargo fmt --manifest-path backend/Cargo.toml --check
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml integrations::telegram::client
make backend-validate
make lint-architecture
```

### Task 29: Telegram Runtime Boundary Decomposition

**Files:**
- Modify: `backend/src/integrations/telegram/runtime.rs`
- Modify: `backend/src/integrations/telegram/api.rs`
- Create: `backend/src/integrations/telegram/runtime/actor.rs`
- Create: `backend/src/integrations/telegram/runtime/commands.rs`
- Create: `backend/src/integrations/telegram/runtime/manager.rs`
- Create: `backend/src/integrations/telegram/runtime/media.rs`
- Create: `backend/src/integrations/telegram/runtime/models.rs`
- Create: `backend/src/integrations/telegram/runtime/state.rs`
- Create: `backend/src/integrations/telegram/runtime/status.rs`
- Create: `backend/src/integrations/telegram/runtime/tests.rs`
- Create: `backend/src/integrations/telegram/runtime/validation.rs`

- [x] **Step 1: Verify Telegram runtime boundary failure**

Run:

```sh
test "$(wc -l < backend/src/integrations/telegram/runtime.rs | tr -d ' ')" -le 700
test -d backend/src/integrations/telegram/runtime
```

Expected before refactor: FAIL because `runtime.rs` had 1538 lines and mixed runtime manager API, account-scoped actor state, command dispatch, TDLib actor loop, media persistence, request/response DTOs, status assembly, validation and tests.

- [x] **Step 2: Extract bounded runtime modules**

Keep `crate::integrations::telegram::runtime` as the public crate-local import path by replacing `runtime.rs` with a facade. Move the manager facade and public orchestration methods into `manager.rs`, TDLib actor loop and native client operations into `actor.rs`, actor command request wrappers into `commands.rs`, media persistence into `media.rs`, public runtime DTOs into `models.rs`, internal actor state into `state.rs`, account/status helpers into `status.rs`, validation helpers into `validation.rs`, and runtime tests into `tests.rs`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < backend/src/integrations/telegram/runtime.rs | tr -d ' ')" -le 700
find backend/src/integrations/telegram/runtime -type f -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
cargo fmt --manifest-path backend/Cargo.toml --check
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml integrations::telegram::runtime
make backend-validate
make lint-architecture
```

### Task 30: Mail Background Sync Boundary Decomposition

**Files:**
- Modify: `backend/src/domains/communications/background_sync.rs`
- Create: `backend/src/domains/communications/background_sync/errors.rs`
- Create: `backend/src/domains/communications/background_sync/models.rs`
- Create: `backend/src/domains/communications/background_sync/provider.rs`
- Create: `backend/src/domains/communications/background_sync/rows.rs`
- Create: `backend/src/domains/communications/background_sync/service.rs`
- Create: `backend/src/domains/communications/background_sync/store.rs`
- Create: `backend/src/domains/communications/background_sync/validation.rs`

- [x] **Step 1: Verify mail background sync boundary failure**

Run:

```sh
test "$(wc -l < backend/src/domains/communications/background_sync.rs | tr -d ' ')" -le 700
test -d backend/src/domains/communications/background_sync
```

Expected before refactor: FAIL because `background_sync.rs` had 1684 lines and mixed run orchestration, provider network sync, SQL persistence, row mapping, DTOs, errors and validation helpers.

- [x] **Step 2: Extract bounded background sync modules**

Keep `crate::domains::communications::background_sync` as the public import path by replacing `background_sync.rs` with a facade. Move lifecycle orchestration into `service.rs`, provider network/projection loops into `provider.rs`, SQL persistence into `store.rs`, public DTOs and internal run models into `models.rs`, error enums into `errors.rs`, SQL row mapping into `rows.rs`, and validation/helper functions into `validation.rs`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < backend/src/domains/communications/background_sync.rs | tr -d ' ')" -le 700
find backend/src/domains/communications/background_sync -type f -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
cargo fmt --manifest-path backend/Cargo.toml --check
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml mail_sync
make backend-validate
make lint-architecture
```

### Task 31: Calendar Handler Boundary Decomposition

**Files:**
- Modify: `backend/src/domains/calendar/handlers/mod.rs`
- Create: `backend/src/domains/calendar/handlers/accounts.rs`
- Create: `backend/src/domains/calendar/handlers/analytics.rs`
- Create: `backend/src/domains/calendar/handlers/brain.rs`
- Create: `backend/src/domains/calendar/handlers/events.rs`
- Create: `backend/src/domains/calendar/handlers/health.rs`
- Create: `backend/src/domains/calendar/handlers/intelligence.rs`
- Create: `backend/src/domains/calendar/handlers/meetings.rs`
- Create: `backend/src/domains/calendar/handlers/reminders.rs`
- Create: `backend/src/domains/calendar/handlers/rules.rs`
- Create: `backend/src/domains/calendar/handlers/scheduling.rs`
- Create: `backend/src/domains/calendar/handlers/search.rs`
- Create: `backend/src/domains/calendar/handlers/sync.rs`

- [x] **Step 1: Verify calendar handler boundary failure**

Run:

```sh
test "$(wc -l < backend/src/domains/calendar/handlers/mod.rs | tr -d ' ')" -le 700
test "$(find backend/src/domains/calendar/handlers -maxdepth 1 -type f -name '*.rs' ! -name mod.rs | wc -l | tr -d ' ')" -gt 0
```

Expected before refactor: FAIL because `handlers/mod.rs` had 1674 lines and mixed calendar accounts, sources, event CRUD, event context, intelligence, meetings, scheduling, health/watchtower, brain/search, rules, import/export, reminders and analytics handlers.

- [x] **Step 2: Extract bounded handler modules**

Keep `crate::domains::calendar::handlers::*` stable for the app router by leaving `mod.rs` as a crate-local facade. Move handlers by documented Calendar domain responsibility: accounts/sources into `accounts.rs`, event CRUD/context into `events.rs`, classify/analyze/risk into `intelligence.rs`, meeting artifacts into `meetings.rs`, deadlines/focus/smart schedule into `scheduling.rs`, watchtower/health into `health.rs`, brief/brain into `brain.rs`, search into `search.rs`, rules into `rules.rs`, import/export/sync into `sync.rs`, reminders into `reminders.rs`, and analytics into `analytics.rs`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < backend/src/domains/calendar/handlers/mod.rs | tr -d ' ')" -le 700
find backend/src/domains/calendar/handlers -maxdepth 1 -type f -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
cargo fmt --manifest-path backend/Cargo.toml --check
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml calendar
make backend-validate
make lint-architecture
```

### Task 32: Shared API Support Boundary Decomposition

**Files:**
- Modify: `backend/src/domains/api_support.rs`
- Create: `backend/src/domains/api_support/automation_calls.rs`
- Create: `backend/src/domains/api_support/communications.rs`
- Create: `backend/src/domains/api_support/formatting.rs`
- Create: `backend/src/domains/api_support/messaging_integrations.rs`
- Create: `backend/src/domains/api_support/platform_dtos.rs`
- Create: `backend/src/domains/api_support/query_parsing.rs`
- Create: `backend/src/domains/api_support/review_commands.rs`
- Create: `backend/src/domains/api_support/review_lists.rs`
- Create: `backend/src/domains/api_support/stores.rs`

- [x] **Step 1: Verify API support boundary failure**

Run:

```sh
test "$(wc -l < backend/src/domains/api_support.rs | tr -d ' ')" -le 700
test -d backend/src/domains/api_support
```

Expected before refactor: FAIL because `api_support.rs` had 1762 lines and mixed store/service factories, application/event DTOs, communication response conversion, Telegram/WhatsApp capability responses, automation/call DTOs, review commands, query parsers, validation helpers and formatting helpers.

- [x] **Step 2: Extract bounded API support modules**

Keep `crate::domains::api_support::*` stable for existing route modules by leaving `api_support.rs` as a crate-local facade. Move state/store helpers into `stores.rs`, application/event/status DTOs into `platform_dtos.rs`, communication DTO conversion into `communications.rs`, candidate list DTOs into `review_lists.rs`, Telegram/WhatsApp API support into `messaging_integrations.rs`, automation/call DTOs into `automation_calls.rs`, review command DTOs into `review_commands.rs`, query parsing and validation into `query_parsing.rs`, and shared formatting/default helpers into `formatting.rs`.

- [x] **Step 3: Validate**

Run:

```sh
test "$(wc -l < backend/src/domains/api_support.rs | tr -d ' ')" -le 700
find backend/src/domains/api_support -type f -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 700 { print; failed=1 } END { exit failed ? 1 : 0 }'
cargo fmt --manifest-path backend/Cargo.toml --check
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml api
make backend-validate
make lint-architecture
```

### Task 33: Continue Backend God File Elimination

**Files:**
- Refactor one file at a time from the current over-700 list.
- Preserve route contracts only when still aligned with current docs.
- Add or update tests around the moved behavior before implementation changes.

- [ ] **Step 1: Pick the next file by blast radius and documentation priority**

Prefer files in mail, Telegram, app routing and API error boundaries before unrelated engines.

- [ ] **Step 2: Write a failing structural or behavioral check**

Use a threshold command for pure decomposition or a targeted unit/integration test for behavior.

- [ ] **Step 3: Split by bounded context**

Keep new files below 700 lines and avoid moving unrelated responsibilities together.

- [ ] **Step 4: Validate**

Run the narrow target first, then `make backend-fmt-check`, `make backend-check`, and the relevant tests.

## Self-Review

- Spec coverage: this plan covers the architecture precondition from the user objective and ADR-0078/ADR-0079. It does not claim mail or Telegram parity is complete.
- Placeholder scan: no task uses `TBD`, `TODO`, or unspecified commands.
- Type consistency: file paths and commands match the current repository layout and Makefile/package scripts.
