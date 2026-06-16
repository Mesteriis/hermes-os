# Telegram Architecture

Статус: архитектурная ревизия и целевая спецификация на 2026-06-15.

## Позиция

Telegram принадлежит Communications Domain как **channel/source boundary**.
Он не является отдельным продуктом, не владеет памятью, знаниями, задачами,
обязательствами, решениями, проектами, организациями или персонами.

Telegram должен поставлять Hermes:

- raw provider evidence;
- provider-specific metadata;
- messages/media/calls source records;
- provider commands;
- realtime updates;
- identity traces;
- локальный desktop workbench.

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
Fixture / TDLib snapshot
  -> communication_raw_records
  -> project_raw_telegram_message
  -> communication_messages
  -> candidate refresh / shared engine integration points
```

Целевой flow:

```text
Telegram runtime event
  -> raw provider event
  -> normalized Telegram source record
  -> canonical Communication projection
  -> typed telegram.* event
  -> Timeline evidence
  -> Search / Risk / Enrichment / AI candidates
  -> UI cache patch + replay
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
| Components | `frontend/src/domains/telegram/components/` | Chat list, timeline, composer, action rail, inspector |

Realtime delivery on the frontend is shared with the rest of Hermes through
`frontend/src/platform/bootstrap/realtime.ts`; Telegram views consume query
state and cache patches instead of opening a second channel-scoped socket.

## Runtime Kinds

Текущие runtime modes:

```text
fixture
live_blocked
tdlib_qr_authorized
```

Целевой runtime map:

| Runtime | Назначение | Статус |
|---|---|---|
| Fixture runtime | Deterministic local/test validation | implemented |
| TDLib user runtime | QR-authorized live user account | partial |
| Bot API runtime | Bot account runtime | missing |
| Offline command runtime | durable local command replay | missing |
| Media capture runtime | voice/video/call capture boundary | missing |

## Account Boundary

Telegram account должен хранить:

- provider kind: `telegram_user`, `telegram_bot`;
- lifecycle state: active/logged_out/removed;
- non-secret config;
- secret references;
- runtime state;
- TDLib/session state refs;
- capability snapshot;
- audit links.

Секреты не должны попадать в account config, audit records, events или frontend state.

## Capability Boundary

Перед появлением операции в UI backend обязан вернуть capability state.

Состояния:

```text
available
degraded
blocked
unsupported
```

Каждая capability должна иметь:

- operation name;
- provider kind;
- runtime kind;
- action class;
- scope;
- reason;
- confirmation requirement;
- closure gate.

Action classes:

```text
read
write
destructive
admin
recording
export
secret-bearing
```

## Message Lifecycle

Текущий lifecycle:

```text
Fixture message or TDLib message snapshot
  -> validated NewTelegramMessage
  -> raw source record
  -> projected communication message
  -> recent message query / selected chat timeline
```

Текущие delivery states ограничены shared `communication_messages.delivery_state`:

```text
received
sent
send_dry_run
send_blocked
```

Целевой lifecycle:

```text
provider message created
  -> raw event
  -> projected message
  -> telegram.message.created

provider message edited
  -> raw edit event
  -> version row
  -> diff metadata
  -> telegram.message.updated

provider/local delete observed
  -> raw delete evidence
  -> tombstone row
  -> telegram.message.deleted

local restore visibility
  -> local visibility state
  -> telegram.message.visibility_restored
```

## Message Identity

Минимально необходимая identity model:

- account_id;
- provider_chat_id;
- provider_message_id;
- provider_sender_id;
- message_timestamp;
- raw_record_id;
- communication_message_id;
- optional topic_id;
- optional reply_to_message_id;
- optional forward_source;
- optional edit_version;
- optional tombstone state.

## Dialog / Chat Model

Текущий chat kind:

```text
private
group
channel
bot
```

Целевые distinctions:

