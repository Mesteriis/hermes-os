# Telegram Modules

Статус: module map на 2026-06-17.

Telegram остаётся Communication Channel. Модули ниже не создают отдельный product domain.

## Backend Modules

| Module | Current files | Назначение | Status |
|---|---|---|---|
| `api` | `backend/src/integrations/telegram/api/` | Protected HTTP handlers for capabilities, accounts, QR login, runtime, chats, messages, message reactions, raw evidence and media, with message reaction handlers split under `api/messages/` for SRP | IMPLEMENTED |
| `accounts` | `backend/src/integrations/telegram/client/accounts/` | Fixture/live account setup, host-vault secret binding, lifecycle logout/remove | PARTIAL |
| `capabilities` | `backend/src/domains/api_support/messaging_integrations.rs` | Detailed global and account-scoped capability matrices with operation-level status, reason, action class and selected-account overrides | PARTIAL |
| `dialogs` | `telegram_chats`, `client/chats.rs`, `client/chat_state.rs` | Local chat/dialog projections from fixture or TDLib snapshots, including provider-observed unread/mute/archive/pin chat-state projection, command reconciliation helpers, typed pin/archive/mute flag events, local and provider-observed unread `telegram.chat.updated` events and projection-backed folder filters from `metadata.folder_name` | PARTIAL |
| `messages` | `client/messages/` | Fixture/TDLib ingestion orchestration, raw record projection, sanitized raw evidence read, recent query, manual send response assembly and sanitized realtime message/chat snapshot enrichment | PARTIAL |
| `message_metadata` | `client/messages/message_metadata.rs` | TDLib-derived mention, public-link, album, attachment hint and structured source-evidence metadata before Communication projection | PARTIAL |
| `runtime` | `backend/src/integrations/telegram/runtime/` | Account-scoped fixture/TDLib actor manager with start/status/sync/send/media commands, TDLib/runtime diagnostics and sanitized runtime event bridge handling for typing/topic/unread/mute/archive/pin provider updates, split into focused realtime chat/topic modules plus dedicated chat event payload builders to stay under the architecture line limit | PARTIAL |
| `tdjson` | `backend/src/integrations/telegram/tdjson/`, `backend/src/integrations/telegram/tdjson/tests/` | TDLib native loading, JSON request builders, parsing, QR login worker and split contract-test modules for environment/request/parsing/QR flows | PARTIAL |
| `media` | `runtime/media.rs`, `runtime/manager/media_download.rs` | On-demand TDLib download into local blob/attachment storage plus started/progress/failed/completed realtime emission | PARTIAL |
| `attachments` | `client/messages/attachments.rs`, mail storage compatibility boundary | Anchor Telegram messages to Communication attachments and patch projected attachment download metadata | PARTIAL |
| `calls` | `backend/src/platform/calls/` | Telegram call metadata and fixture transcripts | PARTIAL |
| `automation` | `backend/src/engines/automation*` | Telegram send dry-run policy/template evaluator and audit | PARTIAL |
| `audit` | `backend/src/platform/audit/telegram.rs` | Redacted audit metadata for manual send, automation dry-run and account lifecycle | IMPLEMENTED |

## Missing / Target Backend Modules

| Module | Назначение | Why needed |
|---|---|---|
| `capability_matrix` | Per-operation capability model | Required before exposing provider-write commands |
| `command_outbox` | Durable provider-write command queue | Needed for edit/delete/reaction/pin/send-media/retry/audit |
| `message_versions` | Observed edit versions and diffs | Needed for source-evidence preserving edit history |
| `message_tombstones` | Delete/visibility history | Needed for delete/restore provider parity |
| `reply_projection` | Reply target and reply graph | Needed for reply chains and topic views |
| `forward_projection` | Forward attribution and chains | Needed for forward provenance |
| `reaction_projection` | Reaction sync and commands | Needed for reaction UX/realtime |
| `topic_projection` | Forum topics and topic-scoped timelines | Needed for supergroup topics |
| `media_gallery` | Provider/local media search and gallery | Needed for media-heavy Telegram usage |
| `provider_search` | TDLib/provider search | Dedicated provider search entrypoint, workspace provider/local/fallback source label and shared Communication saved-search persistence with `channel_kind=telegram` exist; provider-side saved-search scheduling still missing |
| `bot_runtime` | Bot API runtime | Needed for `telegram_bot` parity |
| `session_proxy` | Session bundle/proxy profiles | Needed for portable/local-first runtime configuration |
| `realtime_events` | Sanitized `telegram.*` event contracts with projected message/chat/topic snapshots and deterministic chat/topic resolution | Needed for realtime Telegram UX |

## Frontend Modules

