# WhatsApp Current Audit — 2026-06-24

Status: current repository audit based on `hermes-os-main (3).zip`.

This document records what exists now before starting the full WhatsApp target implementation. It intentionally separates:

- implemented fixture/runtime foundation;
- provider-neutral Communications foundation already present;
- missing production WhatsApp functionality;
- architectural risks that must not be bypassed.

## Executive summary

WhatsApp is already present as a **provider/runtime foundation**, not as a full channel implementation.

The current repository contains:

- WhatsApp documentation under `docs/integrations/whatsapp/`;
- ADR-0051 for the WhatsApp Web companion boundary;
- ADR-0097 for channel-to-integration ownership;
- backend integration skeleton under `backend/src/integrations/whatsapp/`;
- runtime/setup routes under `/api/v1/integrations/whatsapp/*`;
- provider-neutral communication read route support through `/api/v1/communications/messages`;
- PostgreSQL migrations for `whatsapp_web` provider kind, session state, observation kind and canonical Communication tables;
- frontend runtime/setup panel under `frontend/src/integrations/whatsapp/`;
- tests proving fixture ingestion can produce raw signal events, accepted signal events, canonical `communication.message.recorded` traces and downstream candidate extraction.

The repository does **not** yet contain live WhatsApp sync, live QR/pair-code login, native protocol runtime, WebView runtime, live provider-write execution, live media transfer, live reactions, live statuses, live group/community lifecycle, live identity trace handoff, live provider event bridge from a real runtime into accepted WhatsApp observation events, or business cloud API support.

## Current architecture posture

Current target posture is still correct:

```text
WhatsApp provider/runtime
  -> Integration boundary
  -> Provider observation / signal
  -> Communications projection
  -> Timeline / Search / Radar / Review / shared engines
  -> target domains through workflows only
```

WhatsApp must not become:

- a backend domain;
- a frontend product domain;
- a Persona implementation;
- a Task implementation;
- a Knowledge implementation;
- an AI source of truth;
- a hidden automation/scraping daemon.

## Existing repository assets

### Documentation

| File | Current role |
|---|---|
| `docs/integrations/whatsapp/README.md` | Channel framing and principles. |
| `docs/integrations/whatsapp/architecture.md` | Target architecture and trace flow. |
| `docs/integrations/whatsapp/api.md` | Runtime/setup API foundation. |
| `docs/integrations/whatsapp/modules.md` | Target module inventory. |
| `docs/integrations/whatsapp/status.md` | Earlier zero-based production audit. |
| `docs/integrations/whatsapp/gap-analysis.md` | Missing capability matrix. |
| `docs/integrations/whatsapp/blockers.md` | Architectural blockers. |
| `docs/adr/ADR-0051-v5-whatsapp-web-companion-boundary.md` | WhatsApp Web companion boundary. |
| `docs/adr/ADR-0097-communications-channel-domains-to-integrations.md` | Channels are integrations, Communications owns domain state. |

### Backend source

| Path | Current role |
|---|---|
| `backend/src/integrations/whatsapp/mod.rs` | WhatsApp integration module root. |
| `backend/src/integrations/whatsapp/client.rs` | Public re-exports for the current client/store foundation. |
| `backend/src/integrations/whatsapp/client/models.rs` | Fixture account, session and message DTOs. |
| `backend/src/integrations/whatsapp/client/store.rs` | Store constructor and dependencies. |
| `backend/src/integrations/whatsapp/client/store/accounts.rs` | Fixture account setup. |
| `backend/src/integrations/whatsapp/client/store/sessions.rs` | Session upsert/list and session observations. |
| `backend/src/integrations/whatsapp/client/store/ingestion.rs` | Fixture message ingestion and projection path. |
| `backend/src/integrations/whatsapp/client/store/queries.rs` | Recent projected WhatsApp message query. |
| `backend/src/app/provider_runtime_handlers/whatsapp.rs` | Runtime/setup handlers. |
| `backend/src/app/api_support/messaging_integrations.rs` | WhatsApp capability response and DTO wrappers. |
| `backend/src/app/router/routes/messaging.rs` | Integration routes for WhatsApp fixture/setup/session operations. |

### Frontend source

| Path | Current role |
|---|---|
| `frontend/src/integrations/whatsapp/api/whatsapp.ts` | Typed runtime/setup client. |
| `frontend/src/integrations/whatsapp/queries/useWhatsappQuery.ts` | Runtime/session query hooks. |
| `frontend/src/integrations/whatsapp/stores/whatsapp.ts` | Integration runtime UI state. |
| `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue` | Fixture/manual runtime panel. |
| `frontend/src/integrations/whatsapp/components/*` | Runtime rail, session list and status messages. |
| `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue` | Communications-facing provider surface with projected conversation/message/member/media views plus provider-neutral send/reply/forward/edit/delete, conversation read/unread, pin/unpin, archive/unarchive, mute/unmute and reaction actions. |
| `frontend/src/domains/communications/api/whatsappBusinessApi.ts` | Provider-neutral WhatsApp Communications helper for projected reads and message command routes. |

### Migrations

| Migration | Current role |
|---|---|
| `0021_create_v5_whatsapp_web_foundation.sql` | Adds `whatsapp_web`, `whatsapp_web_session_key`, `whatsapp_web_sessions`. |
| `0125_add_whatsapp_session_observation_kind.sql` | Adds `WHATSAPP_WEB_SESSION` observation kind. |
| `0149_create_canonical_communication_tables.sql` | Adds canonical Communication table family. |
| `0157_create_whatsapp_provider_write_commands.sql` | Adds WhatsApp provider-write command compatibility outbox and mirrors rows into canonical `communication_provider_commands`. |

