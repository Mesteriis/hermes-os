# WhatsApp Architecture

Статус: целевая архитектурная спецификация на 2026-06-17.

## Позиция

WhatsApp принадлежит Communications Domain как **channel/source boundary**.
Он не является Memory, Knowledge, Task, Project, Persona, Organization,
Decision или Obligation.

WhatsApp должен поставлять Hermes:

- raw provider evidence;
- provider-specific metadata;
- messages/statuses/media/calls source records;
- provider commands;
- realtime updates;
- phone-centric identity traces;
- local desktop companion state.

## Canonical Flow

Целевой provider flow:

```text
Hidden desktop WebView
  -> WhatsApp Adapter
  -> Communication Projection
  -> Events
  -> Timeline
  -> Shared Engines
```

Detailed flow:

```text
WhatsApp Web runtime event
  -> raw provider event
  -> normalized WhatsApp source record
  -> canonical Communication projection
  -> typed whatsapp.* event
  -> Timeline evidence
  -> Search / Risk / Enrichment / AI candidates
  -> UI cache patch + replay
```

`Hidden desktop WebView` is a desktop-owned companion surface. It may run as a
secondary or background WebView after explicit owner setup, but it must expose
session state, linking, revocation, permission and failure status to the owner.
It must not become hidden headless scraping.

## Key ADR

| ADR | Значение для WhatsApp |
|---|---|
| ADR-0001 | Event sourcing is system spine |
| ADR-0013 | Local-first data ownership |
| ADR-0018 | Provider adapter boundary |
| ADR-0027 | Capability-based permission model |
| ADR-0031 | Desktop-only UI scope |
| ADR-0046 | Blob storage and scanner boundary for attachment bytes |
| ADR-0051 | WhatsApp Web companion boundary |
| ADR-0052 | Capability/action confirmation policy |
| ADR-0056 | Router-level `X-Hermes-Secret` local API auth |
| ADR-0074 | Multi-channel identity traces including WhatsApp/phone |
| ADR-0076 | Host vault for new secret payloads |
| ADR-0085 | Communication spine and Polygraph integration |
| ADR-0091 | Provider channel capability model pattern |
| ADR-0093 | Vue 3 frontend |

ADR-0051 is the controlling WhatsApp-specific decision. It keeps WhatsApp Web
as the first provider boundary and keeps Meta Business API as a separate future
provider shape.

## Provider Source Model

Primary source:

```text
WhatsApp Web
```

Initial product posture:

```text
Local First
Desktop First
Personal Use
```

Rules:

- WhatsApp Web is a linked-device companion experience.
- Live runtime requires visible owner setup, session lifecycle, local storage
  policy and smoke validation before any live capability becomes `available`.
- Session secrets, pairing material and local browser profile secrets must not
  be stored in PostgreSQL.
- Raw provider records are append-only.
- Provider quirks stay inside the adapter/source-record boundary.
- Meta Business API is not a fallback for personal WhatsApp Web.

Future source:

```text
whatsapp_business_cloud
```

That provider must use its own account, capability, auth, command and source
record contracts.

## Backend Layers

Target layers:

| Layer | Target module | Назначение |
|---|---|---|
| API routes | `backend/src/integrations/whatsapp/api/` | Account, capability, session, dialog, message, status, media and command endpoints |
| Companion runtime | `backend/src/integrations/whatsapp/runtime/` | Account-scoped WebView companion orchestration and runtime event bridge |
| Adapter/client | `backend/src/integrations/whatsapp/client/` | Provider record normalization, validation, queries and projection helpers |
| Source records | Communications compatibility boundary | Append-only raw WhatsApp provider records |
| Projection | Communications compatibility boundary | Canonical `communication_messages`, conversations, participants and attachments |
| Media storage | Communication blob/attachment boundary | Local blob storage, metadata, hashes and scanner state |
| Outbox | WhatsApp provider-write command store | Durable command lifecycle, retry and reconciliation |
| Realtime | Shared event bus | Sanitized `whatsapp.*` event emission |
| Audit | Platform audit boundary | Redacted provider-write, lifecycle and capability audit records |

Implementation status is intentionally not counted in this package. This is a
target architecture and starting audit.

## Frontend Layers

Target layers:

| Layer | Target module | Назначение |
|---|---|---|
| Route/view | `frontend/src/domains/whatsapp/views/` | Desktop WhatsApp workbench |
| API client | `frontend/src/domains/whatsapp/api/` | Typed calls to protected backend routes |
| Query hooks | `frontend/src/domains/whatsapp/queries/` | TanStack Query integration |
| Store/helpers | `frontend/src/domains/whatsapp/stores/` | Local UI state, selected account/dialog, filters and command status |
| Components | `frontend/src/domains/whatsapp/components/` | Dialog list, thread, composer, media/status/call panels and inspector |
| Realtime patches | shared platform bootstrap | Cache patching from `whatsapp.*` events |

