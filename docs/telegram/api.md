# Telegram API Reference

Статус: verified route audit + целевой API scope на 2026-06-16.

Все текущие маршруты защищены локальным API guard из ADR-0056, если явно не
указано иначе. Browser WebSocket clients передают local secret через
`hermes_secret`, потому что native WebSocket requests не могут выставить
`X-Hermes-Secret`.

## Base

```text
/api/v1/telegram
```

## Capability Contract

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/capabilities` | Detailed per-operation capability matrix with `operation`, `category`, `status`, `action_class`, `reason`, `confirmation_required`, `closure_gate` |
| GET | `/api/v1/telegram/accounts/{account_id}/capabilities` | Account-scoped capability matrix with the same operation contract plus selected-account scope metadata and runtime/provider overrides |

### Целевые capability states

```text
available
degraded
blocked
unsupported
```

### Целевые operation groups

- account lifecycle;
- runtime;
- sync;
- read/search;
- send;
- edit;
- delete;
- react;
- pin;
- media download/upload;
- export;
- session/proxy;
- calls/recording;
- admin actions.

Account-scoped responses now also include:

```text
account_scope.account_id
account_scope.provider_kind
account_scope.runtime_kind
account_scope.lifecycle_state
```

Current account-aware overrides cover:

- bot accounts not using TDLib QR operations;
- user accounts not using Bot API runtime operations;
- `logged_out` / `removed` lifecycle blocking selected runtime/sync/write actions.

Telegram inspector `About` tab now also surfaces the account-scoped capability
route as a read-only matrix for the selected account, exposing operation status,
action class, reason, confirmation requirement and closure-gate metadata
alongside the existing capability-gated controls.

## Accounts

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/accounts/fixture` | Создать fixture `telegram_user` или `telegram_bot` account metadata |
| GET | `/api/v1/telegram/accounts?include_removed=` | Список Telegram provider accounts |
| POST | `/api/v1/telegram/accounts` | Создать live/live-blocked/QR-authorized Telegram account metadata и secret bindings |
| DELETE | `/api/v1/telegram/accounts/{account_id}` | Mark account `removed`, stop runtime actor, preserve local evidence |
| POST | `/api/v1/telegram/accounts/{account_id}/logout` | Mark account `logged_out`, stop runtime actor |

Account config хранит non-secret metadata. Credential payloads resolved через
host-vault/secret references. Telegram workbench `About` inspector tab now
contains a local account manager UI for `setup`, `logout` and `remove`, and the
header `Add Account` action opens that account-management surface.

### Дополнительно реализовано

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/accounts/{account_id}/capabilities` | Account-scoped detailed capability matrix |

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/accounts/{account_id}/export-session` | Sanitized session bundle export без secrets unless explicitly encrypted |
| POST | `/api/v1/telegram/accounts/import-session` | Import encrypted session bundle |
| GET/PUT | `/api/v1/telegram/accounts/{account_id}/proxy` | Proxy / MTProxy / SOCKS5 profile binding |

## Runtime

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/runtime/status?account_id=` | Account-scoped runtime status, runtime kind, TDLib readiness, split Telegram app-credential flags, TDLib probe error and derived runtime blockers |
| POST | `/api/v1/telegram/runtime/start` | Start fixture или TDLib QR-authorized runtime actor |

Runtime kinds observed:

```text
fixture
tdlib_qr_authorized
live_blocked
```

Current runtime status payload now also exposes:

```text
tdjson_path
tdjson_probe_error
telegram_api_id_configured
telegram_api_hash_configured
runtime_blockers[]
```

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/runtime/stop` | Explicit account-scoped runtime stop |
| POST | `/api/v1/telegram/runtime/restart` | Controlled restart with audit |

## QR Login

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/login/qr/start` | Start TDLib QR login setup |
| GET | `/api/v1/telegram/login/qr/{setup_id}` | Poll QR login status |
| DELETE | `/api/v1/telegram/login/qr/{setup_id}` | Cancel pending QR login session |
| POST | `/api/v1/telegram/login/qr/{setup_id}/password` | Submit 2-step verification password |

QR statuses:

```text
waiting_qr_scan
waiting_password
ready
expired
failed
runtime_unavailable
```

Telegram account management UI now exposes a QR login panel inside the
inspector `About` tab. It can call `start`, `status`, `password` and `cancel`
routes, render the current QR/2FA state, and apply suggested account metadata
back into the local account setup form before the user saves the QR-authorized
account record.

## Chats / Dialogs

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/chats?account_id=&limit=` | Projected Telegram chats ordered by last activity |
| POST | `/api/v1/telegram/sync/chats` | Sync account chats through fixture or TDLib runtime |

