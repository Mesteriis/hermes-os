# WhatsApp Architectural Blockers

Статус: стартовые audit blockers на 2026-06-17.

Блокеры ниже фиксируют причины, последствия и план решения. Они не являются
разрешением на реализацию новых крупных подсистем вне WhatsApp Channel.

## 1. WebView Companion Boundary

**Причина**: ADR-0051 permits WhatsApp Web only through an explicit companion
boundary. The product request uses `Hidden WebView`, but ADR-0051 rejects hidden
headless scraping and requires owner-visible desktop/runtime controls.

**Последствия**: A silent WebView runtime would violate the provider boundary
and create privacy, session and terms-of-use risk.

**План решения**:

- define Hidden WebView as desktop-owned companion runtime, not headless
  scraping;
- expose link, session, revocation and runtime status to the owner;
- keep live runtime `blocked` until smoke validation exists;
- log only redacted lifecycle/audit metadata;
- do not store session secrets in PostgreSQL.

## 2. No Official Personal WhatsApp API

**Причина**: Personal WhatsApp support has no stable documented API equivalent
to email IMAP or a first-party personal messaging API. WhatsApp Web is a
linked-device companion surface.

**Последствия**: Provider behavior may change, live runtime may degrade, and
tests cannot depend on live personal accounts.

**План решения**:

- preserve fixture/manual validation path;
- isolate provider quirks inside the adapter;
- report runtime fragility through capability states;
- avoid hard dependencies on unverified selectors or private protocols in
  canonical architecture;
- keep Meta Business API as a separate future provider.

## 3. Account And Session Lifecycle Missing

**Причина**: WhatsApp needs account-scoped session state, local WebView profile
storage, lifecycle transitions and revocation semantics before live runtime can
be safe.

**Последствия**: Sync, provider writes and media transfer cannot be reliably
started, stopped, audited or recovered.

**План решения**:

- define `whatsapp_personal` and `whatsapp_business` account metadata;
- define lifecycle states;
- use `account_id + secret_purpose` for session protection material;
- store local WebView state only under ignored local data paths;
- preserve local evidence on logout/remove unless explicit destructive purge is
  separately implemented.

## 4. Capability Contract Granularity

**Причина**: WhatsApp operations need the same operation-level capability model
as other provider channels before UI exposure.

**Последствия**: UI cannot reliably distinguish available, degraded, blocked and
unsupported operations for send, reply, forward, reaction, delete, media,
groups, status and voice.

**План решения**:

- add backend capability contract;
- model action class, scope, reason and confirmation requirement;
- start live provider writes as `blocked`;
- mark calls control, recording and STT as `unsupported`;
- add fixture tests before enabling UI controls.

## 5. Provider-Write Command Outbox Missing

**Причина**: WhatsApp sends, replies, forwards, reactions, deletes, media
uploads, group joins/leaves, status publishes and voice sends are provider-side
effects.

**Последствия**: Direct UI-to-provider calls would bypass capability policy,
audit, retry, idempotency and reconciliation.

**План решения**:

- route every provider write through backend command APIs;
- persist command states `queued`, `executing`, `retrying`, `completed`,
  `failed`, `dead_letter`;
- require provider-observed evidence before `completed`;
- emit `whatsapp.command.status_changed` and
  `whatsapp.command.reconciled`;
- keep audit metadata redacted.

## 6. Raw Evidence And Projection Schema Missing

**Причина**: WhatsApp needs append-only raw records and canonical Communication
projections for messages, statuses, media, participants and calls.

**Последствия**: UI or AI would be tempted to work from runtime snapshots
without replayable source evidence.

**План решения**:

- define raw record kinds such as `whatsapp_web_message`,
  `whatsapp_web_status`, `whatsapp_web_media` and `whatsapp_web_call`;
- keep raw records append-only;
- project into canonical Communications;
- preserve provider identifiers and observed timestamps;
- avoid storing media bytes or secrets in raw/audit/event payloads.

## 7. Phone-Centric Identity Resolution

**Причина**: WhatsApp identity is centered on phone numbers, display names and
provider-specific `wa_id` values, not email addresses or stable global person
IDs.

**Последствия**: Incorrect merges could corrupt Persona/Relationship state if
phone traces are treated as truth.

**План решения**:

- store phone numbers, `wa_id` and display names as identity traces;
- preserve source and confidence metadata;
- emit Persona/Relationship candidates only;
- keep merge/split lifecycle outside WhatsApp;
- track display-name history as observed evidence.

