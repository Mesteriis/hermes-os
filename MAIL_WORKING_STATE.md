# Mail Working State

Date: 2026-06-13
Owner: autonomous Codex goal run

This is the live root tracker for bringing Hermes mail to a working state.
Hermes remains a Personal Memory System; mail is the Communications ingestion
spine and evidence source, not a standalone email-client product.

## Audit Sources

- `git status --short`
- `docs/adr/ADR-0041-email-provider-ingestion-foundation.md`
- `docs/adr/ADR-0055-full-email-provider-networking.md`
- `docs/adr/ADR-0080-mail-background-sync-progress-local-trash.md`
- `docs/mail/status.md`
- `docs/mail/api.md`
- `docs/mail/blockers.md`
- `backend/src/app/router.rs`
- `backend/src/domains/mail/*`
- `backend/migrations/0005_create_communication_ingestion.sql`
- `backend/migrations/0055_mail_sync_local_trash.sql`
- `frontend/src/lib/api/endpoints/communications.ts`
- `frontend/src/lib/api/endpoints/accounts.ts`
- `frontend/src/lib/services/communications.ts`
- `frontend/src/lib/stores/communications.ts`
- `frontend/src/lib/components/shared/AccountSetupModal.svelte`

The worktree was already dirty before this mail run. Existing unrelated
persona, relationship, engine, Telegram, WhatsApp, calendar and task changes
must not be reverted or mixed into the mail implementation.

## ADR Impact

Relevant existing ADR:

- ADR-0041: provider-neutral email ingestion storage boundary.
- ADR-0042 / ADR-0076: account credentials are secret references resolved from
  the host vault; PostgreSQL stores metadata only.
- ADR-0046: mail bytes and attachment bytes stay out of PostgreSQL.
- ADR-0055: full email provider networking is allowed, but provider writes
  require explicit user action.
- ADR-0056: protected local API routes use `X-Hermes-Secret`.
- ADR-0077: new UI strings need English keys and Russian translations.
- ADR-0080: UI delete is local trash/tombstone behavior, not provider delete.

New ADR:

- ADR-0092: mail provider capability tiers. This is needed because POP3,
  Exchange/Microsoft 365, Proton, Gmail API and IMAP/SMTP providers do not share
  the same folder, flag, auth or write semantics.

## Current Capability Map

Legend:

- Working: implemented in backend and reachable from current UI/service layer.
- Backend: implemented or partially implemented in backend, UI incomplete.
- Partial: some behavior exists but important requested semantics are missing.
- Missing: no verified implementation found.
- Blocked: explicitly needs external infrastructure or a new adapter decision.