`telegram_chats.chat_kind` currently supports:

```text
private
group
channel
bot
```

TDLib `chatTypeSupergroup` currently collapses into `group`, unless it is a channel.

### Текущие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/chats/{chat_id}` | Projection-backed selected-chat detail |
| GET | `/api/v1/telegram/chats/{chat_id}/members` | Projection-backed top senders/member summary |
| GET | `/api/v1/telegram/chats/{chat_id}/pinned-messages?limit=` | Projection-backed pinned-message list for the selected chat |
| GET | `/api/v1/telegram/chats/search?q=&account_id=` | Projection-backed dialog search |
| GET | `/api/v1/telegram/folders?account_id=` | Projection-backed Telegram folder/chat-list filters derived from `telegram_chats.metadata.folder_name` |
| POST | `/api/v1/telegram/chats/{chat_id}/pin` | Provider/local pin command |
| POST | `/api/v1/telegram/chats/{chat_id}/archive` | Provider/local archive command |
| POST | `/api/v1/telegram/chats/{chat_id}/mute` | Mute/unmute command |
| POST | `/api/v1/telegram/chats/{chat_id}/read` | Local projected mark-read command |
| POST | `/api/v1/telegram/chats/{chat_id}/unread` | Local projected mark-unread command |

Current implementation of pin/archive/mute updates projected local chat metadata
and now records durable local command rows with realtime command-status events.
Pinned-message tab reads a projection-backed chat-scoped message list, but none
of these actions yet execute provider-synced parity. Folder filters currently
surface only projected/local folder names already persisted into chat metadata;
there is no provider folder sync route in the current backend. The Telegram
workbench action rail now consumes this dedicated folders route instead of
recomputing folder groups only from the currently loaded chat list, and the UI
now scopes that request to the selected account when one is available.

## Topics / Forums

Текущих маршрутов нет.

### Текущие lifecycle/history маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/messages/{message_id}/edit` | Record append-only local edit version and command metadata |
| POST | `/api/v1/telegram/messages/{message_id}/delete` | Record tombstone evidence and command metadata |
| POST | `/api/v1/telegram/messages/{message_id}/restore-visibility` | Record local visibility restore event, command metadata and redacted audit |
| POST | `/api/v1/telegram/messages/{message_id}/pin` | Record local pin/unpin projection state, command metadata and redacted audit |
| GET | `/api/v1/telegram/messages/{message_id}/versions` | List observed/local message versions |
| GET | `/api/v1/telegram/messages/{message_id}/tombstones` | List message tombstones |
| GET | `/api/v1/telegram/messages/{message_id}/reply-chain` | Read current reply-chain projection |
| GET | `/api/v1/telegram/messages/{message_id}/forward-chain` | Read current forward-chain projection |
| GET | `/api/v1/telegram/commands?account_id=&limit=` | List durable Telegram command rows; inspector context now surfaces them as a local audit panel with current-chat filter and text search |

These endpoints currently persist lifecycle evidence and command metadata, but
they do not yet execute provider-side edit/delete/restore parity through TDLib/Bot runtime.
Frontend now uses the reply/forward read endpoints for a read-only per-message
reference panel inside the Telegram thread, and the payload now includes local
message summaries for reply targets/replies plus source summaries for forward rows.
That same panel now also consumes `GET /versions`, `GET /tombstones`,
`GET /reactions` and `GET /commands?account_id=` as read-only lifecycle evidence.
Message pin/unpin now follows the same local-write contract and writes both
`pinned` and `is_pinned` metadata keys for projection/UI compatibility.

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/topics?account_id=&provider_chat_id=` | Topic list for forum-enabled supergroups |
| GET | `/api/v1/telegram/topics/{topic_id}/messages` | Topic-scoped timeline |
| POST | `/api/v1/telegram/topics/sync` | Topic metadata sync |

## Messages

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/messages?account_id=&provider_chat_id=&limit=` | Recent projected Telegram messages |
| POST | `/api/v1/telegram/messages` | Ingest fixture Telegram message |
| POST | `/api/v1/telegram/sync/history` | Sync selected chat history |
| POST | `/api/v1/telegram/messages/send` | Manual text send through fixture or TDLib QR-authorized runtime |

`POST /api/v1/telegram/sync/history` supports:

```text
latest
older
full
```

