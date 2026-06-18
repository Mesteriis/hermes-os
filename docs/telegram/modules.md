# Telegram Modules

Status: `COMPLETED` base-domain module map, 2026-06-18.

Telegram remains a Communication Channel. The modules below supply evidence,
commands, projections, realtime events, identity traces, timeline evidence and
media evidence. They do not implement shared Memory, Knowledge, Persona,
Organization, Project, Obligation or Decision lifecycle.

## Backend Modules

| Module | Files | Status |
|---|---|---|
| API | `backend/src/integrations/telegram/api/` | DONE |
| Accounts | `backend/src/integrations/telegram/client/accounts/` | DONE |
| Capabilities | `backend/src/domains/api_support/telegram_capabilities.rs`, `telegram_capability_catalog*.rs` | DONE |
| Dialogs | `client/chats.rs`, `client/chat_state.rs`, runtime chat events | DONE |
| Messages | `client/messages/`, `api/messages.rs`, `api/messages/` | DONE |
| Message lifecycle | `client/lifecycle/` | DONE |
| Reply/forward references | `client/references.rs` | DONE |
| Reactions | `client/reactions.rs`, `api/messages/reactions.rs` | DONE |
| Topics | `client/topics.rs`, runtime topic events | DONE |
| Runtime | `backend/src/integrations/telegram/runtime/` | DONE |
| TDLib bridge | `backend/src/integrations/telegram/tdjson/` | DONE |
| Search | `api/search.rs`, runtime search manager | DONE |
| Media | `api/media.rs`, runtime media/download manager | DONE |
| Attachments | `client/messages/attachments.rs`, shared Communication attachment boundary | DONE |
| Audit | `backend/src/platform/audit/telegram.rs` | DONE |
| Realtime | `backend/src/platform/events/`, Telegram runtime event bridge | DONE |

## Frontend Modules

| Module | Files | Status |
|---|---|---|
| Workbench page | `frontend/src/domains/telegram/views/TelegramPage.vue` | DONE |
| API clients | `frontend/src/domains/telegram/api/` | DONE |
| TanStack Query composables | `frontend/src/domains/telegram/queries/` | DONE |
| Local UI store | `frontend/src/domains/telegram/stores/telegram.ts` | DONE |
| Dialog list/actions | `TelegramChatList.vue`, `TelegramActionRail.vue`, `dialogActionHelpers.ts` | DONE |
| Thread/timeline | `TelegramMessageThread.vue`, `thread/TelegramMessageList.vue` | DONE |
| Lifecycle/reference evidence | `thread/TelegramMessageReferencePanel.vue`, lifecycle/reference queries | DONE |
| Composer | `thread/TelegramComposer.vue`, send action composables | DONE |
| Media gallery/viewer | `thread/TelegramThreadSideSections.vue`, `thread/TelegramMediaViewer.vue`, media search store | DONE |
| Search results | `TelegramSearchResultsPanel.vue`, search query composables | DONE |
| Inspector | `TelegramRail.vue`, account/runtime/capability/member/audit panels | DONE |
| Realtime cache patching | `queries/realtimeTelegramPatches.ts` and shared realtime bootstrap | DONE |

## Deferred Initiative Modules

| Initiative | Status |
|---|---|
| Bot Runtime | planned |
| Voice Recording / Voice Send | planned |
| Video Recording / Live Calls | planned |
| Session Export / Session Import | planned |
| MTProxy / SOCKS5 | planned |
| AI Summary / Translation / Bilingual Reply / AI Review Flows | planned |

## Boundary Rules

Telegram code may depend on Communications, Events, Timeline interfaces, shared
attachment/blob storage, Search, Audit and Secret resolver/host vault
boundaries.

Telegram code must not own shared engine lifecycle. It may only emit evidence,
provider commands, projections and reviewable traces consumed by those engines.