## Existing API surface

Implemented foundation routes:

| Method | Path | Status |
|---|---|---|
| `GET` | `/api/v1/integrations/whatsapp/capabilities` | Implemented foundation. |
| `GET` | `/api/v1/integrations/whatsapp/accounts` | Implemented WhatsApp account list surface with optional removed-account inclusion. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/accounts` | Implemented fixture-only setup. |
| `GET` | `/api/v1/integrations/whatsapp/sessions` | Implemented session list. |
| `GET` | `/api/v1/communications/messages?channel_kind=whatsapp` | Implemented provider-neutral WhatsApp message list; accountless reads now aggregate across provider shapes through Communications projections. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/messages` | Implemented fixture-only message ingest. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/message-updates` | Implemented fixture-only message update ingest and canonical version projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/message-deletes` | Implemented fixture-only message delete ingest and canonical tombstone projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/receipts` | Implemented fixture-only receipt ingest and canonical delivery-state projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/dialogs` | Implemented fixture-only dialog ingest and canonical conversation projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/participants` | Implemented fixture-only participant ingest and canonical identity/participant projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/reactions` | Implemented fixture-only reaction ingest and canonical projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/media` | Implemented fixture-only media metadata ingest and attachment projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/statuses` | Implemented fixture-only status evidence ingest and Communication projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/status-views` | Implemented fixture-only status-view evidence ingest and status metadata projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/status-deletes` | Implemented fixture-only status-delete evidence ingest, status tombstone projection and realtime event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/presence` | Implemented fixture-only presence evidence ingest, identity-metadata patching and realtime event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/calls` | Implemented fixture-only call-metadata evidence ingest, calls read-model upsert and realtime event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/runtime-events` | Implemented fixture-only runtime-event evidence ingest, accepted Signal Hub event emission and sanitized realtime runtime-event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/sessions/authorized` | Implemented fixture auth-complete vault persistence. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/status` | Implemented blocked-safe runtime status. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/start` | Implemented blocked-safe runtime start. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/stop` | Implemented blocked-safe runtime stop. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/revoke` | Implemented blocked-safe session revoke surface. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/relink` | Implemented blocked-safe relink-preparation surface. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/remove` | Implemented blocked-safe remove surface with session secret cleanup. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/health` | Implemented blocked-safe runtime health. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/chats` | Implemented fixture-backed provider sync surface for projected WhatsApp conversations plus sanitized sync lifecycle events. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/history` | Implemented fixture-backed provider sync surface for projected WhatsApp conversation history plus sanitized sync lifecycle events. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/media` | Implemented fixture-backed provider sync surface for projected WhatsApp attachments plus sanitized sync lifecycle events. |
| `POST` | `/api/v1/integrations/whatsapp/login/qr/start` | Implemented blocked-safe QR login surface. |
| `POST` | `/api/v1/integrations/whatsapp/login/pair-code/start` | Implemented blocked-safe pair-code login surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/send` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reply` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/forward` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/edit` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/delete` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Implemented blocked-safe add-reaction provider command surface. |
| `DELETE` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Implemented blocked-safe remove-reaction provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/voice-note` | Implemented blocked-safe voice-note provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/statuses/publish` | Implemented blocked-safe status publish provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/join` | Implemented blocked-safe group join provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/read` | Implemented blocked-safe conversation mark-read command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unread` | Implemented blocked-safe conversation mark-unread command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/archive` | Implemented blocked-safe conversation archive command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unarchive` | Implemented blocked-safe conversation unarchive command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/mute` | Implemented blocked-safe conversation mute command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unmute` | Implemented blocked-safe conversation unmute command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/pin` | Implemented blocked-safe conversation pin command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unpin` | Implemented blocked-safe conversation unpin command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/leave` | Implemented blocked-safe group leave command surface. |
| `GET` | `/api/v1/integrations/whatsapp/commands` | Implemented command list/filter surface for integration runtime state. |
| `POST` | `/api/v1/integrations/whatsapp/commands/{command_id}/retry` | Implemented manual retry transition and sanitized command-status event. |
| `POST` | `/api/v1/integrations/whatsapp/commands/{command_id}/dead-letter` | Implemented manual dead-letter transition and sanitized command-status event. |
| `GET` | `/api/v1/communications/conversations` | Provider-neutral conversation list now includes canonical WhatsApp conversations. |
| `GET` | `/api/v1/communications/conversations/{conversation_id}` | Provider-neutral conversation detail now resolves canonical WhatsApp conversations. |
| `GET` | `/api/v1/communications/conversations/{conversation_id}/members` | Provider-neutral participant list now resolves canonical WhatsApp conversation members. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/read` | Provider-neutral WhatsApp conversation mark-read now dispatches into `WhatsAppProviderRuntime`. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/unread` | Provider-neutral WhatsApp conversation mark-unread now dispatches into `WhatsAppProviderRuntime`. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/pin` | Provider-neutral WhatsApp conversation pin now dispatches into `WhatsAppProviderRuntime`. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/unpin` | Provider-neutral WhatsApp conversation unpin now dispatches into `WhatsAppProviderRuntime`. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/archive` | Provider-neutral WhatsApp conversation archive now dispatches into `WhatsAppProviderRuntime`. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/unarchive` | Provider-neutral WhatsApp conversation unarchive now dispatches into `WhatsAppProviderRuntime`. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/mute` | Provider-neutral WhatsApp conversation mute now dispatches into `WhatsAppProviderRuntime`. |
| `POST` | `/api/v1/communications/conversations/{conversation_id}/unmute` | Provider-neutral WhatsApp conversation unmute now dispatches into `WhatsAppProviderRuntime`. |
| `GET` | `/api/v1/communications/conversations/search` | Provider-neutral conversation search now includes canonical WhatsApp conversations. |
| `GET` | `/api/v1/communications/messages?channel_kind=whatsapp_web` | Provider-neutral read path through Communications. |
| `GET` | `/api/v1/communications/search/messages` | Provider-neutral projection-backed message search now includes WhatsApp rows. |
| `GET` | `/api/v1/communications/search/media` | Provider-neutral projection-backed media search now includes canonical WhatsApp attachments. |
| `GET` | `/api/v1/communications/conversations/{conversation_id}/pinned-messages` | Provider-neutral pinned-message read now falls back to canonical WhatsApp message projection. |
| `GET` | `/api/v1/communications/messages/{message_id}/raw-evidence` | Provider-neutral evidence read now resolves canonical WhatsApp raw records with redaction. |
| `GET` | `/api/v1/communications/messages/{message_id}/versions` | Provider-neutral lifecycle read now falls back to canonical Communication versions for WhatsApp. |
| `GET` | `/api/v1/communications/messages/{message_id}/tombstones` | Provider-neutral lifecycle read now falls back to canonical Communication tombstones for WhatsApp. |
| `GET` | `/api/v1/communications/messages/{message_id}/reactions` | Provider-neutral reaction read now falls back to canonical WhatsApp reaction projection. |
| `GET` | `/api/v1/communications/messages/{message_id}/reply-chain` | Provider-neutral reply-chain read now falls back to canonical WhatsApp message refs. |
| `GET` | `/api/v1/communications/messages/{message_id}/forward-chain` | Provider-neutral forward-chain read now falls back to canonical WhatsApp message refs. |