WhatsApp views must consume the shared frontend realtime bootstrap. They must
not open a channel-specific event transport.

## Runtime Kinds

Target runtime map:

| Runtime | Назначение | Initial state |
|---|---|---|
| Fixture/manual runtime | Deterministic local validation and docs-driven smoke data | planned |
| WebView companion runtime | Owner-linked WhatsApp Web session | blocked |
| Offline command runtime | Durable local outbox and replay | planned |
| Media transfer runtime | Download/upload orchestration through companion runtime | blocked |
| Meta Business Cloud runtime | Future official business provider | unsupported |

Live WebView runtime remains `blocked` until session lifecycle, desktop
visibility, local storage, secret resolution, audit and smoke validation exist.

## Account Boundary

Supported account kinds:

```text
whatsapp_personal
whatsapp_business
```

Provider kind for the first architecture:

```text
whatsapp_web
```

Account metadata may include:

- account_id;
- account kind;
- provider kind;
- display label;
- lifecycle state;
- non-secret runtime metadata;
- local session reference;
- capability snapshot;
- audit links.

Account metadata must not include:

- pairing codes;
- session keys;
- browser cookies;
- local profile secrets;
- message bodies;
- media bytes;
- contact book exports.

Credential lookup uses:

```text
account_id + secret_purpose
```

Provider kind alone must never select credentials.

## Capability Boundary

Backend capability state is required before a WhatsApp operation appears in UI.

States:

```text
available
degraded
blocked
unsupported
```

Required operations:

| Operation | Action class | Initial state |
|---|---|---|
| send | provider_write | blocked |
| reply | provider_write | blocked |
| forward | provider_write | blocked |
| reaction | provider_write | blocked |
| delete | destructive | blocked |
| media upload | provider_write | blocked |
| media download | read/local_write | blocked |
| join group | provider_write | blocked |
| leave group | provider_write/destructive | blocked |
| status read | read | blocked |
| status publish | provider_write | blocked |
| voice send | provider_write | blocked |

`unsupported` is reserved for operations that conflict with Hermes policy or
are intentionally outside the current provider shape. `blocked` means
architecturally allowed but missing required runtime, permission, validation,
secret or adapter support.

## Dialog / Chat Model

Supported dialog classes:

```text
private chat
group
community
broadcast
status
```

Dialog records are account-scoped provider projections. They may expose:

- provider_chat_id;
- dialog kind;
- title or display name;
- participant count when observed;
- last activity timestamp;
- unread/mention counters when observed;
- mute/archive/pin local overlay when implemented;
- provider permissions and admin/member evidence when observed.

Dialog projections must not own Persona, Organization, Project, Memory,
Decision or Obligation lifecycle.

## Message Lifecycle

Target message lifecycle:

```text
provider message created
  -> raw event
  -> projected message
  -> whatsapp.message.created

provider message updated
  -> raw update evidence
  -> observed version metadata
  -> whatsapp.message.updated

provider/local delete observed
  -> raw delete evidence
  -> tombstone row
  -> whatsapp.message.deleted
```

Supported message classes:

- text;
- reply;
- forward;
- reaction;
- delete;
- edit, if supported by the provider/runtime.

Message identity needs:

- account_id;
- provider_chat_id;
- provider_message_id;
- provider_sender_id;
- message timestamp;
- raw_record_id;
- communication_message_id;
- optional reply target;
- optional forward source;
- optional edit version;
- optional tombstone state.

Hermes must not claim provider edit history that was never observed locally.

## Provider Command Outbox

WhatsApp provider writes must use the provider-write command model.

UI command path:

```text
UI intent
  -> protected backend command route
  -> capability decision
  -> provider command outbox
  -> WebView companion adapter
  -> provider-observed state
  -> Communication projection refresh
  -> whatsapp.command.status_changed / whatsapp.command.reconciled
```

Required command states:

```text
queued
executing
retrying
completed
failed
dead_letter
```

Rules:

- UI must not call WebView provider primitives directly.
- `completed` is reserved for provider-observed state, not merely a local ACK.
- Command rows must carry idempotency keys, account scope, provider target,
  capability decision, confirmation decision, retry state and redacted audit
  metadata.
- Message bodies, media bytes, secrets and raw provider payloads must not be
  stored in audit records.

## Replies, Forwards, Reactions

WhatsApp requires first-class projection for:

