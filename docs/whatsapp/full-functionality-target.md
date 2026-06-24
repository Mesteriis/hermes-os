# WhatsApp Full Functionality Target for Hermes

Status: target product/architecture specification.
Date: 2026-06-24.

Goal: implement full WhatsApp functionality inside Hermes without violating the Hermes ownership model.

WhatsApp in Hermes is not a standalone messenger clone. It is a provider/runtime integration that supplies source evidence into the Communications domain and the Hermes memory/intelligence system.

## Core invariant

```text
A channel is never a domain.
A channel is an integration.
A communication is the domain object.
```

Therefore:

```text
WhatsApp Runtime
  -> Provider Observation
  -> Signal Hub / Event Store
  -> Communications Projection
  -> Radar / Review / Timeline / Search / Engines
  -> Domain commands only through workflows
```

## Provider shapes

Hermes should model WhatsApp as a provider family, not as one implementation.

| Provider kind | Purpose | Status | Notes |
|---|---|---|---|
| `whatsapp_web_companion` | Visible desktop WebView/WhatsApp Web companion runtime | Target/safe baseline | Closest to existing ADR-0051 language. |
| `whatsapp_native_md` | Native unofficial WhatsApp multi-device protocol runtime | Proposed experimental provider | Candidate path for full functionality through Rust libraries. |
| `whatsapp_business_cloud` | Official Meta WhatsApp Business Platform Cloud API | Future provider | Business-only semantics; not a personal account substitute. |

Current repository provider kind `whatsapp_web` may remain as compatibility alias while the provider shape is clarified.

## Functional target matrix

### 1. Accounts and sessions

Target capabilities:

- multiple WhatsApp accounts;
- personal WhatsApp account via companion/native provider;
- WhatsApp Business App account via companion/native provider;
- future Meta Business Cloud account as separate provider;
- QR code pairing;
- pair-code/phone-number linking where provider supports it;
- persistent sessions;
- session health and reconnect diagnostics;
- logout, revoke, relink and remove;
- account-scoped local state path;
- account-scoped secret/session protection material;
- no session secrets in PostgreSQL.

Lifecycle states:

```text
created
link_required
qr_pending
pair_code_pending
linked
syncing
available
degraded
blocked
revoked
removed
```

### 2. Dialogs and conversation surfaces

Target source dialog kinds:

- private chat;
- group chat;
- community;
- community subgroup;
- broadcast list, if provider exposes it;
- channel/newsletter, if provider exposes it;
- status feed.

Projection target:

```text
communication_conversations
communication_conversation_participants
communication_identities
communication_channels
```

Dialog state:

- title/display name;
- avatar/profile picture metadata;
- unread count;
- last message;
- pinned/archive/mute/starred overlays;
- provider labels/folders where available;
- local Hermes folders/saved searches separately from provider state;
- source-backed participant count and role metadata.

### 3. Messages

Target message classes:

- text;
- reply/quote;
- forward;
- edit/update, observed versions only;
- delete/tombstone;
- reaction;
- poll;
- contact card;
- location;
- link preview;
- system/service message;
- ephemeral/disappearing message metadata where observed;
- view-once metadata, without bypassing provider privacy semantics.

Required message metadata:

- `account_id`;
- `channel_kind`;
- provider conversation id;
- provider message id;
- sender provider identity id;
- sender display name;
- participant/identity trace links;
- occurred/observed/projected timestamps;
- raw record id;
- source fingerprint;
- delivery/read/play receipt state;
- provenance;
- confidence.

Message lifecycle must store only observed provider facts. Hermes must not invent edit history, missing deletes, missing receipts or unobserved provider metadata. Apparently facts need evidence now. Humanity had a long run without it.

### 4. Reactions

Target capabilities:

- project reaction state;
- add reaction;
- change reaction;
- remove reaction;
- reconcile provider-observed reaction state;
- emit realtime patch events.

Projection target:

```text
communication_message_reactions
```

Command path:

```text
UI
  -> Communications reaction command
  -> communication.provider_command.requested
  -> WhatsApp integration executes
  -> provider-observed reaction evidence
  -> communication.provider_command.completed
```

### 5. Media and attachments

Target media classes:

- photo;
- video;
- document;
- audio;
- voice note;
- sticker;
- GIF;
- contact card;
- location;
- avatar/profile picture;
- status media.

Required rules:

- media bytes live in local blob storage, not PostgreSQL;
- PostgreSQL stores metadata, hashes, local blob refs, scanner state and provenance;
- previews require safe preview artifacts;
- no file is marked `clean` without scanner evidence;
- download/upload is command-driven and observable;
- media dedup uses hashes and provider refs;
- voice notes are attachments first, transcripts later.

Media lifecycle events:

```text
whatsapp.media.download.requested
whatsapp.media.download.started
whatsapp.media.download.progress
whatsapp.media.download.completed
whatsapp.media.download.failed
whatsapp.media.upload.requested
whatsapp.media.upload.completed
whatsapp.media.upload.failed
```

### 6. Voice notes

Target capabilities:

- voice note metadata;
- download;
- local playback;
- waveform/duration metadata where available;
- send voice note from local blob;
- future transcript handoff to shared transcription engine only after explicit permission and ADR.

Unsupported until future ADR:

- hidden microphone capture;
- hidden recording;
- automatic STT on private messages;
- live call capture.

### 7. Status

WhatsApp Status is source evidence, not a new domain.

Target capabilities:

- status read projection;
- status media metadata;
- status author identity trace;
- status publish command;
- status reply message link;
- Timeline evidence;
- Radar candidate if status looks important.

Projection target:

```text
communication_raw_records(record_kind = whatsapp_status)
communication_attachments
Timeline evidence projection
```

### 8. Groups, communities and membership

Target capabilities:

- group metadata projection;
- participant list projection;
- admin/member role evidence;
- join by invite where provider supports it;
- leave group;
- approve membership where provider supports it;
- group subject/description/icon changes where provider supports it;
- community/subgroup linkage;
- membership change events;
- role changes;
- invite link evidence.

All provider writes must go through the outbox and must require capability checks.

### 9. Contacts and identity traces

WhatsApp is phone-centric. Phone numbers and `wa_id` values are identity evidence, not Persona truth.

Target identity trace kinds:

- phone number;
- `wa_id`;
- display name;
- push name;
- business profile;
- profile photo ref;
- contact card;
- group member evidence;
- community member evidence.

Handoff targets:

```text
Persona candidate
Relationship candidate
Organization candidate
Trust/Risk signal
```

WhatsApp integration must never merge Personas by itself.

### 10. Provider command outbox

Provider writes include:

- send text;
- send media;
- send voice note;
- reply;
- forward;
- edit;
- delete;
- react;
- mark read/unread;
- archive/unarchive;
- mute/unmute;
- pin/unpin;
- join/leave group;
- publish status;
- update profile fields, if allowed.

Command states:

```text
queued
confirmed
executing
retrying
completed
failed
dead_letter
cancelled
```

Completion requires provider-observed reconciliation, not merely “we called a function and it didn’t explode.” Software has tried that religion. It ended poorly.

### 11. Realtime

Realtime event families:

```text
whatsapp.runtime.status_changed
whatsapp.session.link_state_changed
whatsapp.dialog.updated
whatsapp.message.created
whatsapp.message.updated
whatsapp.message.deleted
whatsapp.reaction.changed
whatsapp.receipt.changed
whatsapp.media.download.*
whatsapp.command.status_changed
whatsapp.command.reconciled
```

Rules:

- realtime payloads are sanitized;
- no message bodies in broad runtime events unless the event is explicitly scoped and authorized;
- no secrets, QR secrets, session material or raw provider payloads;
- frontend patches Communications caches for product data;
- frontend patches Integrations caches only for runtime/setup state.

### 12. Search

Search targets:

- message text;
- chat title;
- participant display names;
- media filename/caption;
- status text/caption;
- phone number traces;
- extracted entities;
- decisions/tasks/obligations candidates linked to messages.

Search ownership:

```text
Search engine / Communications read model owns index.
WhatsApp integration only supplies source evidence.
```

### 13. Timeline

Timeline events:

- message observed;
- important message detected;
- media received;
- status observed;
- call metadata observed;
- group membership change;
- provider command completed/failed;
- Radar promotion;
- task/decision/obligation candidate created.

Timeline entries must preserve source evidence links.

### 14. Calls

First target:

- call metadata only;
- missed/incoming/outgoing state where provider exposes it;
- call duration where observed;
- Timeline evidence;
- relationship signal.

Unsupported until separate ADR:

- call answering;
- call placement;
- microphone/camera integration;
- recording;
- live capture;
- automatic transcription;
- WebRTC/native call runtime.

### 15. Hermes intelligence layer

WhatsApp evidence should feed Hermes features through existing shared engines and workflows.

#### AI Summary

- conversation summary;
- thread summary;
- daily digest;
- participant-specific summary;
- source-backed citations/evidence links;
- confidence and freshness.

#### AI Reply

- draft suggestions only;
- multilingual reply flow;
- tone/style constraints;
- owner confirmation before send;
- provider command outbox for execution.

#### Task extraction

Flow:

```text
communication.message.recorded
  -> radar.signal.detected
  -> review.item.created
  -> task.candidate.detected
  -> review/promotion
  -> task command
```

No direct `WhatsApp -> Task` mutation.

#### Decision extraction

- detect decisions from conversation;
- preserve message evidence;
- attach rationale and participants;
- create suggested Decision only through workflow/review.

