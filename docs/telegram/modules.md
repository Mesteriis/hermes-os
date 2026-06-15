# Telegram Modules

Статус: module map на 2026-06-15.

Telegram остаётся Communication Channel. Модули ниже не создают отдельный product domain.

## Backend Modules

| Module | Current files | Назначение | Status |
|---|---|---|---|
| `api` | `backend/src/integrations/telegram/api/` | Protected HTTP handlers for capabilities, accounts, QR login, runtime, chats, messages and media | IMPLEMENTED |
| `accounts` | `backend/src/integrations/telegram/client/accounts/` | Fixture/live account setup, host-vault secret binding, lifecycle logout/remove | PARTIAL |
| `capabilities` | `backend/src/domains/api_support/messaging_integrations.rs` | Coarse capability contract for runtime, automation and call/STT availability | PARTIAL |
| `dialogs` | `telegram_chats`, `client/chats.rs` | Local chat/dialog projections from fixture or TDLib snapshots | PARTIAL |
| `messages` | `client/messages/` | Fixture/TDLib ingestion, raw record projection, recent query and manual send response assembly | PARTIAL |
| `runtime` | `backend/src/integrations/telegram/runtime/` | Account-scoped fixture/TDLib actor manager with start/status/sync/send/media commands | PARTIAL |
| `tdjson` | `backend/src/integrations/telegram/tdjson/` | TDLib native loading, JSON requests, parsing, QR login worker | PARTIAL |
| `media` | `runtime/media.rs`, `runtime/manager/media_download.rs` | On-demand TDLib download into local blob/attachment storage | PARTIAL |
| `attachments` | `client/messages/attachments.rs`, mail storage compatibility boundary | Anchor Telegram messages to Communication attachments | PARTIAL |
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
| `provider_search` | TDLib/provider search | Needed beyond loaded-message filtering |
| `bot_runtime` | Bot API runtime | Needed for `telegram_bot` parity |
| `session_proxy` | Session bundle/proxy profiles | Needed for portable/local-first runtime configuration |
| `realtime_events` | Sanitized `telegram.*` event contracts | Needed for realtime Telegram UX |

## Frontend Modules

| Module | Current files | Назначение | Status |
|---|---|---|---|
| `page` | `frontend/src/domains/telegram/views/TelegramPage.vue` | Desktop three-pane Telegram workbench | PARTIAL |
| `api` | `frontend/src/domains/telegram/api/telegram.ts` | Typed backend API calls and UI helper wrappers | IMPLEMENTED |
| `queries` | `frontend/src/domains/telegram/queries/useTelegramQuery.ts` | TanStack Query hooks/mutations for accounts, chats, messages, calls and runtime | PARTIAL |
| `store` | `frontend/src/domains/telegram/stores/telegram.ts` | Pinia UI state, filters, counts and derived message/file/link/pinned views | PARTIAL |
| `dialogs` | `TelegramChatList.vue` | Virtualized chat list with local filters and metadata badges | PARTIAL |
| `messages` | `TelegramMessageThread.vue`, `thread/TelegramMessageList.vue` | Selected chat timeline, local text search, older-history trigger | PARTIAL |
| `composer` | `thread/TelegramComposer.vue` | Text-only manual send UI; attachment/voice buttons disabled | PARTIAL |
| `media` | `thread/TelegramThreadSideSections.vue` | File/link/pinned/timeline tabs derived from loaded messages | PARTIAL |
| `inspector` | `TelegramRail.vue` | Placeholder inspector for context/members/about | MISSING |

## Missing / Target Frontend Modules

| Module | Назначение |
|---|---|
| `account_setup` | QR login, account lifecycle, capability status and live runtime controls |
| `chat_inspector` | Members, about, permissions, topic list, linked personas/projects |
| `topic_view` | Forum/topic scoped timeline |
| `reply_thread_view` | Reply-chain navigation |
| `reaction_bar` | Reaction display, add/remove and sync state |
| `message_actions` | Edit/delete/reply/forward/pin/read commands with capability gates |
| `media_viewer` | Photo/video/document/voice preview and download manager |
| `media_gallery` | Media search and grouped media browsing |
| `voice_player` | Voice playback, transcript, summary handoff |
| `calls_panel` | Call metadata, transcript and future call controls |
| `global_search` | Dialog/message/media search UI |
| `runtime_status` | TDLib/Bot capability and degraded state diagnostics |
| `realtime_status` | Telegram-specific event stream/live sync state |

## Functional Module Map

| Capability module | Назначение | Current repository status |
|---|---|---|
| `accounts` | Account metadata, lifecycle, secrets | PARTIAL |
| `runtime_fixture` | Deterministic local/test runtime | IMPLEMENTED |
| `runtime_tdlib_user` | TDLib QR-authorized user runtime | PARTIAL |
| `runtime_bot` | Bot API runtime | MISSING |
| `dialogs` | Account-scoped dialog/chat list | PARTIAL |
| `private_chats` | 1:1 chat projection | PARTIAL |
| `groups` | Basic group/supergroup-as-group projection | PARTIAL |
| `supergroups` | Distinct supergroup identity and permissions | MISSING |
| `channels` | Channel chat projection | PARTIAL |
| `topics` | Forum topic projection and topic-scoped replies | MISSING |
| `messages` | Source-backed message projection | PARTIAL |
| `message_versions` | Edit history and diffs | MISSING |
| `message_tombstones` | Delete/restore visibility history | MISSING |
| `replies` | Reply target and reply-chain graph | MISSING |
| `forwards` | Forward attribution and forward chains | MISSING |
| `mentions` | Mention counters/filters from metadata | PARTIAL |
| `pinned_messages` | Local pinned metadata display only | PARTIAL |
| `reactions` | Reaction add/remove/sync | MISSING |
| `media` | TDLib raw media metadata and download | PARTIAL |
| `attachments` | Communication attachment rows after download | PARTIAL |
| `voice` | Voice attachment read/download only when represented by TDLib metadata | PARTIAL |
| `video_notes` | Dedicated video note support | MISSING |
| `stickers` | Sticker projection/viewer | MISSING |
| `gif_animation` | GIF/animation projection/viewer | MISSING |
| `calls` | Call metadata and fixture transcript storage | PARTIAL |
| `search` | Local UI filter; shared communication search can include channel_kind | PARTIAL |
| `provider_search` | TDLib/provider search | MISSING |
| `sync` | Chat and selected-history sync | PARTIAL |
| `realtime` | Generic transports only; no Telegram event contract | MISSING |
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