`older` requires `from_message_id`.

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/messages/{message_id}/reply` | Reply command with reply target |
| POST | `/api/v1/telegram/messages/{message_id}/forward` | Forward command |
| POST | `/api/v1/telegram/messages/{message_id}/mark-read` | Provider mark read |
| GET | `/api/v1/telegram/messages/{message_id}/raw` | Sanitized raw provider evidence view |

## Reactions

### Текущие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/messages/{message_id}/reactions` | Add reaction |
| DELETE | `/api/v1/telegram/messages/{message_id}/reactions` | Remove reaction |
| GET | `/api/v1/telegram/messages/{message_id}/reactions` | Current reaction projection |

Current implementation persists local reaction projection rows, durable command
metadata and redacted audit records for add/remove, exposes degraded local-write
capability states, and emits both `telegram.reaction.changed` and
`telegram.command.status_changed`, but provider-side sync/execution is still
missing.

## Media

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/media/download` | Download TDLib file and persist completed local blob + Communication attachment row |
| GET | `/api/v1/telegram/search/media?account_id=&provider_chat_id=&kind=&limit=` | Projected Telegram media gallery/filter with attachment download metadata for files-tab parity |

Fixture runtime intentionally fails closed for media downloads.
Completed TDLib downloads now also update projected Telegram message attachment
metadata (`download_state`, `local_path`, `attachment_id`, `tdlib_file_id`) and
emit `telegram.media.downloaded` with sanitized projected `message` and `chat`
snapshots for realtime cache patching.

Required fields:

```text
account_id
provider_chat_id
provider_message_id
tdlib_file_id
```

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/media` | Media gallery/search |
| GET | `/api/v1/telegram/media/{media_id}/preview` | Telegram workbench preview |
| POST | `/api/v1/telegram/media/upload` | Upload/send media attachment |
| GET | `/api/v1/telegram/media/search?q=&type=` | Provider/local media search |
| POST | `/api/v1/telegram/voice/send` | Voice message send |
| POST | `/api/v1/telegram/video-note/send` | Video note send |

## Attachments

Telegram currently reuses shared Communication attachment APIs after media download.

Target: Telegram workbench should expose Telegram-specific attachment views while
storage remains provider-neutral.

## Search

### Текущие возможности

- local chat title filter in UI;
- local loaded thread text filter;
- shared Communication search can include Telegram-projected messages by `channel_kind`.
- workspace dialog search UI backed by `GET /api/v1/telegram/chats/search`;
- workspace message search UI backed by `GET /api/v1/telegram/search/messages`;
- workspace media search UI plus files tab gallery backed by `GET /api/v1/telegram/search/media`.
- downloaded local Telegram media can open in a read-only viewer using projected
  attachment metadata and local blob paths.

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/search/provider` | Provider-side TDLib search command |

Current search contract notes:

- `GET /api/v1/telegram/chats/search` searches projected Telegram chats by
  title and optional `account_id` scope.
- `GET /api/v1/telegram/search/messages` searches projected Telegram messages by
  text and optional `account_id` / `provider_chat_id` scope.
- `GET /api/v1/telegram/search/media` is a projection-backed gallery/filter
  endpoint. It currently supports optional free-text `q`, scope and `kind`, and
  returns projected attachment download metadata (`tdlib_file_id`,
  `provider_attachment_id`, `local_path`) so files-tab gallery rows can keep
  preview/download parity outside the currently loaded message window, but it
  does not reach provider-side TDLib search.

## Automation / Policies

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/policies/templates` | List automation templates |
| POST | `/api/v1/policies/templates` | Create/update automation template |
| GET | `/api/v1/policies` | List automation policies |
| POST | `/api/v1/policies` | Create/update automation policy |
| POST | `/api/v1/policies/telegram-send/dry-run` | Telegram send policy/template validation and sanitized audit |

Automation live send is blocked. Dry-run stores preview hashes and redacted metadata.
Telegram composer now also surfaces this route as a read-only preflight panel in
the send dropdown: it loads policies/templates, filters enabled policies for the
selected account/chat, accepts template variables and renders the returned
`rendered_text` plus `rendered_preview_hash` before any live send.

## Audit

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/audit/events?target_id=&actor_id=&after_audit_id=&limit=` | Shared protected audit-event reader |

Telegram audit records must never include:

- message bodies;
- rendered variables;
- media bytes;
- passwords;
- tokens;
- app secrets;
- raw TDLib payloads.

## Calls

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/calls?account_id=&limit=` | List Telegram call metadata rows |
| POST | `/api/v1/calls` | Create/update fixture Telegram call metadata |
| GET | `/api/v1/calls/{call_id}/transcript` | Get latest stored transcript; inspector context now surfaces this through a first-class calls panel with local metadata search |
| POST | `/api/v1/calls/{call_id}/transcript` | Create fixture transcript through fixture STT provider |