- reply target;
- reply context;
- forward attribution when provider exposes it;
- reaction state;
- delete/tombstone state;
- edit/update state when provider exposes it.

Raw WebView/provider data remains evidence, but UI and queries should use
projection contracts rather than parsing raw payloads directly.

## Media Lifecycle

Target lifecycle:

```text
WhatsApp media metadata
  -> media projection
  -> optional download command
  -> local blob storage
  -> scanner backend
  -> preview artifact
  -> media gallery/search index
  -> timeline attachment evidence
```

Supported media classes:

- photo;
- video;
- document;
- audio;
- voice note;
- contact;
- location;
- sticker;
- gif.

Media bytes stay out of PostgreSQL. Attachment metadata must pass through the
shared attachment safety scanner boundary. A no-op scanner records
`not_scanned`; it must not mark WhatsApp attachments as `clean`.

## Participants And Identity

WhatsApp is a phone-centric provider.

Participant evidence may include:

- phone number;
- display name;
- `wa_id`;
- group member state;
- admin state;
- community member state.

Identity traces should feed candidate flows:

```text
phone trace
  -> identity trace
  -> contact resolution evidence
  -> Persona candidate
  -> Relationship candidate
```

Rules:

- A phone number is evidence, not Persona truth.
- Display names are mutable provider labels.
- `wa_id` is provider-specific evidence, not a global identity.
- Contact resolution belongs to Persona/Relationship systems.
- WhatsApp Channel may emit candidates only.

## WhatsApp Statuses

WhatsApp Status is modeled as provider evidence, not a separate domain.

Status records may produce:

- source evidence;
- timeline evidence;
- identity signal;
- media attachment evidence;
- review candidates.

Status read is a provider read capability. Status publish is a provider-write
capability and must use the outbox. Status reactions/replies, if supported by
the provider/runtime, follow message command semantics.

## Voice Notes

Voice notes are Communication attachments with WhatsApp-specific metadata.

Supported architecture:

- voice metadata;
- voice attachment;
- local playback;
- provider download;
- future transcript integration.

Not included in the first version:

- STT;
- hidden audio capture;
- voice recording;
- automatic transcription.

Future transcripts must remain local, source-backed and permission-gated.

## Calls

First-version call scope:

- call metadata;
- call evidence;
- call timeline entries.

Out of scope:

- audio capture;
- video capture;
- call control;
- recording;
- live call handling;
- STT.

Any live call, recording or transcript runtime requires a future ADR.

## Search Architecture

Target search layers:

1. local loaded dialog filter;
2. local loaded message filter;
3. shared Communication full-text search;
4. WhatsApp provider search, only when runtime support is validated;
5. media/status search over local projections;
6. participant/phone trace search over evidence.

Search indexes are derived and rebuildable. Provider search hits that are not
projected locally are evidence candidates until raw records are preserved.

## Realtime Architecture

WhatsApp uses the shared event transports:

```text
WebSocket
SSE
Long Poll
Replay
Heartbeat
```

Required event contracts:

```text
whatsapp.message.created
whatsapp.message.updated
whatsapp.message.deleted

whatsapp.chat.updated

whatsapp.reaction.changed

whatsapp.media.download.started
whatsapp.media.download.progress
whatsapp.media.download.completed
whatsapp.media.download.failed

whatsapp.command.status_changed
whatsapp.command.reconciled
```

Events must never include:

- message body;
- media bytes;
- tokens;
- passwords;
- pairing codes;
- browser profile secrets;
- raw provider payload;
- full contact exports.

Frontend cache patching should happen before broad query invalidation, using
the shared realtime bootstrap.

## Attachment Boundary

WhatsApp attachments reuse Communication attachment metadata and local blob
storage.

Target abstraction:

```text
CommunicationBlobStore
communication_blobs
communication_attachments
```

Compatibility table/module names from earlier implementation phases must not be
treated as product architecture. No rename is required during documentation work.

## Security Rules

WhatsApp Channel follows:

```text
Evidence First
Capability Gated
Local First
Owner Controlled
No Hidden Recording
```

Additional rules:

- no secrets in PostgreSQL payload columns;
- no message bodies in audit records;
- no media bytes in event payloads;
- no invisible recording or capture;
- no provider writes from UI bypassing the backend;
- no automatic Persona merge from phone traces;
- no Meta Business API substitution for personal WhatsApp Web.

## Scope Boundary

WhatsApp Channel may prepare candidates for shared engines, but must not
implement:

- Obligation Engine;
- Decision Engine;
- Memory Engine;
- Knowledge Engine;
- Persona Intelligence;
- Organization Intelligence;
- Project Intelligence.

It may emit source-backed observations, identity traces and review candidates
only.
