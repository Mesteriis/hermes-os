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
- successful QR/pair-code/native authorization must write session material to
  host vault and create an account-scoped `whatsapp_web_session_key` secret
  reference/binding so runtime startup can restore the account without owner
  action.

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

Current fixture foundation:

- fixture dialog records are source-backed through `WhatsAppProviderRuntime`;
- projected WhatsApp conversations already back provider-neutral conversation
  list/detail/member/search reads;
- fixture dialog metadata now carries source-backed unread count,
  participant-count, avatar/profile-picture and provider-label facts through
  canonical conversation metadata, with the same sanitized fields emitted on
  `whatsapp.dialog.updated`;
- live `/runtime-bridge/dialogs` observed reconciliation now preserves
  `provider_observed.runtime_bridge_dialog` provenance and emits canonical
  `conversation.archive.completed` runtime-event evidence through the same
  event spine contract used by fixture dialog reconciliation;
- fixture-backed `/api/v1/integrations/whatsapp/provider-sync/chats` and
  `/provider-sync/history` can return projected conversations/history for
  runtime-control flows and emit sanitized `whatsapp.sync.*` lifecycle events;
- live runtime sync, paging, reconciliation breadth and realtime runtime
  bridging are still required.

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

Current fixture foundation:

- fixture message update records are source-backed through
  `WhatsAppProviderRuntime`;
- fixture message metadata can already carry broader structured provider facts
  for mentions, link previews, polls, location, contact cards, stickers,
  join/leave service updates, system/service messages, ephemeral flags and
  view-once flags through canonical `message_metadata`;
- projected WhatsApp message/status evidence can already reuse the shared
  Communications summary workflow to mirror review candidates for
  `new_person`, `new_organization` and `knowledge_candidate` items, so
  people/organization discovery remains event-driven and review-driven rather
  than a direct provider-to-Personas write path;
- fixture reply/forward evidence can project canonical
  `communication_message_refs` when provider reference metadata is observed;
- accepted update signals project into canonical
  `communication_message_versions`;
- fixture message delete records project into canonical
  `communication_message_tombstones`;
- fixture receipt records project into canonical
  `communication_messages.delivery_state`;
- live `/runtime-bridge/messages`, `/message-updates`, `/message-deletes`,
  `/reactions`, `/media`, `/dialogs`, `/participants` and `/statuses` now also
  stamp raw-evidence provenance with their respective
  `provider_observed.runtime_bridge_*` sources, so accepted Signal Hub evidence
  keeps live-runtime origin instead of collapsing back into fixture-only raw
  metadata before downstream consumers see it;
- live `/runtime-bridge/receipts` now also stamps raw-evidence provenance with
  `provider_observed.runtime_bridge_receipt`, so receipt observations keep
  their live-runtime ingress origin before Signal Hub acceptance;
- accepted WhatsApp receipt events now also participate in the shared
  provider-observation reconciliation consumer, so later receipt evidence can
  advance durable command `delivery_state` after the original
  provider-observed message completion has established a stable
  `provider_message_id`;
- live `/runtime-bridge/runtime-events` now also stamps raw-evidence
  provenance with `provider_observed.runtime_bridge_runtime_event`, while
  live `/runtime-bridge/sync-lifecycle` and `/runtime-bridge/media-lifecycle`
  captured runtime events preserve their lifecycle `source` markers as raw
  `observed_source` values instead of collapsing back into fixture-only or
  internal capture semantics;
- provider-neutral `/api/v1/communications/messages/{message_id}/reply-chain`
  and `/api/v1/communications/messages/{message_id}/forward-chain` can now read
  canonical WhatsApp refs;
- live provider lifecycle consumer, reconciliation and realtime runtime event
  bridge are still required.
- live command workers should receive explicit provider/runtime dispatch
  metadata from `/runtime-bridge/commands/claim` (`provider_kind`,
  `provider_shape`, `runtime_kind`, `lifecycle_state`,
  `session_restore_available`, `runtime_blockers`) plus durable execution
  state (`capability_state`, `action_class`, `confirmation_decision`,
  `provider_state`, `result_payload`) instead of relying on hidden account
  lookups.
