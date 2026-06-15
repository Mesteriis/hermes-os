# Telegram Architectural Blockers

Статус: audit blockers на 2026-06-15.

Блокеры ниже фиксируют причины, последствия и план решения. Они не являются
разрешением на реализацию новых крупных функций в текущей documentation phase.

## 1. Capability Contract Granularity

**Причина**: `/api/v1/telegram/capabilities` currently reports coarse runtime
and automation states. ADR-0091 requires every Telegram operation to be
represented in backend capability state before UI exposure.

**Последствия**: UI cannot reliably distinguish missing, blocked, degraded and
unsupported operations for edits, deletes, reactions, topics, exports, proxies,
session bundles, calls and destructive actions.

**План решения**: Extend the backend capability contract before adding new
controls. Model per-operation capability, action class, scope, reason and
closure gate. Add fixture tests for each new state.

## 2. TDLib Runtime Dependency

**Причина**: Live user runtime depends on a loadable native TDLib JSON runtime,
Telegram app credentials, QR-authorized account metadata and account-scoped
TDLib state paths.

**Последствия**: Live sync/send/media features are not generally available in
CI or on machines without configured TDLib resources. Fixture runtime must
remain the deterministic validation path.

**План решения**: Keep live validation opt-in. Document TDLib resource setup,
preserve fixture runtime, and only mark live capabilities `available` when
runtime dependency checks and smoke validation pass.

## 3. Bot Runtime Missing

**Причина**: `telegram_bot` account setup and secret references exist, but no
Bot API runtime adapter is implemented.

**Последствия**: Bot accounts can be represented, but live bot send/sync
capabilities must remain `blocked`.

**План решения**: Add a separate ADR-backed Bot API runtime slice if needed.
Keep bot credentials account-scoped and host-vault backed.

## 4. Mail-Named Blob / Storage Facade

**Причина**: Telegram media persistence currently uses mail-named storage
modules and tables (`MailStorageStore`, `communication_mail_blobs`) as the
existing Communication attachment/blob boundary.

**Последствия**: The implementation is functionally usable but semantically
confusing. It can make docs and future code incorrectly imply Telegram belongs
to Mail.

**План решения**: Introduce a provider-neutral Communication attachment/blob
facade when a storage refactor is explicitly scoped. Do not rename tables in
this documentation phase.

## 5. No Tombstone / Version Schema

**Причина**: Current message projection upserts canonical communication rows.
There is no durable Telegram tombstone table, observed edit-version table or
deletion-history event model.

**Последствия**: Delete, restore visibility, edit history and diff views cannot
be safely implemented without losing source-evidence semantics.

**План решения**: Add an ADR and migration before implementing destructive
provider commands or observed edit history. Preserve raw evidence and create
append-only tombstone/version events.

## 6. No Telegram Realtime Event Contracts

**Причина**: Generic WebSocket/SSE/long-poll transports exist, but Telegram does
not emit typed event contracts for new messages, edits, deletes, reactions or
sync progress.

**Последствия**: Frontend must reload/query-invalidate manually and cannot
provide reliable live Telegram UX.

**План решения**: Define sanitized `telegram.*` event payloads, avoiding message
bodies/media bytes/secrets in event or audit records. Add frontend cache patch
handlers after backend contracts are stable.

## 7. No Topic / Reaction / Reply / Forward Projection Schema

**Причина**: TDLib raw payload is preserved, but dedicated projection fields for
topics, replies, forwards, forward chains, mentions and reactions are not
modeled.

**Последствия**: UI can show only shallow selected-chat timelines and metadata
derived opportunistically from raw payload. Provider parity features would be
fragile if implemented directly against raw JSON.

**План решения**: Add explicit projection tables or JSON contract fields for
topic identity, reply target, forward attribution, reaction state and mention
state before exposing commands or review UX.

## 8. Provider-Write Command Model Beyond Send

**Причина**: Manual text send is the only implemented Telegram provider-write
command. Edit, delete, react, pin, mark read/unread, archive, join/leave and
admin commands do not share a durable command/outbox model.

**Последствия**: High-risk and destructive actions cannot be retried, audited,
explained or rolled back consistently.

**План решения**: Design an account-scoped provider command model with
idempotency keys, per-message result records, capability decisions, audit
metadata and retry/degraded state before adding provider-write parity.

## 9. Desktop Media Permissions

**Причина**: Voice/video recording and live calls require desktop microphone,
camera and device-selection boundaries. Current code has fixture call metadata
and fixture STT only.

**Последствия**: Voice/video messages, real local transcription, call accept,
decline, redial and audio capture remain blocked. Hidden recording stays
unsupported.

**План решения**: Add a separate Tauri/native permission ADR and runtime slice
before enabling media capture or live call controls.