Not implemented yet:

- WhatsApp live provider-command execution;
- WhatsApp live command consumer/backoff loop for full provider coverage;
- WhatsApp live provider-observed command reconciliation/completion worker;
- WhatsApp live media transfer execution/progress/completion;
- WhatsApp live status feed, status media and provider-observed publish completion;
- WhatsApp live group/community sync, roster reconciliation and provider-management completion;

## Current capability status

| Capability area | Current status | Notes |
|---|---|---|
| Channel framing | Implemented in docs | Correctly treats WhatsApp as integration/channel. |
| Provider kind `whatsapp_web` | Implemented | Present in migration and provider kind model/tests. |
| Secret purpose `whatsapp_web_session_key` | Implemented foundation | Validated in tests; authorized session material is stored in host vault, bound to the account and resolved by runtime status/start/health. |
| Fixture account setup | Implemented | Local/dev foundation only. |
| Session metadata table | Implemented | `whatsapp_web_sessions`; no live WebView/native session yet. |
| Capability matrix route | Implemented broader foundation | `/api/v1/integrations/whatsapp/capabilities` now exposes an operation-level, provider-shape-aware capability contract with planned/unsupported features, while `/api/v1/integrations/whatsapp/accounts/{account_id}/capabilities` overlays account-scoped lifecycle/runtime state so blocked-vs-degraded WhatsApp operations are visible before UI/provider writes. |
| Fixture message ingest | Implemented foundation | Creates raw evidence/projection path. |
| Provider-neutral message read | Implemented foundation | Via `/api/v1/communications/messages`. |
| Signal Hub connection sync | Implemented foundation | Fixture account setup syncs provider account signal connection. |
| Trace causation/correlation for fixture message | Implemented in tests | Observation -> raw signal -> accepted signal -> communication recorded. |
| Decision/obligation/task candidate extraction from fixture message | Implemented in tests | Downstream engines/workflows already receive communication evidence. |
| Knowledge candidate extraction from fixture message/status | Implemented in tests | Shared Review workflow now mirrors knowledge candidates from WhatsApp projected message/status evidence using the same `message_summary_contract` path as other Communications evidence. |
| Persona/organization candidate extraction from fixture message/status | Implemented in tests | Shared Review workflow now mirrors `new_person` and `new_organization` review items from projected WhatsApp message/status evidence through the same `message_summary_contract` path used by other Communications evidence. This remains review-driven candidate creation from canonical communication evidence, not direct WhatsApp-to-Personas mutation. |
| Frontend runtime panel | Implemented broader foundation | Runtime workspace now shows account-scoped capability/status/health data, provider-shape-aware blocked live account provisioning, blocked-safe lifecycle controls (`start`, `stop`, `revoke`, `relink`, `rotate`, `remove`), QR/pair-code linking responses, an account-scoped provider-command audit panel with retry/dead-letter actions, owner-visible nested runtime health diagnostics from backend `checks` (`session`, `storage`, `runtime`, `webview`, `validation` when present), and projected sync snapshot panels for chats, selected chat history, selected chat members, statuses, contacts, plus selected-chat-scoped presence/call/media snapshots on top of the existing fixture/manual ingest workspace. Integration-owned realtime bootstrap now also patches/invalidate WhatsApp runtime-panel caches for session/runtime lifecycle, provider-command status, direct chat/member/status/presence/call/contact snapshot updates, and projected sync snapshot refresh including `sync-chats`, `sync-history`, `sync-members`, `sync-statuses`, `sync-presence`, `sync-calls`, `sync-contacts` and `sync-media` instead of relying only on manual refresh. Live runtime execution is still blocked. |
| Provider sync control surface | Implemented broader fixture + runtime-bridge foundation | `/api/v1/integrations/whatsapp/provider-sync/chats`, `/provider-sync/history`, `/provider-sync/conversations/{provider_chat_id}/members`, `/provider-sync/statuses`, `/provider-sync/presence`, `/provider-sync/calls`, `/provider-sync/contacts` and `/provider-sync/media` now return projected fixture-backed conversation/history/roster/status-feed/presence/call/contact/media snapshot views, emit sanitized `whatsapp.sync.*` lifecycle events, and also materialize canonical `signal.accepted.whatsapp.runtime_event` evidence for sync `started`/`progress`/`completed` phases. Dedicated `/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle` now lets an external runtime process publish `chats` / `history` / `members` / `statuses` / `presence` / `calls` / `contacts` / `media` sync phases into the same event spine without fixture-only endpoints. Live runtime paging/reconciliation and a real producer are still missing. |
| Live runtime | Missing | Runtime lifecycle API exists, but real WebView/native execution is still blocked. |
| QR/pair-code login | Implemented blocked-safe pairing-state foundation | Login surfaces still do not produce live QR/pair-code material, but they now move account/session lifecycle through `qr_pending` and `pair_code_pending`, persist those states in `whatsapp_web_sessions`, and emit sanitized `whatsapp.runtime.status_changed` plus `whatsapp.session.link_state_changed` events through the event spine. |
| Authorized session vault persistence | Implemented broader foundation | Fixture auth-complete path stores account-scoped session material in host vault, binds `whatsapp_web_session_key` and verifies restore on runtime status/start/health; `runtime/revoke`, `runtime/relink` and `runtime/remove` now also remove the restorable session binding, `secret_references` metadata and host-vault payload so startup cannot silently restore a revoked or relink-required account. Authorized-session completion emits sanitized `whatsapp.runtime.status_changed` and `whatsapp.session.link_state_changed` events; repeat authorization of the same account now emits `session_rotated` instead of looking like a second primary authorization, while preserving the same account-scoped secret binding and overwriting the vault payload. Live provider must call the same service after successful pairing. |
| Relink / remove lifecycle | Implemented broader fixture/runtime foundation | `runtime/relink` resets account lifecycle to `link_required` for a fresh pairing flow and clears restorable session material, while `runtime/remove` clears account-scoped secret bindings before marking the provider account removed and dropping the Signal Hub connection. `runtime/start|stop|revoke|relink|remove` append sanitized runtime/session lifecycle events into the event spine. Historical communications evidence remains intact. |
| Message send/reply/forward/delete/edit | Implemented broader blocked + fixture executor foundation | Provider command endpoints validate through `WhatsAppProviderRuntime`, require idempotency keys, persist durable rows in `whatsapp_provider_write_commands`, mirror to canonical `communication_provider_commands`, emit sanitized `whatsapp.command.status_changed` events and return blocked for live runtime use; the background fixture executor now claims retried `send_text`, `reply`, `forward`, `edit` and `delete` rows and drives provider-observed completion through the event spine, including canonical reply/forward refs for the message-producing paths plus canonical tombstones for deletes. Accepted WhatsApp message and lifecycle observation events now also have a dedicated background reconciliation consumer that reconstructs typed provider DTOs from raw evidence and drives the same durable command completion path through `WhatsAppProviderRuntime`, so future live runtimes can reuse the same event-spine contract instead of direct fixture-only hooks. Provider-neutral Communications handlers for conversation send plus message reply/forward/edit/delete now also dispatch WhatsApp accounts into the same runtime command path instead of falling back to telegram-only handler assumptions. Exact fixture validation already proves direct `send_text` reconciliation and `reply`/`forward` reconciliation inside the full fixture workflow; an added provider-neutral route regression test currently compiles but local execution can still be blocked by host Docker network exhaustion. Live execution and the real runtime event bridge are still incomplete. |
| Rich message semantics | Implemented broader fixture metadata foundation | Fixture message evidence can now carry structured provider metadata for mentions, link previews, polls, location, contact cards, stickers, join/leave service facts, system/service updates, ephemeral/disappearing flags and view-once flags through the raw/accepted Signal Hub path into canonical `communication_messages.message_metadata`. The provider-neutral WhatsApp Communications surface now also renders those projected metadata families directly in the active timeline as chips/cards/list items instead of collapsing everything into plain text only. Live runtime extraction and richer workflow consumers are still missing. |
| Reaction add/remove command flow | Implemented broader fixture reconciliation + executor foundation | Add/remove reaction provider command endpoints use the same durable outbox and sanitized command-status events as other WhatsApp writes; fixture provider-observed reaction evidence now reconciles `react`/`unreact` commands into durable `completed`/`failed` command state and emits `whatsapp.command.status_changed` plus `whatsapp.command.reconciled`, and the background fixture executor can complete retried `react` and `unreact` rows through provider-observed evidence. Provider-neutral `/api/v1/communications/messages/{message_id}/reactions` now also dispatches WhatsApp accounts into the same runtime command path instead of remaining telegram-only, and the WhatsApp Communications panel can add/remove reactions over those routes while rendering projected reaction summaries from canonical message metadata. Live execution/runtime bridging is still missing. |
| Media upload/download command flow | Implemented broader blocked-safe + runtime-bridge foundation | `/api/v1/integrations/whatsapp/provider-media/upload` validates local attachment/blob metadata, `/provider-media/download` queues durable read-class command rows, both go through `WhatsAppProviderRuntime`, mirror into canonical `communication_provider_commands`, emit sanitized `whatsapp.command.status_changed` plus media lifecycle events, and now also materialize canonical `signal.accepted.whatsapp.runtime_event` evidence for blocked-safe `media.upload.requested|failed`, `media.download.requested|failed`, and fixture-executor `media.upload.started|progress|completed` / `media.download.started|progress|completed` phases. The background fixture executor can complete retried `send_media` and `download_media` rows through provider-observed fixture media evidence while preserving canonical attachment/blob contracts (`local_fs`, validated `sha256`). Exact fixture validation now also proves direct provider-observed reconciliation plus canonical runtime-event evidence for `send_media`, `download_media` and `send_voice_note` command rows. Dedicated `/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle` now lets an external runtime process publish live upload/download lifecycle phases into the same sanitized `whatsapp.media.*` and accepted runtime-event path without using fixture-only endpoints. Real live producer execution, byte transfer and full reconciliation are still missing. |
| Provider command list/retry/dead-letter | Implemented broader fixture executor + runtime-bridge worker foundation | Integration-control endpoints expose command status without command payload text and update both runtime compatibility rows and canonical provider command rows. Manual retry now feeds a background WhatsApp fixture command executor that can claim due outbox rows and complete supported fixture commands through provider-observed evidence, including text/reply/forward/edit/delete, reactions, media upload/download, dialog state, join/leave membership and status publish flows. The same executor now imports canonical queued WhatsApp commands from `communication_provider_commands` into `whatsapp_provider_write_commands` so fixture/runtime-safe event-driven command execution exists end to end. Dedicated `/api/v1/integrations/whatsapp/runtime-bridge/commands/claim` and `/runtime-bridge/commands/{command_id}/failed` now provide a worker-facing live execution surface for non-fixture accounts, reusing the same durable `executing -> retrying|dead_letter` lifecycle instead of inventing a second outbox model. Claimed command payloads now also carry explicit provider/runtime metadata (`provider_kind`, `provider_shape`, `runtime_kind`, `lifecycle_state`, `session_restore_available`, `runtime_blockers`) plus durable execution state (`capability_state`, `action_class`, `confirmation_decision`, `provider_state`, `result_payload`) so a replaceable external worker does not need hidden account-side lookups before dispatch. The failed-command surface now also accepts optional structured `error_code` and `retry_after_seconds`, persisting them into durable command failure metadata while still driving the same retry/dead-letter lifecycle. Shared retry scheduling now uses a capped exponential backoff by durable retry count, stale interrupted executions persist the same structured failure metadata (`error_code = interrupted_execution`, `reported_via = stale_execution_recovery`) instead of silently dropping execution context, and stale recovery is now runtime-scoped so fixture executor and live runtime workers cannot steal each other's `executing` rows. Runtime-bridge observed evidence still preserves live/runtime-bridge provenance on command reconciliation instead of falsely collapsing back to fixture-only sources. |
| Dialogs/conversations | Implemented fixture projection + broader reconciliation foundation | Fixture dialog evidence is source-backed and projected into canonical `communication_channels` and `communication_conversations`; provider-neutral conversation list/detail/search now read those rows, provider-neutral conversation `read`/`unread`, `pin`/`unpin`, `archive`/`unarchive` and `mute`/`unmute` routes now dispatch WhatsApp accounts into the same runtime command path, fixture dialog metadata now carries source-backed unread count, participant count, avatar/profile-picture metadata and provider labels through canonical conversation metadata, and fixture provider-observed dialog evidence reconciles durable `archive`/`unarchive`, `pin`/`unpin`, `mute`/`unmute` and `mark_read`/`mark_unread` command rows with `whatsapp.command.reconciled` events. Exact validation now also proves the live `/runtime-bridge/dialogs` path preserves `provider_observed.runtime_bridge_dialog` provenance both in reconciliation/runtime events and in raw evidence. Live sync and provider management are still missing. |
| Participants/membership | Implemented broader fixture lifecycle foundation | Fixture participant evidence is source-backed and projected into `communication_identities` and `communication_conversation_participants`; participant ids are now stable per conversation/member, repeated participant observations can record previous/current role and membership status facts in canonical participant metadata, and sanitized `whatsapp.participant.changed` events now surface role/status lifecycle changes. Exact validation now also proves the live `/runtime-bridge/participants` path preserves `provider_observed.runtime_bridge_participant` provenance both in reconciliation/runtime events and in raw evidence. Provider-neutral members read resolves the current canonical row, and `/api/v1/integrations/whatsapp/provider-sync/conversations/{provider_chat_id}/members` now exposes an integration-side roster sync/read surface over the same canonical participant projection with sanitized `whatsapp.sync.members.*` lifecycle evidence. Real live roster producer coverage and broader membership lifecycle are still missing. |
| Message lifecycle versions/deletes | Implemented fixture projection + executor-backed reconciliation foundation | Fixture message update/delete evidence is source-backed and projected into canonical `communication_message_versions` and `communication_message_tombstones`; fixture provider-observed update/delete evidence now reconciles durable `edit` and `delete` command rows and emits `whatsapp.command.reconciled`, and the background fixture command executor can complete retried `edit` and `delete` commands end-to-end. Accepted WhatsApp message-update and message-delete observation events now also feed the same dedicated reconciliation consumer, so edit/delete completion no longer depends only on direct fixture ingest paths. Reply/forward lineage is also verified on retried fixture `reply` and `forward` commands through canonical `communication_message_refs`. Exact validation now also proves the live `/runtime-bridge/messages`, `/message-updates`, `/message-deletes`, `/reactions`, `/media`, `/dialogs`, `/participants` and `/statuses` paths stamp raw evidence provenance with their respective `provider_observed.runtime_bridge_*` sources instead of silently falling back to fixture-only raw metadata. Live provider event bridge/runtime wiring is still missing. |
| Delivery receipts | Implemented broader projection + reconciliation foundation | Fixture receipt evidence is source-backed and projected into canonical `communication_messages.delivery_state`, and the live `/runtime-bridge/receipts` path now also stamps raw evidence provenance with `provider_observed.runtime_bridge_receipt` instead of collapsing that ingest back into fixture-only semantics. Accepted WhatsApp receipt events now also participate in the provider-observation reconciliation consumer, so already-correlated live/provider-observed message commands can advance durable `result_payload` / `provider_state.delivery_state` from later receipt evidence without direct handler coupling. Full live receipt producer coverage is still missing. |
| Reactions | Implemented fixture projection foundation | Fixture reaction evidence is source-backed and projected into `communication_message_reactions`; provider-neutral reaction reads now fall back to canonical rows, while live reaction commands and reconciliation are still missing. |
| Presence | Implemented broader fixture sync foundation | Fixture presence evidence is source-backed through `WhatsAppProviderRuntime`, now emits accepted Signal Hub presence events and can patch canonical `communication_identities.metadata` with observed presence/last-seen facts for known identity traces. `/api/v1/integrations/whatsapp/provider-sync/presence` now exposes an integration-side presence snapshot over those canonical identity rows, optionally scoped by `provider_chat_id`, and emits the same sanitized `whatsapp.sync.*` plus accepted runtime-event evidence as the other sync control surfaces. The live `/runtime-bridge/presence` path now also stamps raw evidence provenance with `provider_observed.runtime_bridge_presence`, and `/runtime-bridge/sync-lifecycle` accepts `scope = presence` for external runtime sync phases. Live runtime presence feed and broader UI/runtime bridging are still missing. |
| Calls | Implemented broader fixture sync foundation | Fixture call evidence is source-backed through `WhatsAppProviderRuntime`, now emits accepted Signal Hub call-metadata events, upserts the existing calls read model with observed WhatsApp call facts and appends sanitized `whatsapp.call.updated` events. Those projection events now carry the generic Timeline replay contract fields (`subject.entity_id`, `source.kind`, `source.source_id`) so call evidence can flow into Timeline through the shared event-log replay path. `/api/v1/integrations/whatsapp/provider-sync/calls` now exposes an integration-side call snapshot over that shared read model, optionally scoped by `provider_chat_id`, and emits the same sanitized `whatsapp.sync.*` plus accepted runtime-event evidence as the other sync control surfaces. The live `/runtime-bridge/calls` path now also stamps raw evidence provenance with `provider_observed.runtime_bridge_call`, and `/runtime-bridge/sync-lifecycle` accepts `scope = calls` for external runtime sync phases. Live runtime call feed and broader relationship workflows are still missing. |
| Runtime events | Implemented broader fixture + runtime-bridge foundation | Fixture runtime-event records now enter through `WhatsAppProviderRuntime`, are stored as raw Communication evidence with accepted Signal Hub `signal.accepted.whatsapp.runtime_event`, and append sanitized `whatsapp.runtime.event` entries into the event spine. Blocked-safe runtime lifecycle transitions such as `session_authorized`, `runtime_start`, `runtime_stop`, `runtime_revoke`, `runtime_relink`, `runtime_remove`, `login_qr_start` and `login_pair_code_start` now also materialize canonical runtime-event evidence through the same raw/accepted Signal Hub path instead of existing only as ad-hoc realtime payloads. Sync control flows now do the same for `sync.chats.*` and `sync.history.*` phases. Unknown/unsupported runtime/provider events now default to `runtime_status=degraded`, `lifecycle_state=degraded` and `severity=warning` even when the caller omits those markers, so unsupported provider evidence is preserved and explicitly degraded instead of being silently dropped. Accepted WhatsApp runtime-event lifecycle signals now also have a dedicated background reconciliation consumer that derives account/session lifecycle state plus Signal Hub connection status from the event spine and clears restorable session bindings for `link_required`, `revoked` and `removed` transitions, so future live runtimes can drive the same recovery path without depending on direct HTTP handlers. The sanitized runtime event emits only account id, provider event id, event kind, lifecycle/runtime status, severity and top-level metadata keys, while secret-like runtime metadata is redacted before persistence. Dedicated `/api/v1/integrations/whatsapp/runtime-bridge/*` routes now let an external runtime process reuse the same typed observation and session-authorized ingest path without going through fixture-only endpoints. A real producer/runtime implementation is still missing. |
| Realtime projection events | Implemented broader fixture foundation | Fixture ingest now appends sanitized `whatsapp.dialog.updated`, `whatsapp.message.created`, `whatsapp.message.updated`, `whatsapp.message.deleted`, `whatsapp.reaction.changed`, `whatsapp.receipt.changed`, `whatsapp.presence.changed`, `whatsapp.call.updated`, `whatsapp.status.updated`, `whatsapp.status.deleted` and `whatsapp.runtime.event` events into the event spine without leaking message body text or raw provider payloads. Projection and command/media events now include `subject.entity_id` plus `source.kind`/`source.source_id`, and the exact WhatsApp fixture test verifies that status/participant/call events replay through the generic `TimelineEngine::replay_event_log` path. Live runtime event bridging from a real provider is still missing. |
| Media | Implemented broader fixture sync + executor foundation | Fixture media metadata is projected into `communication_attachments` with `not_scanned`, `/api/v1/integrations/whatsapp/provider-sync/media` now exposes an integration-side attachment snapshot over those canonical rows with optional `provider_chat_id` and `content_type` filters, and retried fixture media upload/download commands now reconcile against provider-observed media events to complete durable outbox rows through the event spine. `/runtime-bridge/sync-lifecycle` also accepts `scope = media` for external runtime sync phases. Live transfer bytes, progress reporting and scanner backend are still missing. |
| Voice notes | Implemented blocked command + broader local preview foundation | Voice-note send now has a durable blocked-safe provider command surface over local attachment/blob metadata, shares the same fixture media execution/reconciliation path as `send_media`, is covered by a dedicated fixture end-to-end retry/executor test, and now also has direct provider-observed reconciliation proof through fixture media evidence. The shared attachment preview boundary and WhatsApp media preview surface now also support local audio playback from safe local blobs, which gives projected voice-note playback a real UI path when the voice note is present as a canonical attachment. Live upload/execution, waveform metadata and transcript handoff are still missing. |
| Media viewer | Implemented broader safe local preview foundation | The WhatsApp Communications surface now offers an in-panel media viewer for projected safe local image/text/audio/video/pdf attachments via the existing provider-neutral attachment preview route instead of forcing message-only navigation. Shared attachment preview surfaces in Communications now also render safe local PDF previews from canonical blobs, so WhatsApp document attachments can be reviewed in-panel without leaving the provider-neutral flow. Richer binary viewer support still depends on broader blob/viewer support. |
| Status | Implemented broader fixture lifecycle foundation | Fixture status evidence is projected as provider-neutral Communication metadata; fixture status records can now also materialize source-backed author identity evidence (`sender_identity_kind`, address, push name, business profile, profile-photo ref) into canonical `communication_identities` plus status message metadata, trigger shared decision/task candidate refresh through the existing Communications review workflow, project a synthetic canonical `status-feed` conversation surface into `communication_conversations` so provider-neutral conversation list/detail/search can surface WhatsApp statuses without creating a separate domain, patch canonical status metadata from fixture status-view evidence (`status_viewed`, viewers, count, timestamps), record canonical tombstones plus `status_deleted` metadata from status-delete evidence, attach fixture status media through the canonical attachment path, and let fixture messages reply to projected status messages through canonical reply refs. The WhatsApp Communications timeline now also renders status-specific lifecycle details (author identity, view counts, last viewer, delete marker) plus linked status media cards from the same projected `status-feed` conversation instead of collapsing statuses to plain text rows. Blocked-safe and retried `publish_status` commands now also materialize canonical `signal.accepted.whatsapp.runtime_event` evidence for `status.publish.requested|failed|started|completed`, while both fixture status evidence and `/runtime-bridge/statuses` can reconcile durable `publish_status` command rows plus sanitized `whatsapp.command.reconciled` events through the same status-feed projection path, with live runtime-bridge provenance preserved as `provider_observed.runtime_bridge_status` instead of pretending to be fixture evidence. `/api/v1/integrations/whatsapp/provider-sync/statuses` now exposes an integration-side status-feed sync/read surface over the projected `status-feed` conversation, and `/runtime-bridge/sync-lifecycle` now accepts `scope = statuses` so external runtimes can publish status-feed sync phases into the same event spine. The live `/runtime-bridge/status-views` and `/runtime-bridge/status-deletes` paths now also stamp raw evidence provenance with `provider_observed.runtime_bridge_status_view` and `provider_observed.runtime_bridge_status_delete`. Exact validation now proves both fixture and runtime-bridge observed reconciliation plus canonical runtime-event evidence for `publish_status`, and raw-provenance preservation for live status-view/delete ingress. Real live producer coverage remains missing. |
| Groups/communities/broadcasts | Implemented broader fixture evidence + blocked command foundation | Fixture dialog metadata can now carry source-backed community/subgroup linkage (`community_parent_chat_id`, `community_parent_title`), invite-link evidence and broadcast/newsletter/community-root flags through canonical conversation metadata plus sanitized `whatsapp.dialog.updated` events, and blocked-safe join/leave/archive/mute/pin/read command surfaces already exist. Fixture provider-observed dialog and participant state completes archive/mute/pin/read plus join/leave outbox rows, while live group/community sync, roster reconciliation and provider-observed completion are still missing. |
| Phone identity traces | Implemented broader fixture sync foundation | Fixture participant projection now stores canonical communication identities and participant metadata for phone/wa_id/display-name evidence plus source-backed `push_name`, `business_profile` and `profile_photo_ref` traces. Fixture participant and status ingest now also materialize source-backed unattached `person_identities` traces for `whatsapp` and `phone` via the existing compatibility identity-trace boundary, and those rows now carry source-backed WhatsApp evidence metadata rather than only the raw identity value. Participant ingest also records `message_participant` traces for group/community member evidence with role/status/profile metadata, and message ingest records phone traces from contact-card evidence with the observed contact-card payload. Canonical `communication_identities.metadata` now also preserves `display_name_history` across repeated participant/presence evidence so provider-visible naming drift remains source-backed instead of silent overwrite. `/api/v1/integrations/whatsapp/provider-sync/contacts` now exposes an integration-side contact snapshot over those canonical identity rows plus compatibility WhatsApp/phone trace metadata, and `/runtime-bridge/sync-lifecycle` accepts `scope = contacts` for external runtime sync phases. Persona assignment remains review-driven instead of automatic merge. |
| Search/media gallery | Implemented broader foundation | Provider-neutral `/api/v1/communications/search/messages`, `/api/v1/communications/search/media` and pinned-message reads now return WhatsApp projection data from canonical messages and attachments, while `WhatsAppCommunicationsPanel.vue` also exposes projected conversation/message/member/media views plus provider-neutral send/reply/forward/edit/delete, conversation read/unread, pin/unpin, archive/unarchive, mute/unmute and reaction add/remove flows over the same Communication routes. Frontend realtime cache patching now also applies `whatsapp.dialog.updated` conversation lifecycle metadata (`is_unread`, `is_pinned`, `is_archived`, `is_muted`, counts) directly into cached WhatsApp conversation list/detail queries instead of relying only on broad invalidation. The WhatsApp Communications surface now also replaces prompt-based forward targeting and message editing with in-panel control surfaces, lets the owner jump from pinned/media sections back to the source message in the active timeline, and adds a dedicated media browsing mode with kind filtering (`all` / `image` / `video` / `audio` / `document`) on top of the existing provider-neutral media search route. Richer gallery/index UX is narrower now, but still not fully complete. |
| Calls | Unsupported except metadata future | No call capture/recording/live handling without ADR. |
| Business Cloud API | Future provider only | Must not replace personal WhatsApp Web/native provider. |