| Area | Status | Verified Current State | Gap To Working State |
|---|---|---|---|
| Add account | Working | Gmail OAuth start/complete; iCloud/generic IMAP setup; shared account setup modal with iCloud, Microsoft 365 IMAP, Fastmail, Mail.ru, Yandex, Proton Bridge, Yahoo, AOL and manual IMAP presets. | Native provider-specific adapters remain incomplete outside Gmail read. |
| Delete account | Working | Protected `DELETE /api/v1/email-accounts/{account_id}` deletes unused account metadata only. | Accounts with retained raw records/messages return conflict and remain preserved. |
| Authorization | Partial | Gmail OAuth; IMAP app/password credentials behind secret boundary; local logout disables sync and marks account logged_out. | Gmail send scopes absent; no Microsoft OAuth; no provider-side OAuth revoke surface. |
| Logout | Working | `POST /api/v1/email-accounts/{account_id}/logout` marks account config `auth_state=logged_out` and disables sync. | Provider-side OAuth revoke is still future provider-adapter work. |
| Multiple accounts | Working | Provider accounts table, settings account list and Settings / Integrations inspector support multiple mail accounts. | More bulk account operations remain future work. |
| Account switching | Working | Communications store has selected mail account and account selector. | Needs more explicit empty/error states for unavailable capabilities. |
| Import settings | Working | `POST /api/v1/email-accounts/import` upserts sanitized account metadata and sync settings; Settings / Integrations exposes JSON import. | Secret payloads and secret refs are rejected; credentials must be reconnected through account setup. |
| Export settings | Working | `GET /api/v1/email-accounts/{account_id}/export` returns sanitized config, capabilities and sync settings; Settings / Integrations exports JSON. | Export intentionally omits credentials and secret references. |
| OAuth2 | Partial | Gmail OAuth setup exists. | Scope/capability model incomplete for send and Microsoft 365. |
| IMAP | Working | iCloud/generic IMAP setup, sync, local cache. | Folder discovery is not first-class. |
| POP3 | Missing | No provider kind or adapter found. | Needs separate ADR/migration because POP3 has weaker mailbox semantics. |
| SMTP | Partial | IMAP-backed SMTP setup, capability detection and send route exist; fixed provider presets now include SMTP host/port/TLS/STARTTLS. | Outbox, retry queue and delivery tracking incomplete. |
| Exchange / Microsoft 365 | Partial | Wizard labels Microsoft 365 / Exchange Online as IMAP/SMTP preset against Office 365 hosts. | Need Microsoft Graph/EWS adapter and OAuth before native Exchange support can be called complete. |
| Gmail | Partial | OAuth setup and Gmail read sync exist; frontend says Gmail send unavailable. | Need write scopes and Gmail send/draft mutation implementation exposed. |
| Fastmail / Mail.ru / Yandex | Working | Wizard presets create standard IMAP/SMTP accounts with provider-specific hosts, ports and app-password credential kind. | No native provider APIs; advanced folder/label semantics depend on IMAP behavior. |
| Proton | Partial | Wizard has Proton Bridge preset using local IMAP/SMTP bridge ports. | Requires user-running Proton Bridge; direct Proton credential handling remains intentionally unsupported. |
| Mailboxes/folders | Partial | Workflow tabs, unified list, active/trash local state. | No first-class provider folder model, custom folders, virtual folders, unified sent. |
| Messages list/detail | Working | `/api/v1/communications/messages` list/detail with attachments. | Folder/account/date/attachment query filters need expansion. |
| Receive mail | Partial | Provider sync pipeline and background/manual sync exist. | Live scheduling robustness and provider folder coverage need validation. |
| Send/reply/reply-all/forward | Partial | Routes exist for send, reply, reply-all, forward, forward-eml. | Gmail send unavailable; outbox/delivery status missing. |
| Drafts | Partial | Draft create/list/detail/delete and compose form exist. | Autosave/recovery/scheduled send not verified in UI. |
| Delete/restore | Working | Local trash/restore routes exist per ADR-0080. | Provider delete must remain explicit and separate. |
| Archive/important/read flags | Partial | Workflow archive, local pin/mute/labels, local important flag and explicit Mark read / Mark unread UI exist; mark unread is constrained to `reviewed -> new`. | Move/copy and provider folder mutations still need coherent UI/API semantics. |
| Original message | Partial | EML export and raw blob architecture exist. | "Open original" UI and raw source viewer not verified. |
| Compose attachments/images/links/signature/templates | Partial | Compose and template backend exist. | File insertion, image insertion, signature selection and rich template editor incomplete. |
| Recipients/autocomplete/contacts/groups/recent | Partial | Personas and smart CC exist. | Recipient autocomplete, groups and recents not verified. |
| Threads | Partial | Thread list and messages routes exist; navigator has thread mode. | Expand/collapse/latest/history controls need explicit UI. |
| Attachments | Partial | Attachment metadata, blob storage, duplicate detection and scanner boundary exist. | Preview/open/save all/export/share and archive/OCR scanning incomplete. |
| Search and filters | Partial | `/communications/search` and local frontend search exist. | Sender/recipient/attachment/folder/account/date/full-text filter matrix incomplete. |
| Labels | Partial | Add/remove message label routes exist. | Label CRUD, color, rename and list UI incomplete. |
| Rules | Backend | `email_rules` table and rule module exist. | Rule editor/apply/disable UI not verified. |
| Notifications | Missing | No mail notification subsystem found. | Need desktop notification policy and error delivery events. |
| Signatures | Backend | `mail/signatures.rs` exists. | CRUD and account selection UI not verified. |
| Contacts | Partial | Personas/identity traces and settings contacts provider grouping exist. | Contact CRUD/import/export is Persona-domain work, not mail-only. |
| Calendar ICS/invites | Partial | Calendar domain exists; mail attachment ICS route not verified. | Need ICS preview and invitation response flow. |
| Export | Partial | Message export supports md/eml/json and the frontend downloads the returned file with a sanitized filename. | Folder/thread/MBOX/PDF export incomplete. |
| Synchronization/offline | Partial | Local cache, manual sync, settings and run history exist. | Durable offline outbox and reconnect queue missing. |
| Soft delete | Working | Local trash with changed_at and reason exists. | Need richer tombstone reasons and UI display for all delete causes. |
| Message history | Partial | Workflow/local state timestamps exist. | Event timeline for received/read/replied/forwarded/restored incomplete. |
| Knowledge lifecycle | Partial | Mail projects to personas/orgs/graph and engines. | Explicit NEW/INDEXED/ANALYZED/etc lifecycle state not verified. |
| Deletion restrictions | Partial | Raw records append-only; local trash preserves data. | UI warning for knowledge loss and linked object list incomplete. |
| Security | Partial | SPF/DKIM/DMARC parsing and signature detection routes exist. | S/MIME/PGP cryptographic verification is blocked by external crypto tooling. |
| Desktop UX | Partial | Desktop-only Svelte/Tauri surface exists. | Multi-window, tray, shortcuts, context menu and drag/drop incomplete. |
| Hermes-specific links | Partial | Project/task/note/person workflows exist in Communications context. | Explicit link/unlink actions for all target domains incomplete. |
| Mail intelligence without AI | Partial | Heuristic intelligence, invoices, legal docs, subscriptions, risks exist. | Requested flags need normalized non-AI state names and UI filters. |

## Immediate Implementation Slice

Implemented in this run slice:

1. Added protected backend mail account management endpoints:
   - list accounts;
   - get account;
   - delete account when no retained evidence would be orphaned;
   - logout/disable account by turning sync off and marking account config;
   - export sanitized settings without secrets;
   - import sanitized settings without secret payloads.
2. Added backend tests first and verified RED before implementation.
3. Exposed matching frontend API endpoint helpers and service helpers.
4. Added frontend endpoint/service tests.
5. Added frontend mail-provider presets for Fastmail, Mail.ru, Yandex,
   Proton Bridge, Microsoft 365 / Exchange Online, Yahoo and AOL; SMTP
   settings are now carried through the IMAP setup request.
6. Exposed account management actions in Settings / Integrations:
   sanitized settings export, sanitized settings import, local logout and
   metadata delete with explicit confirmation.
7. Added explicit Mark read / Mark unread message actions using the existing
   workflow-state contract. Backend now permits `reviewed -> new` only for the
   local unread toggle; other action states are not silently erased.
8. Added a local important toggle:
   - backend `POST /api/v1/communications/messages/{message_id}/important`;
   - `message_metadata.important` persistence through `MessageFlags`;
   - frontend API/service/store wiring;
   - detail-pane button with Russian translations;
   - backend HTTP regression and frontend endpoint regression tests.
9. Made message export usable from the frontend:
   - export results now create a browser file download;
   - unsafe filename characters are sanitized;
   - non-DOM environments keep the existing `Export ready` fallback.

Non-goals for this slice:

- No POP3 provider kind migration.
- No Microsoft Graph/EWS adapter.
- No Proton direct login.
- No provider-side destructive delete.
- No broad frontend redesign while the current dirty worktree contains
  unrelated changes.

## Validation Log

- Ran: `cargo test --manifest-path backend/Cargo.toml --test email_account_management_api -- --nocapture`
  Result: passed, 3 tests.
- Ran: `pnpm vitest run src/lib/api/endpoints/accounts.test.ts src/lib/services/accounts.test.ts`
  Result: passed, 2 files / 16 tests.
- Ran: `rustfmt --edition 2024 --check backend/src/domains/mail/core.rs backend/src/domains/mail/handlers/mod.rs backend/src/app/router.rs backend/src/app/error.rs backend/tests/email_account_management_api.rs`
  Result: passed.
- Ran: `cargo check --manifest-path backend/Cargo.toml`
  Result: passed.
- Ran: `pnpm lint:ts`
  Result: passed, 0 errors and 0 warnings.
- Ran: `pnpm lint`
  Result: passed, style policy plus Svelte/TypeScript checks.
- Ran: `pnpm vitest run src/lib/services/communications.test.ts src/lib/services/accounts.test.ts src/lib/api/endpoints/accounts.test.ts`
  Result: passed, 3 files / 35 tests.
- Ran: `cargo test --manifest-path backend/Cargo.toml --test messages workflow_state_valid_transitions -- --nocapture`
  Result: passed, 1 test.
- Ran: `rustfmt --edition 2024 --check backend/src/domains/mail/messages.rs backend/tests/messages.rs`
  Result: passed.
- Browser check: opened `http://localhost:5173` in the in-app Browser; page loaded
  without console errors but redirected to `/auth`, so account-modal interaction
  could not be verified without a local user session.
- Ran: `git diff --check`
  Result: passed.
- Ran: `pnpm vitest run src/lib/services/communications.test.ts`
  Result: passed, 1 file / 20 tests.
- Ran: `pnpm lint`
  Result: passed, style policy plus Svelte/TypeScript checks.
- Ran: `cargo test --manifest-path backend/Cargo.toml --test message_flags_api -- --nocapture`
  Result: passed, 1 test.
- Ran: `cargo test --manifest-path backend/Cargo.toml --test email_account_management_api -- --nocapture`
  Result: passed, 3 tests.
- Ran: `cargo test --manifest-path backend/Cargo.toml --lib domains::mail::flags::tests::is_important_detects_flag -- --nocapture`
  Result: passed, 1 test.
- Ran: `cargo test --manifest-path backend/Cargo.toml --test messages workflow_state_valid_transitions -- --nocapture`
  Result: passed, 1 test.
- Ran: `pnpm vitest run src/lib/api/endpoints/communications.test.ts src/lib/services/communications.test.ts src/lib/services/accounts.test.ts src/lib/api/endpoints/accounts.test.ts`
  Result: passed, 4 files / 36 tests.
- Ran: `pnpm lint`
  Result: passed, style policy plus Svelte/TypeScript checks.
- Ran: `rustfmt --edition 2024 --check backend/src/domains/mail/core.rs backend/src/domains/mail/handlers/mod.rs backend/src/app/router.rs backend/src/app/error.rs backend/src/domains/mail/messages.rs backend/src/domains/mail/flags.rs backend/tests/messages.rs backend/tests/email_account_management_api.rs backend/tests/message_flags_api.rs`
  Result: passed.
- Ran: `cargo check --manifest-path backend/Cargo.toml`
  Result: passed.
- Ran: `git diff --check`
  Result: passed.