Telegram workbench now consumes `GET /api/v1/calls?account_id=&limit=` through
a first-class inspector calls panel for the selected account, and that same
panel can locally search projected call metadata before loading
`GET /api/v1/calls/{call_id}/transcript` as read-only transcript evidence for
the selected projected call. Live call controls and recording flows remain
outside the current UI slice.

### Заблокированные live возможности

- live call control;
- audio capture;
- device selection;
- real STT;
- hidden recording.

## Realtime / Events

### Текущие generic routes

| Method | Path | Описание |
|---|---|---|
| GET | `/api/events/ws?after_position=&hermes_secret=` | Protected WebSocket event stream with replay/heartbeat |
| GET | `/api/events/stream?after_position=` | Protected SSE stream |
| GET | `/api/v1/events?after_position=&limit=&wait_seconds=` | Protected JSON replay/long-poll fallback |
| POST | `/api/v1/events` | Local event API command boundary |
| GET | `/api/v1/events/{event_id}` | Read single event |

### Текущие Telegram realtime contracts

```text
telegram.sync.started
telegram.sync.progress
telegram.sync.completed

telegram.message.created
telegram.message.updated
telegram.message.deleted
telegram.message.visibility_restored

telegram.media.downloaded

telegram.reaction.changed
telegram.command.status_changed
```

Frontend cache patching still accepts the legacy `telegram.message.edited`
event name for compatibility with older event-log rows or stale clients, but
new backend emissions use `telegram.message.updated`.

Telegram UI consumers should use the shared frontend realtime bootstrap backed
by `/api/events/ws`, `/api/events/stream` and `/api/v1/events`; the Telegram
workbench no longer opens a dedicated page-local realtime socket.

Current emission scope:

- fixture ingest and manual send -> `telegram.message.created`;
- lifecycle edit/delete/restore -> corresponding `telegram.message.*`;
- chat/history sync -> `telegram.sync.started/progress/completed/failed`;
- local reaction add/remove -> `telegram.reaction.changed`;
- completed TDLib media download persistence -> `telegram.media.downloaded`;
- manual send, lifecycle edit/delete/restore, reaction add/remove and current local dialog actions (`pin/unpin/archive/unarchive/mute/unmute`) -> `telegram.command.status_changed`.

Current message-event payload shape is richer than the initial coarse contract:

- top-level lifecycle fields such as `provider_chat_id`, `provider_message_id`,
  `version_number`, `reason_class`, `tombstone_id`, `is_pinned`;
- sanitized projected `message` snapshot for created/updated/deleted/visibility
  restore events;
- sanitized projected `chat` snapshot for message events when a corresponding
  `telegram_chats` projection row exists;
- deterministic `telegram_chat_id` derived from projection row or
  `account_id + provider_chat_id`, so frontend pinned-message caches can patch
  even when a dedicated `telegram_chats` row has not been materialized yet.

Current frontend realtime runtime-state usage also relies on:

- `event.metadata.account_id` for account-scoped runtime query matching;
- `telegram.sync.*` payload fields such as `scope`, `status`, `synced_count`,
  `has_more`, `provider_chat_id`;
- `telegram.command.status_changed` payload fields such as `command_id`,
  `status`, `provider_chat_id`, `telegram_chat_id`, `message_id`.

Current local dialog command payloads are also richer than the original coarse
command-status contract:

- `action` for the local dialog operation kind;
- sanitized projected `chat` snapshot for current local dialog actions, so
  frontend chat list/detail caches can patch immediately after pin/archive/
  mute/read/unread changes.

When the database-backed event log is configured, these same typed events are
also appended into canonical `event_log`, so they are available through:

- `GET /api/events/ws?after_position=&hermes_secret=`;
- `GET /api/events/stream?after_position=`;
- `GET /api/v1/events?after_position=&limit=&wait_seconds=`.

Missing parity:

- topic/chat update contracts;
- Telegram media progress/in-flight contracts beyond completed downloads;
- fully patched provider-dialog cache surfaces without fallback invalidation.

## Frontend API Client

Current frontend client:

```text
frontend/src/domains/telegram/api/telegram.ts
frontend/src/domains/telegram/queries/useTelegramQuery.ts
```

Covered:

- capabilities;
- accounts;
- chats;
- messages;
- runtime;
- QR login;
- send;
- media download;
- dry-run;
- calls.

Not covered because backend routes do not exist:

- edit/delete/reaction/pin/topic/search/provider-write parity;
- media gallery;
- provider folders;
- Telegram-specific realtime cache patching.