| Module | Current files | Назначение | Status |
|---|---|---|---|
| `page` | `frontend/src/domains/telegram/views/TelegramPage.vue`, `frontend/src/domains/telegram/views/dialogActionHelpers.ts` | Desktop three-pane Telegram workbench with shared thread/search navigation helpers and shared realtime recovery status surfaced from the view layer | PARTIAL |
| `api` | `frontend/src/domains/telegram/api/telegram.ts`, `frontend/src/domains/telegram/api/telegramWorkspace.ts`, `frontend/src/domains/telegram/api/telegramAutomation.ts` | Typed backend route calls, separated UI-friendly workspace helpers, projection-backed folder filter fetch and automation dry-run contracts | IMPLEMENTED |
| `queries` | `frontend/src/domains/telegram/queries/useTelegramQuery.ts`, `frontend/src/domains/telegram/queries/useTelegramRuntimeQuery.ts`, `frontend/src/domains/telegram/queries/useTelegramFolderFilters.ts`, `frontend/src/domains/telegram/queries/useTelegramLifecycleQuery.ts`, `frontend/src/domains/telegram/queries/useTelegramQrLoginQuery.ts`, `frontend/src/domains/telegram/queries/useTelegramAutomationQuery.ts` | TanStack Query hooks/mutations for accounts, chats, folder filters, QR login, automation dry-run, lifecycle evidence, messages, calls and isolated runtime status/start/stop/restart controls | PARTIAL |
| `store` | `frontend/src/domains/telegram/stores/telegram.ts` | Pinia UI state, filters, counts and derived message/file/link/pinned/voice views | PARTIAL |
| `dialogs` | `TelegramChatList.vue`, `TelegramActionRail.vue` | Virtualized chat list with local filters, unread/mention metadata badges, runtime start/stop/restart controls and action-rail folder groups from a selected-account backend route with local fallback | PARTIAL |
| `messages` | `TelegramMessageThread.vue`, `thread/TelegramMessageList.vue`, `thread/TelegramMessageReferencePanel.vue`, `thread/TelegramMessageReactions.vue`, `thread/referenceEvidence.ts`, `stores/telegramMentionProjection.ts` | Selected chat timeline, local text search, older-history trigger, per-message mention source-evidence chips, diff-aware lifecycle/reference evidence panel with local evidence filtering, isolated reaction chip/picker rendering, and search-driven message focusing | PARTIAL |
| `composer` | `thread/TelegramComposer.vue`, `thread/TelegramSendDryRunPanel.vue` | Text send UI plus policy-backed dry-run preview; attachment/voice buttons remain disabled | PARTIAL |
| `media` | `thread/TelegramThreadSideSections.vue`, `thread/TelegramMediaViewer.vue`, `TelegramSearchResultsPanel.vue`, `stores/telegramMediaSearch.ts`, `queries/useTelegramAttachmentPreviewQuery.ts` | File/link/voice/pinned/timeline tabs plus dedicated preview/playback surfaces that now merge loaded-message attachment hints with query-backed media-search metadata, projected readiness labels and shared safe attachment previews for files-tab/search parity | PARTIAL |
| `inspector` | `TelegramRail.vue`, `TelegramCallsPanel.vue`, `TelegramCallTranscriptPanel.vue`, `TelegramAccountManager.vue`, `TelegramCommandAuditPanel.vue`, `TelegramQrLoginPanel.vue`, `TelegramCapabilityMatrix.vue` | Inspector for projected context, member summaries plus local projected-member search, first-class selected-account calls panel, durable command audit rows with retry/dead-letter summaries, read-only call transcript evidence, about, account-scoped runtime sync/command/TDLib diagnostics with explicit targets, QR login flow, selected-account capability matrix and local account lifecycle management | PARTIAL |

## Missing / Target Frontend Modules

| Module | Назначение |
|---|---|
| `account_setup` | QR login, account lifecycle, capability status and live runtime controls |
| `chat_inspector` | Members, about, permissions, topic list, linked personas/projects |
| `topic_view` | Forum/topic scoped timeline |
| `reply_thread_view` | Reply-chain navigation |
| `reaction_sync_state` | Provider reconciliation, pending command state and richer reaction diagnostics |
| `message_actions` | Edit/delete/pin/reply/forward/read commands with capability gates and query invalidation |
| `media_viewer` | Photo/video/document/voice preview and download manager |
| `media_gallery` | Media search and grouped media browsing |
| `voice_player` | Dedicated voice/audio playback exists; transcript and summary handoff remain missing |
| `calls_panel` | Call metadata, read-only transcript evidence and future call controls |
| `global_search` | Dialog/message/media search UI |
| `runtime_status` | TDLib/Bot capability diagnostics, degraded state diagnostics and native dependency remediation |
| `realtime_status` | Telegram-specific event stream/live sync state |

## Functional Module Map

