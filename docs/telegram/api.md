# Telegram API Reference

Статус: verified route audit на 2026-06-15.

All routes are protected by the local API guard from ADR-0056 unless documented
otherwise. Browser WebSocket clients pass the local secret through the
`hermes_secret` query parameter because native WebSocket requests cannot set
`X-Hermes-Secret`.

## Base Telegram Routes

Base: `/api/v1/telegram`

## Capabilities

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/telegram/capabilities` | Coarse capability contract for fixture runtime, TDLib readiness, bot runtime block, automation, calls and STT |

Capability states currently returned by backend are `available` and `blocked`.
ADR-0091 also defines `degraded` and `unsupported`; detailed per-feature
coverage is tracked in [Gap Analysis](gap-analysis.md).

## Accounts

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/telegram/accounts/fixture` | Create fixture `telegram_user` or `telegram_bot` account metadata for tests/local validation |
| GET | `/api/v1/telegram/accounts?include_removed=` | List Telegram provider accounts; removed accounts are hidden unless explicitly requested |
| POST | `/api/v1/telegram/accounts` | Create live/live-blocked/QR-authorized Telegram account metadata and secret bindings |
| DELETE | `/api/v1/telegram/accounts/{account_id}` | Mark account `removed`, stop runtime actor and preserve local evidence |
| POST | `/api/v1/telegram/accounts/{account_id}/logout` | Mark account `logged_out` and stop runtime actor |

Account config stores non-secret metadata. Credential payloads are resolved via
host-vault/secret references. Current setup supports `telegram_user` and
`telegram_bot` provider kinds; live Bot API runtime is blocked.

## Runtime

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/telegram/runtime/status?account_id=` | Account-scoped runtime status, runtime kind, TDLib readiness and live-send flag |
| POST | `/api/v1/telegram/runtime/start` | Start fixture or TDLib QR-authorized account runtime actor |

Runtime kinds observed in current implementation include `fixture`,
`tdlib_qr_authorized` and `live_blocked`.

## QR Login

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/telegram/login/qr/start` | Start TDLib QR login setup; uses configured app credentials when payload omits them |
| GET | `/api/v1/telegram/login/qr/{setup_id}` | Poll QR login status |
| DELETE | `/api/v1/telegram/login/qr/{setup_id}` | Cancel pending QR login session |
| POST | `/api/v1/telegram/login/qr/{setup_id}/password` | Submit 2-step verification password for pending QR session |

QR login statuses include `waiting_qr_scan`, `waiting_password`, `ready`,
`expired`, `failed` and `runtime_unavailable`.

## Chats / Dialogs

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/telegram/chats?account_id=&limit=` | List projected Telegram chats ordered by last activity |
| POST | `/api/v1/telegram/sync/chats` | Sync account chats through fixture or TDLib runtime |

`telegram_chats.chat_kind` currently supports `private`, `group`, `channel` and
`bot`. TDLib `chatTypeSupergroup` is collapsed into `group` unless it is a
channel.

## Messages

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/telegram/messages?account_id=&provider_chat_id=&limit=` | List recent projected Telegram messages |
| POST | `/api/v1/telegram/messages` | Ingest fixture Telegram message into raw records and communication projection |
| POST | `/api/v1/telegram/sync/history` | Sync selected chat history through fixture or TDLib runtime |
| POST | `/api/v1/telegram/messages/send` | Manual text send through fixture or TDLib QR-authorized runtime; records redacted provider-write audit |

`POST /api/v1/telegram/sync/history` supports `mode=latest`, `older` and `full`
in the request body. `older` requires `from_message_id`.

There are no current endpoints for edit, delete, restore, reply, forward,
reaction, pin or provider mark-read/unread.

## Media

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/telegram/media/download` | Download a TDLib file through the account actor and persist completed files through the Communication attachment/blob scanner boundary |

Fixture runtime intentionally fails closed for media downloads. The endpoint
requires `account_id`, `provider_chat_id`, `provider_message_id` and
`tdlib_file_id`.

There are no Telegram-specific preview, upload, media search or media gallery
endpoints.

## Automation / Policies

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/policies/templates` | List automation templates |
| POST | `/api/v1/policies/templates` | Create/update automation template |
| GET | `/api/v1/policies` | List automation policies |
| POST | `/api/v1/policies` | Create/update automation policy |
| POST | `/api/v1/policies/telegram-send/dry-run` | Run Telegram send policy/template validation and record sanitized audit metadata |

Automation live send is blocked. Dry-run stores preview hashes and redacted
metadata; rendered body, variables and source context are not written to audit.

## Audit

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/audit/events?target_id=&actor_id=&after_audit_id=&limit=` | Shared protected audit-event reader; includes Telegram send, dry-run, logout and remove audit records when filtered by target or actor |

Telegram audit records must stay sanitized: no message bodies, rendered
automation variables, media bytes, passwords, tokens or app secrets.

## Calls

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/calls?account_id=&limit=` | List Telegram call metadata rows |
| POST | `/api/v1/calls` | Create/update fixture Telegram call metadata |
| GET | `/api/v1/calls/{call_id}/transcript` | Get latest stored transcript for a call |
| POST | `/api/v1/calls/{call_id}/transcript` | Create fixture transcript through fixture STT provider |

Live call control, audio capture, device selection and real STT are blocked.
Hidden recording is unsupported.

## Realtime / Events

| Method | Path | Description |
|---|---|---|
| GET | `/api/events/ws?after_position=&hermes_secret=` | Protected WebSocket event stream with replay/heartbeat |
| GET | `/api/events/stream?after_position=` | Protected SSE stream |
| GET | `/api/v1/events?after_position=&limit=&wait_seconds=` | Protected JSON replay/long-poll fallback |
| POST | `/api/v1/events` | Local event API command boundary |
| GET | `/api/v1/events/{event_id}` | Read single event |

Telegram-specific realtime event contracts such as new message, edit, delete,
reaction update and sync update are not currently implemented.

## Frontend API Client

Current frontend client:

- `frontend/src/domains/telegram/api/telegram.ts`
- `frontend/src/domains/telegram/queries/useTelegramQuery.ts`

The client covers current backend routes for capabilities, accounts, chats,
messages, runtime, QR login, send, media download, dry-run and calls. It does
not expose edit/delete/reaction/search/provider-write parity APIs because those
routes do not exist.