## 8. Dialog And Status Model Missing

**Причина**: WhatsApp has private chats, groups, communities, broadcasts and
statuses. Statuses are not equivalent to ordinary chats, but they still produce
Communication evidence.

**Последствия**: Treating Status as a separate domain or ignoring it would break
Timeline, identity-signal and source-evidence consistency.

**План решения**:

- model status as provider evidence under WhatsApp Channel;
- produce Timeline evidence and identity signals;
- keep status publish as provider-write command;
- do not create a Status domain;
- keep broadcast/community distinctions at provider projection boundary.

## 9. Media And Attachment Safety Missing

**Причина**: WhatsApp media includes photos, videos, documents, audio, voice
notes, contacts, locations, stickers and GIFs. Media bytes are untrusted input.

**Последствия**: Unsafe preview, PostgreSQL blob storage or missing scanner
state would violate local-first storage and attachment safety rules.

**План решения**:

- store bytes in local blob storage, not PostgreSQL;
- store metadata, hashes, scanner state and local refs in database records;
- default scanner state to `not_scanned`;
- never mark attachments `clean` without scanner backend;
- use explicit download commands and progress events.

## 10. Realtime Contracts Missing

**Причина**: WhatsApp requires typed events for messages, chats, reactions, media
downloads and command reconciliation.

**Последствия**: Frontend would fall back to broad reloads and could miss
source-backed state changes.

**План решения**:

- define sanitized `whatsapp.*` event payloads;
- emit events at projection and command boundaries;
- include stable identifiers for cache patching;
- exclude message bodies, media bytes, raw payloads and secrets;
- consume events through the shared realtime bootstrap.

## 11. Calls Boundary Requires Future ADR

**Причина**: Calls touch microphone, camera, speakers, live media devices,
recording and potentially STT. The first version permits metadata only.

**Последствия**: Implementing call control, capture or transcription now would
violate the no-hidden-recording boundary.

**План решения**:

- support only call metadata, call evidence and Timeline entries;
- mark audio/video capture, call control, recording, live handling and STT
  `unsupported`;
- require future ADR before any live call runtime;
- keep UI explicit that no recording occurs.

## 12. Voice Notes Need Attachment-First Design

**Причина**: Voice notes are media attachments, but they are often treated like
messages or transcripts in product surfaces.

**Последствия**: Early STT or hidden recording could bypass permission,
provenance and source-evidence boundaries.

**План решения**:

- model voice notes as metadata plus attachment;
- support local playback only after download;
- leave transcript integration as future shared-engine handoff;
- require explicit permission before any recording or STT work;
- keep transcript output source-backed and reviewable.

## 13. Meta Business API Provider Confusion

**Причина**: Meta Business Platform Cloud API is official for business messaging
but has different onboarding, auth, account, template and policy semantics than
personal WhatsApp Web.

**Последствия**: Using it as the primary architecture would break the personal
local-first WhatsApp Web boundary and mix incompatible provider assumptions.

**План решения**:

- keep WhatsApp Web as primary provider source;
- document Meta Business API as future provider only;
- use distinct provider kind such as `whatsapp_business_cloud`;
- create separate ADR before implementation;
- do not reuse personal WebView commands for Cloud API semantics.

## 14. Test And Validation Path Missing

**Причина**: Live WhatsApp Web validation cannot be a default CI dependency.
Fixture/manual state is required before any live companion runtime can be
validated safely.

**Последствия**: Without deterministic tests, provider changes and session
fragility would create hidden regressions.

**План решения**:

- create fixture/manual records first;
- test idempotent raw record ingestion;
- test capability states;
- test outbox state transitions;
- keep live WebView smoke tests opt-in;
- never require live WhatsApp credentials in CI.

## 15. Cross-Domain Temptation

**Причина**: WhatsApp naturally exposes people, groups, obligations, decisions,
tasks, organizations, projects, locations and documents.

**Последствия**: Hermes would get channel-specific mini-domains and duplicated
business logic if WhatsApp implements those lifecycles directly.

**План решения**:

- WhatsApp may emit evidence and candidates only;
- lifecycle belongs to shared domains/engines;
- do not implement Memory, Knowledge, Persona, Organization, Project, Decision,
  Obligation or Task logic inside WhatsApp Channel;
- preserve source references for every candidate.
