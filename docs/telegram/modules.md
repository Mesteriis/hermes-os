# Telegram Modules

Статус: module map на 2026-06-15.

Пути ниже описывают текущую реализацию. Telegram remains a Communication
Channel; modules do not define a separate product domain.

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

## Functional Module Map

| Capability module | Назначение | Current repository status |
|---|---|---|
| `dialogs` | Account-scoped dialog/chat list | PARTIAL |
| `private_chats` | 1:1 chat projection | PARTIAL |
| `groups` | Basic group/supergroup-as-group projection | PARTIAL |
| `supergroups` | Distinct supergroup identity and permissions | MISSING |
| `channels` | Channel chat projection | PARTIAL |
| `topics` | Forum topic projection and topic-scoped replies | MISSING |
| `messages` | Source-backed message projection | PARTIAL |
| `replies` | Reply target and reply-chain graph | MISSING |
| `forwards` | Forward attribution and forward chains | MISSING |
| `mentions` | Mention counters/filters from metadata | PARTIAL |
| `pinned_messages` | Local pinned metadata display only | PARTIAL |
| `reactions` | Reaction add/remove/sync | MISSING |
| `media` | TDLib raw media metadata and download | PARTIAL |
| `attachments` | Communication attachment rows after download | PARTIAL |
| `voice` | Voice attachment read/download only when represented by TDLib metadata | PARTIAL |
| `calls` | Call metadata and fixture transcript storage | PARTIAL |
| `search` | Local UI filter; shared communication search can include channel_kind | PARTIAL |
| `sync` | Chat and selected-history sync | PARTIAL |
| `realtime` | Generic transports only; no Telegram event contract | MISSING |
| `ai` | Shared-engine integration points only | PARTIAL |