- live command failure reports should also be able to carry structured
  `error_code` and retry-delay hints so a replaceable runtime worker can feed
  the durable retry/dead-letter path without collapsing everything into one
  opaque error string.
- the durable retry path should use the same capped exponential backoff policy
  for runtime-reported failures and stale interrupted executions, with
  structured failure metadata preserved in durable command state.

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

Current fixture foundation:

- fixture reaction records are source-backed through `WhatsAppProviderRuntime`;
- accepted reaction signals project into `communication_message_reactions`;
- provider-neutral `/api/v1/communications/messages/{message_id}/reactions`
  can now read canonical WhatsApp reaction rows;
- live runtime reaction execution is still required; fixture
  provider-observed reaction reconciliation now completes durable command rows
  through the event spine.

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

Current fixture foundation:

- fixture media metadata is source-backed through `WhatsAppProviderRuntime`;
- media metadata projects into `communication_attachments`;
- `/api/v1/integrations/whatsapp/provider-sync/media` can now return
  projected WhatsApp attachment snapshots from canonical attachment storage,
  optionally scoped by `provider_chat_id` and `content_type`;
- provider-neutral `/api/v1/communications/search/media` can already return
  projected WhatsApp attachments from canonical attachment storage;
- fixture media command retries now emit sanitized
  `whatsapp.media.upload.started|progress|completed` and
  `whatsapp.media.download.started|progress|completed` lifecycle events through
  the event spine while reconciling durable provider-command rows;
- scanner state is `not_scanned` until a real scanner backend provides a
  verdict;
- blocked-safe `/api/v1/integrations/whatsapp/provider-media/upload` and
  `/provider-media/download` routes now persist durable command rows and emit
  sanitized `whatsapp.media.*.(requested|failed)` events, and those blocked-safe
  media lifecycle phases now also materialize canonical accepted
  `whatsapp.runtime_event` evidence through the shared raw-signal path;
- fixture executor `whatsapp.media.upload.started|progress|completed` and
  `whatsapp.media.download.started|progress|completed` phases now also
  materialize canonical accepted `whatsapp.runtime_event` evidence through the
  same shared raw-signal path while keeping the existing realtime lifecycle
  events;
- `/runtime-bridge/sync-lifecycle` now also accepts `scope = media` so an
  external runtime can publish attachment/media sync phases into the same event
  spine contract without fixture-only endpoints;
- live transfer execution/progress/completion, blob byte persistence and media
  reconciliation are still required.

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

Current fixture foundation:

- fixture status records are source-backed through `WhatsAppProviderRuntime`;
- statuses project as provider-neutral Communication messages with status
  metadata;
- fixture status records can now also materialize source-backed author
  identity evidence into canonical `communication_identities` and link that
  identity through status message metadata when explicit author identity fields
  are present;
- projected fixture status messages now also run through the shared
  decision/task candidate refresh path, giving status evidence the same
  Review-bound candidate extraction boundary as other Communications evidence;
- fixture status lifecycle events now also satisfy the generic Timeline replay
  contract by carrying `subject.entity_id` plus `source.kind` and
  `source.source_id`, so status evidence can already enter Timeline through the
  shared event-log replay path;
- projected fixture status messages now also upsert a synthetic canonical
  `status-feed` conversation surface, so provider-neutral conversation
  list/detail/search can expose WhatsApp statuses without introducing a Status
  domain;
- `/api/v1/integrations/whatsapp/runtime-bridge/statuses` now reuses the same
  status-feed projection path and can reconcile live/runtime-bridge
  `publish_status` completion through provider-observed evidence without
  mislabeling that provenance as fixture-only;
- live `/runtime-bridge/status-views` and `/runtime-bridge/status-deletes` now
  also preserve explicit raw-evidence provenance as
  `provider_observed.runtime_bridge_status_view` and
  `provider_observed.runtime_bridge_status_delete` instead of collapsing those
  observed paths into fixture-only ingress metadata;