| Capability module | Назначение | Current repository status |
|---|---|---|
| `accounts` | Account metadata, lifecycle, secrets | PARTIAL |
| `runtime_fixture` | Deterministic local/test runtime | IMPLEMENTED |
| `runtime_tdlib_user` | TDLib QR-authorized user runtime | PARTIAL |
| `runtime_bot` | Bot API runtime | MISSING |
| `dialogs` | Account-scoped dialog/chat list, projection-backed folder/chat-list filters, queued read/unread/archive/mute provider toggles and provider-observed unread/archive/mute counter-state reconciliation | PARTIAL |
| `private_chats` | 1:1 chat projection | PARTIAL |
| `saved_messages` | TDLib owner private-chat detection via `is_saved_messages` metadata and read-only UI marker; provider reconciliation still missing | PARTIAL |
| `groups` | Basic group projection plus compatibility projection for supergroups | PARTIAL |
| `supergroups` | TDLib supergroup identity metadata and read-only inspector projection; admin lifecycle still missing | PARTIAL |
| `permissions` | TDLib chat permissions metadata and read-only inspector summary; member roles/admin state still missing | PARTIAL |
| `channels` | Channel chat projection | PARTIAL |
| `topics` | Forum topic projection, topic-scoped replies, read-only topic state/provider labels and provider-observed `updateForumTopicInfo` projection refresh via `telegram.topic.updated` | PARTIAL |
| `messages` | Source-backed message projection, public permalink and poll/location/contact-card metadata, and sanitized raw evidence access with thread-level source evidence UI | PARTIAL |
| `message_source_evidence_ui` | Dedicated source-evidence rendering component and pure metadata evidence helper for raw/link/structured/custom reaction evidence, keeping the lifecycle reference panel below SRP limits | PARTIAL |
| `message_reactions_ui` | Dedicated reaction summary chip and capability-gated emoji picker component extracted from the message list to keep thread rendering below SRP limits | PARTIAL |
| `message_versions` | Edit history and diffs | PARTIAL |
| `message_tombstones` | Delete/restore visibility history | PARTIAL |
| `replies` | Reply target summaries, reply-chain graph and single-hop thread reopen from projected references | PARTIAL |
| `forwards` | Forward origin metadata, forward chains, TDLib forward command path and single-hop thread reopen from projected sources | PARTIAL |
| `mentions` | Derived mention counters/filters from message metadata and unread chat state | PARTIAL |
| `join_leave` | Read-only TDLib join/leave service-message evidence; provider commands/lifecycle reconciliation still missing | PARTIAL |
| `pinned_messages` | Projection-backed pinned-message list, local message pin/unpin path and focused thread reopen within selected chat | PARTIAL |
| `reactions` | Reaction add/remove/sync plus TDLib aggregate emoji/custom reaction metadata, read-only custom aggregate evidence UI and sender-level recent emoji row upserts from provider source evidence; removal reconciliation/custom rows still missing | PARTIAL |
| `media` | TDLib raw media metadata, provider-refresh media search, download status events, completed-download projection refresh and realtime event emission | PARTIAL |
| `media_albums` | TDLib `media_album_id` projection and loaded-history Files tab grouping; provider-wide album sync/send still missing | PARTIAL |
| `attachments` | Communication attachment rows after download, projected attachment metadata refresh and Files-tab search via shared Communication attachment index | PARTIAL |
| `voice` | Voice attachment read/download only when represented by TDLib metadata | PARTIAL |
| `video_notes` | TDLib video-note attachment metadata projection plus existing download/viewer handoff | PARTIAL |
| `stickers` | TDLib sticker attachment metadata projection plus existing download/viewer handoff | PARTIAL |
| `gif_animation` | TDLib animation attachment metadata projection plus existing download/viewer handoff | PARTIAL |
| `calls` | Call metadata and fixture transcript storage | PARTIAL |
| `search` | Local UI filter; shared communication search can include channel_kind | PARTIAL |
| `provider_search` | TDLib/provider search | PARTIAL |
| `sync` | Chat and selected-history sync | PARTIAL |
| `realtime` | Shared realtime bootstrap, Telegram typed events and cache patching for message/chat/detail/pinned/search/runtime/media/typing/topic projections, including typed local chat flag, local/provider unread/mute/archive chat updates, TDLib actor typing/topic/chat-state event bridge for explicit and implicit actor starts, expiry-bounded typing UI metadata, topic list/search patching and media download lifecycle events | PARTIAL |
| `ai` | Shared-engine integration points only | PARTIAL |
| `automation` | Dry-run only, live automation blocked | PARTIAL |
| `audit` | Redacted audit for lifecycle/provider-write/dry-run | IMPLEMENTED |

## Module Boundary Rules

Telegram code may depend on:

```text
Communications
Events
Timeline interfaces
Shared attachment/blob boundary
Search engine interface
Risk/enrichment candidate interfaces
Audit
Secret resolver / host vault
```

Telegram code must not own or implement:

```text
Obligation lifecycle
Decision lifecycle
Memory lifecycle
Persona Intelligence
Organization Intelligence
Project Intelligence
```

Telegram may produce evidence and candidates for those systems only.