## Current trace evidence

The fixture message test confirms the target trace shape:

```text
observation.captured.v1
  -> signal.raw.whatsapp.message.observed
  -> signal.accepted.whatsapp.message
  -> communication.message.recorded
```

The same test asserts that legacy `integration.whatsapp.%` events are not emitted for the current Signal Hub trace path. That means the live runtime should publish into the same canonical observation/signal path rather than inventing a second WhatsApp event dialect.

The fast guard layer now also encodes this rule in tests:
`backend/tests/whatsapp_signal_hub.rs` checks the documented WhatsApp Signal Hub
fixture families and sanitized command/reconciliation event contract, while the
WhatsApp slices in `backend/tests/communications_architecture_target.rs` keep
provider libraries and runtime implementation details out of domains, engines
and workflows.

## Architecture debt observed

### 1. Runtime composition still wires Communications ports

Current WhatsApp application services depend on `dyn WhatsAppProviderRuntime`,
and the current `WhatsappWebStore` implements that trait as the fixture/Web
companion foundation. The application composition root still wires the runtime
with Communications port implementations. That wiring is allowed as composition,
but it must not become a precedent for provider libraries or runtime modules
importing Communications stores directly.

Target direction:

```text
Integration runtime captures provider event
  -> platform event / observation contract
  -> Communications consumer/projection
```