- fixture media metadata can already attach to projected status messages through
  the canonical attachment path when provider evidence targets the status
  provider id;
- fixture replies can already target projected status messages through canonical
  reply refs;
- blocked-safe and retried `publish_status` command lifecycle now also
  materializes canonical accepted `whatsapp.runtime_event` evidence for
  `status.publish.requested|failed|started|completed`, in addition to durable
  provider-command reconciliation and status projection;
- live status feed, status media, identity trace handoff and publish command are
  still required.

### 7.5 Presence

Target capabilities:

- online/offline observations;
- typing observations;
- recording-audio observations;
- last-seen evidence where observed;
- sanitized realtime presence patches;
- identity-trace metadata updates without creating Personas directly.

Presence must remain observed provider fact, not inferred truth.

Current fixture foundation:

- fixture presence records are source-backed through `WhatsAppProviderRuntime`;
- presence evidence emits raw and accepted Signal Hub events through the event
  spine;
- known canonical communication identities can be patched with observed
  `presence_state`, `last_seen_at` and related provenance metadata;
- sanitized realtime `whatsapp.presence.changed` events now exist for fixture
  presence ingest;
- `/api/v1/integrations/whatsapp/provider-sync/presence` now exposes an
  integration-side presence snapshot surface over canonical identity metadata,
  optionally scoped by `provider_chat_id`, and emits the same sanitized
  `whatsapp.sync.*` plus accepted runtime-event evidence as the other sync
  control flows;
- live `/runtime-bridge/presence` now also stamps raw-evidence provenance with
  `provider_observed.runtime_bridge_presence`, so accepted Signal Hub evidence
  can distinguish live runtime ingest from fixture-only observation paths;
- `/runtime-bridge/sync-lifecycle` now also accepts `scope = presence` so an
  external runtime can publish presence-sync lifecycle phases into the same
  event spine contract;
- live runtime presence feed and broader frontend/runtime bridging are still
  required.

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

Current fixture foundation:

- fixture dialog records are source-backed through `WhatsAppProviderRuntime`;
- fixture dialog metadata projects into canonical conversation rows and can now
  carry source-backed community/subgroup linkage, invite-link evidence and
  broadcast/newsletter/community-root flags;
- provider-neutral Communications conversation list/detail/search already read
  projected WhatsApp conversation rows;
- fixture participant records are source-backed through `WhatsAppProviderRuntime`;
- fixture participant metadata projects into canonical identities and
  conversation participants;
- repeated participant evidence can now update a stable canonical participant
  row, preserve previous/current role and status facts in participant metadata,
  and emit sanitized `whatsapp.participant.changed` lifecycle events;
- live `/runtime-bridge/participants` observed reconciliation now preserves
  `provider_observed.runtime_bridge_participant` provenance and emits canonical
  `group.join.completed` runtime-event evidence through the shared event spine
  instead of inventing a separate live-only completion path;
- provider-neutral conversation member reads already resolve projected WhatsApp
  participant rows;
- live roster sync, group/community reconciliation and membership commands are
  still required.

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

Current fixture foundation:

- phone/wa_id/display-name traces can be projected into canonical communication
  identities from fixture participant evidence;
- fixture participant evidence now also carries source-backed `push_name`,
  `business_profile` and `profile_photo_ref` traces through canonical
  communication identity and participant metadata;
- fixture participant and status ingest now also create unattached
  compatibility `person_identities` traces for `whatsapp` and `phone` with
  observation links, so Persona assignment stays explicit and review-driven;
- fixture participant ingest also records `message_participant` traces for
  group/community member evidence, and fixture message ingest records phone
  traces from contact-card evidence through the same compatibility boundary;
- these compatibility traces now also retain source-backed WhatsApp evidence
  metadata for push name, business profile, profile photo refs, participant
  role/status and contact-card payloads;
