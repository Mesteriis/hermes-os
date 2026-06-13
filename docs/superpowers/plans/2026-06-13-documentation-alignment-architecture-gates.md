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
- `docs/adr/ADR-0078-frontend-component-decomposition.md`
- `docs/adr/ADR-0079-script-logic-decomposition.md`
- `docs/adr/ADR-0083-telegram-live-user-client-runtime.md`
- `docs/adr/ADR-0091-telegram-production-client-capability-model.md`
- `docs/adr/ADR-0092-mail-provider-capability-tiers.md`
- `docs/domains/telegram-channel.md`
- `docs/mail/README.md`

## Current Stop-Factors

- Backend source files over 700 lines still exist outside the first completed slice.
- Frontend Svelte components over 500 lines still exist: `AccountSetupModal.svelte`, `TelegramPage.svelte`, `AISettingsControlCenter.svelte`, `CommunicationsMessageDetail.svelte`.
- Large shared CSS files still exist: `pages.css`, `panels.css`, `app.css`, `sidebar.css`.
- Feature parity work for mail and Telegram must not add code to these files until the relevant file/component is decomposed.

### Task 1: Mail Handler God File Decomposition

**Files:**
- Modify: `backend/src/domains/mail/handlers/mod.rs`
- Create: `backend/src/domains/mail/handlers/account_management.rs`
- Create: `backend/src/domains/mail/handlers/account_setup.rs`
- Create: `backend/src/domains/mail/handlers/account_support.rs`
- Create: `backend/src/domains/mail/handlers/communication_messages.rs`
- Create: `backend/src/domains/mail/handlers/communication_queries.rs`
- Create: `backend/src/domains/mail/handlers/finance_analytics.rs`
- Create: `backend/src/domains/mail/handlers/legal_export.rs`
- Create: `backend/src/domains/mail/handlers/message_actions.rs`
- Create: `backend/src/domains/mail/handlers/sending.rs`
- Create: `backend/src/domains/mail/handlers/templates_status.rs`
- Create: `backend/src/domains/mail/handlers/workflow_state.rs`

- [x] **Step 1: Verify the stop-factor fails**

Run:

```sh
test "$(wc -l < backend/src/domains/mail/handlers/mod.rs | tr -d ' ')" -le 700
```

Expected before refactor: FAIL because the file has 3000 lines.

- [x] **Step 2: Split bounded handler groups**

Move contiguous handler groups into sibling modules and keep `pub(crate)` route handlers re-exported from `mod.rs`.

The resulting `backend/src/domains/mail/handlers/mod.rs` owns module assembly and imports only. Handler modules own one bounded route group each.

- [x] **Step 3: Verify the target files are below threshold**

Run:

```sh
find backend/src/domains/mail/handlers -maxdepth 1 -type f -name '*.rs' -print0 \
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
git add backend/src/domains/mail/handlers docs/superpowers/plans/2026-06-13-documentation-alignment-architecture-gates.md IMPLEMENTATION_STATUS.md
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

Keep Telegram actions capability-gated per ADR-0091 and `docs/domains/telegram-channel.md`.

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

### Task 10: Continue Backend God File Elimination

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
