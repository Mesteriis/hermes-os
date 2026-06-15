# Telegram API Reference

Статус: verified route audit + целевой API scope на 2026-06-15.

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
| GET | `/api/v1/telegram/capabilities` | Coarse capability contract для fixture runtime, TDLib readiness, bot runtime block, automation, calls и STT |

Текущий backend в основном возвращает coarse states. Для production Telegram scope
нужна детальная capability matrix.

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
host-vault/secret references.

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/accounts/{account_id}/export-session` | Sanitized session bundle export без secrets unless explicitly encrypted |
| POST | `/api/v1/telegram/accounts/import-session` | Import encrypted session bundle |
| GET/PUT | `/api/v1/telegram/accounts/{account_id}/proxy` | Proxy / MTProxy / SOCKS5 profile binding |
| GET | `/api/v1/telegram/accounts/{account_id}/capabilities` | Account-scoped detailed capability matrix |

## Runtime

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/runtime/status?account_id=` | Account-scoped runtime status, runtime kind, TDLib readiness, live-send flag |
| POST | `/api/v1/telegram/runtime/start` | Start fixture или TDLib QR-authorized runtime actor |

Runtime kinds observed:

```text
fixture
tdlib_qr_authorized
live_blocked
```

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/runtime/stop` | Explicit account-scoped runtime stop |
| POST | `/api/v1/telegram/runtime/restart` | Controlled restart with audit |
| GET | `/api/v1/telegram/runtime/health?account_id=` | Detailed TDLib/native dependency health |

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

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/chats/search?q=&account_id=` | Backend dialog search |
| GET | `/api/v1/telegram/chats/{chat_id}` | Full projected chat detail |
| GET | `/api/v1/telegram/chats/{chat_id}/members` | Participants/members projection |
| POST | `/api/v1/telegram/chats/{chat_id}/pin` | Provider/local pin command |
| POST | `/api/v1/telegram/chats/{chat_id}/archive` | Provider/local archive command |
| POST | `/api/v1/telegram/chats/{chat_id}/mute` | Mute/unmute command |
| GET | `/api/v1/telegram/folders` | Telegram folders/chat lists |
| POST | `/api/v1/telegram/folders/sync` | Provider folder sync |

## Topics / Forums

Текущих маршрутов нет.

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
| POST | `/api/v1/telegram/messages/{message_id}/edit` | Edit provider message |
| POST | `/api/v1/telegram/messages/{message_id}/delete` | Delete provider/local message with tombstone evidence |
| POST | `/api/v1/telegram/messages/{message_id}/restore-visibility` | Restore local visibility after tombstone/local hide |
| POST | `/api/v1/telegram/messages/{message_id}/reply` | Reply command with reply target |
| POST | `/api/v1/telegram/messages/{message_id}/forward` | Forward command |
| POST | `/api/v1/telegram/messages/{message_id}/pin` | Pin/unpin message |
| POST | `/api/v1/telegram/messages/{message_id}/mark-read` | Provider mark read |
| GET | `/api/v1/telegram/messages/{message_id}/versions` | Observed edit versions/diffs |
| GET | `/api/v1/telegram/messages/{message_id}/raw` | Sanitized raw provider evidence view |

## Reactions

Текущих маршрутов нет.

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/messages/{message_id}/reactions` | Add reaction |
| DELETE | `/api/v1/telegram/messages/{message_id}/reactions` | Remove reaction |
| GET | `/api/v1/telegram/messages/{message_id}/reactions` | Current reaction projection |

## Media

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/media/download` | Download TDLib file and persist completed local blob + Communication attachment row |

Fixture runtime intentionally fails closed for media downloads.

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

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/search/messages?q=&account_id=&chat_id=` | Telegram message search |
| GET | `/api/v1/telegram/search/dialogs?q=&account_id=` | Dialog search |
| GET | `/api/v1/telegram/search/media?q=&account_id=&type=` | Media search |
| POST | `/api/v1/telegram/search/provider` | Provider-side TDLib search command |

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
| GET | `/api/v1/calls/{call_id}/transcript` | Get latest stored transcript |
| POST | `/api/v1/calls/{call_id}/transcript` | Create fixture transcript through fixture STT provider |

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

### Недостающие Telegram event contracts

```text
telegram.sync.started
telegram.sync.progress
telegram.sync.completed
telegram.sync.failed

telegram.message.created
telegram.message.edited
telegram.message.deleted
telegram.message.tombstoned
telegram.message.visibility_restored

telegram.reaction.changed
telegram.chat.updated
telegram.topic.updated
telegram.media.downloaded
telegram.command.status_changed
```

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