- private chat;
- bot dialog;
- basic group;
- supergroup;
- channel;
- forum/topic-enabled supergroup;
- saved messages;
- archived chats;
- pinned chats;
- muted chats;
- folders/chat lists.

## Replies, Forwards, Reactions

Telegram требует first-class projection для:

- reply target;
- reply chain;
- forward attribution;
- forward chain;
- mentions;
- reactions;
- pinned messages;
- topic identity.

Raw TDLib JSON недостаточно для устойчивого UI и provider parity. Оно должно
сохраняться как evidence, но UI/queries должны работать через projection contract.

## Media Lifecycle

Текущий lifecycle:

```text
TDLib message raw metadata
  -> UI attachment hint from message metadata
  -> POST /api/v1/telegram/media/download
  -> TDLib downloadFile
  -> local blob write
  -> communication_attachments row
  -> scan status from attachment scanner boundary
```

Целевой lifecycle:

```text
Telegram media metadata
  -> media projection
  -> optional download command
  -> local blob storage
  -> scanner backend
  -> preview artifact
  -> media gallery/search index
  -> timeline attachment evidence
```

Media types:

- photo;
- video;
- document;
- voice note;
- video note;
- audio;
- sticker;
- GIF/animation;
- media album;
- contact/location/poll metadata.

## Attachment Boundary

Telegram attachments reuse Communication attachment metadata and local blob storage.

Current compatibility issue:

```text
MailStorageStore
communication_mail_blobs
```

This is an implementation compatibility label. Target architecture should expose
provider-neutral facade:

```text
CommunicationBlobStore
communication_blobs
communication_attachments
```

No table rename is required in documentation/audit phase.

## Search Architecture

Search layers:

1. local loaded chat filter;
2. local loaded message filter;
3. shared Communication full-text search;
4. provider-side Telegram search;
5. media search;
6. dialog/member/topic search.

Current implementation has layers 1-3 partially. Provider search and media search
are missing.

## Realtime Architecture

Generic transports already exist:

```text
WebSocket
SSE
Long Poll
Replay
Heartbeat
```

Telegram needs typed event contracts:

```text
telegram.sync.started
telegram.sync.progress
telegram.sync.completed
telegram.sync.failed

telegram.message.created
telegram.message.updated
telegram.message.deleted
telegram.message.tombstoned
telegram.message.visibility_restored

telegram.reaction.changed
telegram.chat.updated
telegram.chat.pinned
telegram.chat.archived
telegram.chat.muted
telegram.topic.updated
telegram.media.downloaded
telegram.command.status_changed
```

Events must never include:

- message body;
- media bytes;
- tokens;
- passwords;
- app secrets;
- raw provider payload;
- rendered automation variables.

Frontend should patch TanStack Query caches before invalidation, following the
Mail pattern.

## Provider Write Command Model

Manual text send currently exists. Target command model must support:

- send text;
- send media;
- edit;
- delete;
- restore local visibility;
- react/unreact;
- pin/unpin;
- mark read/unread;
- mute/unmute;
- archive/unarchive;
- join/leave;
- admin actions if ever scoped.

Each command should have:

- command_id;
- account_id;
- provider target;
- idempotency key;
- capability decision;
- user confirmation decision;
- audit metadata;
- retry/degraded state;
- per-target result rows;
- sanitized realtime events.

## Calls / Voice / STT

Current calls support fixture metadata and fixture transcripts.

Target architecture requires separate Tauri/native permission ADR for:

- microphone;
- camera;
- speaker/device selection;
- call control;
- voice recording;
- video recording;
- STT provider;
- storage retention;
- visible recording consent.

Hidden recording remains unsupported.

## Scope Boundary

Telegram Channel may prepare candidates for shared engines, but must not implement:

- Obligation Engine;
- Decision Engine;
- Memory Engine;
- Knowledge Engine;
- Persona Intelligence;
- Organization Intelligence;
- Project Intelligence.

It may emit source-backed observations and review candidates only.