The integration must not own Communications tables or bypass Communications event consumers.

### 2. WhatsApp docs are zero-based while code has foundation work

`docs/integrations/whatsapp/status.md` still says production WhatsApp capability is counted as zero. That was true for the earlier audit baseline, but the current code already has fixture/provider foundation. Keep that older doc as historical baseline and use this audit for current state.

### 3. Frontend has mixed naming

Runtime UI correctly lives under `frontend/src/integrations/whatsapp/`.

Any user-facing Communication workspace should stay under:

```text
frontend/src/domains/communications/*
```

Provider-specific Communication cache keys must remain rooted under `['communications', ...]`, while runtime/setup caches may use `['integrations', 'whatsapp', 'runtime', ...]`.

### 4. Full functionality conflicts with ADR-0051 if interpreted as hidden automation

Full WhatsApp functionality requires either:

- a visible WebView companion runtime;
- an unofficial native multi-device protocol runtime;
- a future official business cloud provider.

These are three different provider shapes. They must not be blurred into one `whatsapp` adapter because that would turn the integration into a junk drawer with ambitions.

## Required next documentation decisions

Before implementing live support, add or accept:

1. Provider runtime selection ADR.
2. Full functionality target spec.
3. Provider command/outbox contract for WhatsApp.
4. Raw evidence and canonical projection contracts for every source record kind.
5. Media/blob/scanner contract.
6. Phone identity trace contract.
7. Runtime/session/secret storage contract.
8. Privacy/consent/capability UX contract.
9. Fixture-first test matrix.
10. Live manual smoke-test matrix outside CI.

Current state update:

- WhatsApp-specific raw-evidence redaction is now asserted in backend tests for
  nested secret-like metadata keys (`session_key`, `access_token`,
  `refresh_token`).
- Fixture-first source-record and command-class coverage is now tracked in
  `docs/integrations/whatsapp/fixture-test-matrix.md`.
- Manual live validation checklist now exists in
  `docs/integrations/whatsapp/live-smoke-checklist.md` for ADR-0101 acceptance work.

## Recommended immediate implementation order

```text
P0 Documentation and ADR
P1 Provider runtime boundary and capability model
P2 Native/WebView fixture adapter contract
P3 Inbound messages/dialogs/participants/status/media metadata
P4 Provider command outbox for writes
P5 Hermes intelligence: Radar, Review, Timeline, Search, Context Packs
P6 Business Cloud provider only if explicitly needed
```

## Non-negotiables

- No hidden scraping.
- No provider write without capability, owner-visible intent, audit and reconciliation.
- No session secrets in PostgreSQL, event payloads, logs or frontend state.
- No direct domain mutation from WhatsApp integration.
- No AI-created business entities without evidence, confidence and review/promotion.
- No live account dependency in CI.
- No fake production status. Fixture is fixture. A civilization collapses one euphemism at a time.
