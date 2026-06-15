# Telegram Architecture

Статус: текущая архитектурная ревизия на 2026-06-15.

## Позиция

Telegram принадлежит Communications Domain как channel/source boundary. Он не
является отдельным продуктом и не владеет Memory, Knowledge, Obligations,
Decisions, Projects или Personas.

## Canonical Flow

```text
Telegram Provider
  -> Raw Records
  -> Communication Projection
  -> Events
  -> Timeline
  -> Shared Engines
```

Текущий backend flow для сообщений:

```text
Fixture/TDLib snapshot
  -> communication_raw_records
  -> project_raw_telegram_message
  -> communication_messages
  -> candidate refresh / shared engine integration points
```

## Key ADR

| ADR | Значение для Telegram |
|---|---|
| ADR-0001 | Event sourcing is system spine |
| ADR-0018 | Provider adapter boundary |
| ADR-0031 | Desktop-only UI scope |
| ADR-0046 | Blob storage and scanner boundary for attachment bytes |
| ADR-0050 | V4 Telegram policy automation and call intelligence |
| ADR-0052 | Capability/action confirmation policy |
| ADR-0056 | Router-level `X-Hermes-Secret` local API auth |
| ADR-0076 | Host vault for new secret payloads |
| ADR-0083 | Account-scoped TDLib runtime slice |
| ADR-0085 | Communication spine and Polygraph integration |
| ADR-0091 | Production Telegram capability model |
| ADR-0093 | Vue 3 frontend |

## Backend Layers

| Layer | Current files | Назначение |
|---|---|---|
| API routes | `backend/src/integrations/telegram/api/` | Account, capability, runtime, QR, chat, message and media endpoints |
| Runtime manager | `backend/src/integrations/telegram/runtime/` | Fixture and TDLib account actor orchestration |
| Client/store | `backend/src/integrations/telegram/client/` | Account metadata, chat projection, message ingestion, queries, attachment anchors |
| TDLib boundary | `backend/src/integrations/telegram/tdjson/` | JSON request builders, parsing, QR login, native TDLib loading |
| Source records | `backend/src/domains/mail/core/` compatibility boundary | Raw provider records and provider accounts |
| Projection | `backend/src/domains/mail/messages/` compatibility boundary | Canonical `communication_messages` projection |
| Media storage | `backend/src/domains/mail/storage/` compatibility boundary | Local blob and attachment metadata/scanner boundary |
| Audit | `backend/src/platform/audit/telegram.rs` | Redacted provider-write, automation and lifecycle audit records |
| Calls | `backend/src/platform/calls/` | Telegram call metadata and fixture transcripts |

## Frontend Layers

| Layer | Current files | Назначение |
|---|---|---|
| Route/view | `frontend/src/app/views/TelegramView.vue`, `frontend/src/domains/telegram/views/TelegramPage.vue` | Desktop Telegram workbench |
| API client | `frontend/src/domains/telegram/api/telegram.ts` | Typed calls to protected backend routes |
| Query hooks | `frontend/src/domains/telegram/queries/useTelegramQuery.ts` | TanStack Query integration |
| Store/helpers | `frontend/src/domains/telegram/stores/telegram.ts` | Local UI state, filters, derived lists |
| Components | `frontend/src/domains/telegram/components/` | Chat list, thread, composer, action rail, inspector |

## Realtime

Current repository has generic protected event transports:

- `GET /api/events/ws?after_position=&hermes_secret=`
- `GET /api/events/stream?after_position=`
- `GET /api/v1/events?after_position=&limit=&wait_seconds=`

Telegram-specific event contracts are not implemented. Current Telegram routes
use query invalidation and manual reload paths rather than typed
`telegram.*` realtime events. Provider sync progress, reaction updates, edits,
deletes and message lifecycle events are therefore `MISSING` for Telegram.

## Message Lifecycle

Current lifecycle:

```text
Fixture message or TDLib message snapshot
  -> validated `NewTelegramMessage`
  -> raw source record
  -> projected communication message
  -> recent message query / selected chat timeline
```

Implemented states are limited to `received`, `sent`, `send_dry_run` and
`send_blocked` in the shared `communication_messages.delivery_state` constraint.

Missing lifecycle slices:

- edit command and observed edit version persistence;
- provider/local delete and tombstone history;
- restore visibility;
- reply/reply-chain projection;
- forward/forward-chain projection;
- reaction update projection;
- pinned message command/projection.

## Media Lifecycle

Current lifecycle:

```text
TDLib message raw metadata
  -> UI attachment hint from message metadata
  -> POST /api/v1/telegram/media/download
  -> TDLib downloadFile
  -> local blob write
  -> communication_attachments row
  -> scan status from attachment scanner boundary
```

Fixture runtime fails closed for media download. Completed TDLib downloads are
copied into local blob storage; PostgreSQL stores metadata and blob references.

Missing media slices:

- first-class Telegram media gallery;
- preview endpoint specific to Telegram workbench;
- provider-side media search;
- upload/send attachments;
- voice/video recording;
- persisted preview artifacts.

## Attachment Flow

Telegram attachments reuse `communication_attachments` and
`communication_mail_blobs` today. This is an accepted compatibility label from
ADR-0083, but the domain documentation treats the boundary as provider-neutral:
Communication attachment metadata plus local blob storage.

No scanner backend marks Telegram attachments `clean`; no-op scanner status
remains `not_scanned` unless a real scanner backend is introduced.

## Sync Flow

Current sync paths:

- fixture sync returns already projected chats/messages;
- `tdlib_qr_authorized` sync uses an account-scoped actor;
- chat sync calls TDLib chat APIs and upserts `telegram_chats`;
- selected history sync calls TDLib history APIs and projects messages;
- older history uses `from_message_id` pagination.

Global background sync, sync progress events, offline outbox and provider
checkpoint UI are not implemented for Telegram.