#### Obligation extraction

- detect commitments, promises, deadlines, waiting state;
- create obligation/task candidates with evidence;
- never mark as truth without review or strong rule.

#### Note and knowledge extraction

- create Knowledge Note candidates;
- source link to message/status/media;
- confidence and evidence;
- promotion to Documents/Notes/Knowledge through Review/Radar.

#### Persona and relationship intelligence

- communication DNA;
- relationship health;
- trust/risk signals;
- identity resolution candidates;
- memory cards;
- dossier context packs.

All of these are Hermes features, not WhatsApp-owned features.

### 16. Rules and workflows

Target rules:

- detect urgent WhatsApp messages;
- detect waiting-for replies;
- detect follow-up needed;
- detect invoices/contracts/documents in media;
- detect unknown phone numbers;
- detect risky links/files;
- detect important group changes;
- detect project/org/person mentions;
- detect possible spam/scam.

Rules create:

```text
Radar item
Review item
Candidate
Notification
Context pack input
```

They do not mutate final domain state directly.

### 17. Privacy and safety

Mandatory rules:

- owner-visible runtime status;
- explicit linking/unlinking;
- explicit send confirmation according to capability policy;
- no hidden recording;
- no hidden scraping UI;
- no raw provider dumps in logs;
- no secrets in DB/events/frontend;
- no AI training/fine-tuning on WhatsApp data;
- local-first storage;
- export/delete/revocation policy;
- attachment scanning and safe preview.

### 18. Business Cloud future provider

`whatsapp_business_cloud` is valid only for official Business Platform use cases:

- business phone numbers;
- WABA assets;
- templates;
- webhooks;
- media endpoints;
- product catalogs;
- flows;
- business policy/rate-limit semantics.

It must not be used as a pretend personal WhatsApp provider.

## Target backend module map

```text
backend/src/integrations/whatsapp/
├── mod.rs
├── api/
│   ├── accounts.rs
│   ├── capabilities.rs
│   ├── runtime.rs
│   ├── sessions.rs
│   ├── sync.rs
│   ├── media.rs
│   └── commands.rs
├── runtime/
│   ├── mod.rs
│   ├── supervisor.rs
│   ├── provider_runtime.rs
│   ├── web_companion.rs
│   ├── native_md.rs
│   ├── business_cloud.rs
│   ├── qr.rs
│   ├── pair_code.rs
│   ├── session_store.rs
│   └── health.rs
├── adapter/
│   ├── mod.rs
│   ├── inbound_mapper.rs
│   ├── outbound_mapper.rs
│   ├── ids.rs
│   ├── validation.rs
│   └── capabilities.rs
├── source_records/
│   ├── messages.rs
│   ├── dialogs.rs
│   ├── participants.rs
│   ├── reactions.rs
│   ├── media.rs
│   ├── statuses.rs
│   └── calls.rs
├── commands/
│   ├── outbox_consumer.rs
│   ├── send.rs
│   ├── media.rs
│   ├── reactions.rs
│   ├── dialogs.rs
│   └── reconciliation.rs
└── tests/
    ├── fixtures.rs
    ├── contract.rs
    └── runtime_smoke.rs
```

Important: `integrations/whatsapp` may depend on platform/contracts, platform/events, secrets/runtime storage and external libraries. It must not mutate business domains directly.

## Target frontend module map

```text
frontend/src/integrations/whatsapp/
├── api/
├── queries/
├── stores/
├── views/
│   └── WhatsAppRuntimePanel.vue
└── components/
    ├── RuntimeStatus.vue
    ├── LinkFlow.vue
    ├── CapabilityMatrix.vue
    └── CommandAuditPanel.vue

frontend/src/domains/communications/providers/whatsapp/
├── filters/
├── badges/
└── provider-specific communication render helpers only
```

Provider setup/runtime belongs to `frontend/src/integrations/whatsapp`.

User-facing conversation/message work belongs to `frontend/src/domains/communications`.

## Acceptance criteria

A full WhatsApp implementation is not complete until all of the following are true:

1. WhatsApp live runtime has explicit owner-visible lifecycle.
2. QR/pair-code link flow is implemented or explicitly unsupported per provider shape.
3. Session secrets never enter PostgreSQL, logs, events or frontend state.
4. Inbound messages, dialogs, participants, reactions, media metadata and statuses become source-backed Communication projections.
5. Provider writes use durable command outbox, capability checks, audit and reconciliation.
6. Media uses local blob storage and scanner state.
7. Phone identity traces create Persona candidates only.
8. Tasks, obligations, decisions, notes and knowledge are created only through Radar/Review/workflows.
9. Realtime events patch correct provider-neutral caches.
10. Live provider tests are manual/smoke and never required in CI.
11. Fixture tests cover every source record and command class.
12. Documentation, ADRs and architecture guard agree.