- canonical `communication_identities.metadata` now also preserves
  `display_name_history` and previous/current observed display names when
  repeated WhatsApp evidence changes the visible provider name;
- `/api/v1/integrations/whatsapp/provider-sync/contacts` now exposes an
  integration-side contact snapshot over canonical communication identities
  plus compatibility WhatsApp/phone trace metadata, so a replaceable runtime
  can resync contact/identity evidence without coupling to Personas;
- `/runtime-bridge/sync-lifecycle` now also accepts `scope = contacts` so an
  external runtime can publish contact-sync lifecycle phases into the same
  event spine contract;
- no Persona is created directly from WhatsApp integration projection;
- automatic Persona merge remains out of scope for WhatsApp integration.

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

Current foundation:

- blocked-safe text send/reply/forward/edit/delete/react/unreact, voice-note
  send, status publish, join/leave, mark-read/unread, archive/unarchive,
  mute/unmute and pin/unpin command rows are durable;
- the current WhatsApp Communications panel can already drive projected
  send/reply/forward/edit/delete flows plus conversation read/unread,
  pin/unpin, archive/unarchive and mute/unmute through the provider-neutral
  `/api/v1/communications/*` routes instead of going through integration-only
  endpoints for day-to-day communication mutations;
- idempotency keys are required;
- `whatsapp_provider_write_commands` is mirrored into canonical
  `communication_provider_commands`;
- command list, manual retry and manual dead-letter integration-control
  endpoints exist;
- fixture provider-observed dialog state now completes durable `archive`,
  `unarchive`, `pin`, `unpin`, `mute`, `unmute`, `mark_read` and
  `mark_unread` command rows through the event spine;
- a background fixture command executor can now claim retried outbox rows and
  drive supported fixture command kinds such as `send_text`, `reply`,
  `forward`, `edit`, `delete`, `react`/`unreact`, media upload/download,
  voice-note send, dialog-state changes, group join/leave and status publish
  into provider-observed completion;
- live provider execution, full backoff worker coverage and provider-observed
  completion are still required before send/mutate capabilities can become
  available.

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
whatsapp.presence.changed
whatsapp.call.updated
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

Current fixture foundation:

- sanitized WhatsApp projection events for status, participant lifecycle and
  call metadata now carry `subject.entity_id` and source references compatible
  with the generic `TimelineEngine::replay_event_log` contract;
- exact WhatsApp fixture coverage verifies that these events replay into
  Timeline entries with source-backed references rather than using a
  WhatsApp-specific timeline path.

### 14. Calls

First target:

- call metadata only;
- missed/incoming/outgoing state where provider exposes it;
- call duration where observed;
- Timeline evidence;
- relationship signal.

Current fixture foundation:

- fixture call-metadata records are source-backed through
  `WhatsAppProviderRuntime`;
- call evidence emits raw and accepted Signal Hub events through the event
  spine using `call_metadata` event kind;
- fixture call metadata can upsert the existing calls read model with observed
  direction/state/timestamps and provenance;
- sanitized realtime `whatsapp.call.updated` events now exist for fixture call
  ingest;
- `/api/v1/integrations/whatsapp/provider-sync/calls` now exposes an
  integration-side call snapshot surface over the shared calls read model,
  optionally scoped by `provider_chat_id`, and emits the same sanitized
  `whatsapp.sync.*` plus accepted runtime-event evidence as the other sync
  control flows;
- live `/runtime-bridge/calls` now also stamps raw-evidence provenance with
  `provider_observed.runtime_bridge_call`, so accepted call evidence preserves
  live ingress provenance before Timeline/search/review consumers read it;
- `/runtime-bridge/sync-lifecycle` now also accepts `scope = calls` so an
  external runtime can publish call-sync lifecycle phases into the same event
  spine contract;
- live runtime call feed, timeline workflow integration and broader
  relationship-signal handling are still required.

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
- projected WhatsApp message and status evidence now already feed shared
  knowledge candidates through Review using the same Communications summary
  contract path as other channels;
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
