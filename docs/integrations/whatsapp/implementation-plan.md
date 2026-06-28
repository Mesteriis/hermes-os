# WhatsApp Implementation Plan

Status: target implementation plan.
Date: 2026-06-24.

This plan starts from the current fixture/runtime foundation and moves toward full WhatsApp functionality plus Hermes intelligence.

## Principle

Implementation must proceed by contracts, not by provider UI temptation.

Correct path:

```text
Provider event
  -> observation/signal
  -> Communications projection
  -> Radar/Review/Timeline/Search/Engines
  -> workflows
  -> domain commands
```

Incorrect path:

```text
WhatsApp adapter
  -> Tasks/Personas/Documents/Knowledge directly
```

That path is how architecture becomes soup.

## Phase P0 — Documentation and decision closure

Goal: make the target explicit before touching live accounts.

Deliverables:

- `docs/integrations/whatsapp/current-audit-2026-06-24.md`;
- `docs/integrations/whatsapp/full-functionality-target.md`;
- `docs/integrations/whatsapp/rust-provider-research.md`;
- `docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md`;
- update `docs/integrations/whatsapp/README.md` with the new document set;
- update `docs/integrations/whatsapp/api.md` after route design is accepted.

Acceptance:

- provider shapes are named;
- full capability matrix exists;
- third-party Rust project choice is documented;
- ToS/account-risk posture is explicit;
- no code claims live runtime support.

## Phase P1 — Provider runtime contract

Goal: define a runtime abstraction that supports WebView, native multi-device and future business cloud providers without changing domain code.

Backend target modules:

```text
backend/src/integrations/whatsapp/runtime/
├── mod.rs
├── provider_runtime.rs
├── supervisor.rs
├── session_store.rs
├── health.rs
├── qr.rs
├── pair_code.rs
├── web_companion.rs
├── native_md.rs
└── business_cloud.rs
```

Core trait sketch:

```rust
pub trait WhatsAppProviderRuntime: Send + Sync {
    fn provider_shape(&self) -> WhatsappProviderShape;
    async fn start(&self, account_id: &str) -> Result<RuntimeStatus, RuntimeError>;
    async fn stop(&self, account_id: &str) -> Result<RuntimeStatus, RuntimeError>;
    async fn link_qr(&self, account_id: &str) -> Result<QrLinkSession, RuntimeError>;
    async fn link_pair_code(&self, account_id: &str, phone: &str) -> Result<PairCodeSession, RuntimeError>;
    async fn health(&self, account_id: &str) -> Result<RuntimeHealth, RuntimeError>;
    async fn store_authorized_session_credential(&self, account_id: &str, session_material: SecretMaterial) -> Result<CredentialBinding, RuntimeError>;
}
```

Acceptance:

- all live capabilities default to `blocked`;
- application code depends on `dyn WhatsAppProviderRuntime`, not a concrete provider library;
- `whatsapp_web_companion` runtime health exposes a bridge contract manifest,
  not a live-availability flag: it requires an owner-visible desktop WebView,
  forbids hidden/headless mode, lists the protected runtime-bridge event routes
  for all inbound/runtime/media/sync families, fixes provider writes to durable
  outbox claim/failure paths with provider-observed reconciliation, and excludes
  session material, cookies, browser profile secrets, QR/pair-code artifacts,
  message bodies and media bytes from health/event-like payloads;
- `whatsapp_web_companion` has a Tauri visible-desktop producer shell:
  `open_whatsapp_web_companion` creates or focuses an account-scoped
  `https://web.whatsapp.com/` WebView and
  `whatsapp_web_companion_manifest` returns the same sanitized event/outbox
  contract. This shell is not live availability: the companion window receives
  only an explicit remote capability for the metadata relay dispatch, the
  commands do not read cookies/session/profile secrets/message bodies/media
  bytes, and public availability remains blocked until manual smoke passes;
- `whatsapp_web_companion` installs only a safe extractor injection contract at
  this stage: a main-frame-only initialization script guarded to
  `https://web.whatsapp.com`, a same-origin navigation guard and a frozen
  metadata contract. It must not read cookies, Web Storage, IndexedDB, browser
  profile secrets, session material, message bodies or media bytes, and must not
  call `fetch`, XHR, `postMessage` or domain APIs. Runtime health reports this
  as `contract_injected_relay_dispatch_available` with
  `tauri_allowlisted_companion_runtime_bridge_dispatch`; the relay posts
  sanitized metadata as `NewWhatsappWebRuntimeEvent` into
  `/runtime-bridge/runtime-events` and does not attempt typed projection until
  richer WebView payloads exist;
- frontend integration code has a typed `@tauri-apps/api` bridge for those
  shell commands. The bridge calls Tauri `invoke` directly and must not route
  through `ApiClient`, `fetch` or backend/domain HTTP APIs; visible runtime-panel
  controls remain gated on UI evidence/design review, while live smoke remains
  the runtime closure gate;
- native provider crates are represented by a driver descriptor; compile-only or
  smoke-gated descriptors must not flip public live runtime capabilities to
  available;
- native runtime health must expose the verified `wa-rs` public SDK methods,
  including the forwarded-text reemit contract for `forward`, and the missing
  safe write APIs for any smoke-gated unsupported command. Missing SDK support
  for `publish_status`, dialog-state writes, `mark_unread` and join-by-invite
  must remain public-blocked and may only produce structured terminal
  dead-letter evidence until a provider API and live smoke prove the path;
- native `native_md` has an account-scoped runtime actor contract whose command
  channel is the durable provider outbox, whose event sink is Signal Hub raw
  evidence, and whose session material policy is host-vault-only with
  PostgreSQL metadata bindings;
- the actor contract names every provider event family that must enter the Hub:
  auth, runtime lifecycle, sync lifecycle, messages, updates, deletes,
  receipts, reactions, dialogs, participants, presence, call metadata, statuses,
  status views/deletes, media lifecycle, command reconciliation and unsupported
  evidence;
- `native_md` classifies real `wa-rs` provider events into those families before
  any projection or workflow sees them; raw SDK notifications and provider-only
  business events are retained as unsupported runtime evidence, not silently
  dropped;
- classified native events are wrapped in a raw-evidence envelope that carries
  account id, stable provider event id, provider shape, runtime driver, raw
  record kind, raw Signal Hub event kind, accepted event kind and source
  fingerprint seed before any future append;
- the native raw-evidence envelope allows sanitized metadata only and must not
  carry session/token/cookie/raw secrets, message bodies or media bytes;
- native events are also converted into a sanitized inbound DTO before any
  future live producer writes Hub evidence; the DTO may carry provider ids,
  JIDs, timestamps, state flags, sync counters and payload-shape flags, but must
  exclude QR codes, pair codes, raw SDK nodes, protobuf action payloads,
  history-sync payloads, about text, push names, session material, message
  bodies and media bytes;
- the sanitized native DTO fixes its dispatch target to an existing
  `/api/v1/integrations/whatsapp/runtime-bridge/*` endpoint family instead of
  allowing ad-hoc runtime-to-domain calls;
- native runtime health exposes the driver descriptor, readiness blocker,
  durable outbox command channel, Signal Hub raw-evidence sink and host-vault
  session boundary under sanitized `checks.native_md_driver` metadata;
- native runtime health also exposes the `wa-rs` backend manifest: the future
  live driver must use `NativeMdHostVaultBackend`, which already implements
  `Backend` over `SignalStore`, `AppSyncStore`, `ProtocolStore` and
  `DeviceStore`, uses the account-scoped `whatsapp_web_session_key` host-vault
  binding, and does not use SDK SQLite or PostgreSQL secret payload storage;
- native runtime health also exposes the compile-checked client factory wiring:
  `NativeMdWaRsClientFactory::configured_builder` binds the host-vault backend,
  `TokioWebSocketTransportFactory`, `UreqHttpClient`, optional pair-code
  options and a sanitized event-handler DTO path without calling `build()` or
  claiming live readiness;
- native runtime health also exposes the compile-checked live driver lifecycle:
  `NativeMdLiveDriver` builds the configured `wa_rs::bot::Bot`, starts it with
  `Bot::run()`, stops through `Client::disconnect()` and task abort cleanup, and
  routes inbound provider events only as owned sanitized DTOs into the shared
  `WhatsAppRuntimeEventSink` contract;
- `WhatsappRuntimeSignalIngestService` implements that shared sink at the
  application boundary: sanitized native DTOs become append-only raw evidence,
  `signal.raw.whatsapp.*.observed` and accepted Signal Hub events, with
  recursive secret-like metadata redaction and duplicate provider event
  idempotency;
- native runtime health also exposes the smoke-gated runtime manager:
  `checks.native_md_manager` / `checks.runtime.native_manager` report account
  scoping, explicit `native_md_live_smoke_enabled` opt-in, host-vault session
  binding requirement, SDK feature state, running state and the
  `blocked_until_manual_live_smoke` public availability gate;
- QR/pair-code startup is vault-aware: `WhatsAppProviderRuntime::start_qr_link`
  and `start_pair_code_link` receive `SecretReferenceStore` and `HostVault`
  context, and the native smoke manager can create the account-scoped
  `whatsapp_web_session_key` host-vault bootstrap binding before starting the
  feature-gated driver;
- live QR/pair-code artifacts are exposed only through the native manager's
  in-memory, one-time transient start response channel after provider-observed
  `PairingQrCode` / `PairingCode` events; they must not be written to
  PostgreSQL, events, logs or health payloads;
- startup restore reconciliation now attempts eligible `whatsapp_native_md`
  runtime startup from the host-vault `whatsapp_web_session_key` binding through
  `WhatsAppProviderRuntime::start_runtime`, gated by explicit native smoke
  opt-in, and emits only sanitized restore-start status/session events;
- native reconnect policy is account-scoped and tick-driven: provider-observed
  degraded/recovered lifecycle evidence updates manager state, bounded reconnect
  attempts reuse the same vault-bound session, and restart events go through the
  sanitized Signal Hub sink;
- QR/pair-code and provider-command blockers include provider-shape-specific
  native blockers while the live driver is missing or feature-disabled;
- session storage is account-scoped;
- successful authorization stores session material in host vault and binds it with `whatsapp_web_session_key`;
- runtime status/start/health resolve the account-scoped host-vault session
  binding without returning raw session material;
- fixture runtime now maps account/session state into target-style lifecycle
  values such as `link_required`, `linked`, `available`, `revoked` and
  logical `removed`;
- local runtime data path is ignored by Git;
- runtime emits sanitized lifecycle events;
- no message bodies or secrets in lifecycle events.

## Phase P2 — Third-party Rust library spike

Goal: decide whether to use `whatsapp-rust` or a fallback fork instead of writing the protocol ourselves.

Spike target:

```text
crates/whatsapp-native-spike/
```

Test matrix:

| Test | Required result |
|---|---|
| Compile on Rust 1.89 | Pass without changing global toolchain. |
| QR callback | Can surface QR code through runtime API. |
| Pair-code callback | Can surface pair code if provider supports it. |
| Session persistence | Can store under account-scoped local path. |
| Receive text | Emits provider event with stable ids. |
| Receive media metadata | Emits media ref without storing bytes in DB. |
| Receive reaction/delete/edit | Emits lifecycle event or known unsupported marker. |
| Send text dry path | Can execute under explicit manual smoke mode. |
| Reconnect | Emits degraded/recovered status through sanitized runtime lifecycle evidence; manual live smoke still required. |
| Multiple accounts | Isolates sessions. |

Acceptance:

- choose `whatsapp-rust`, `wa-rs`, or reject both;
- if `wa-rs` is selected behind the native boundary, expose
  `whatsapp_native_md_public_availability_blocked` until QR/session/event/
  command flows pass smoke and provider-observed evidence;
- no production dependency before ADR accepted;
- no live account needed in CI;
- source evidence mapping design is proven.

## Phase P3 — Inbound source records and projection

Goal: ingest WhatsApp evidence into canonical Communications.

Source record kinds:

```text
whatsapp_message
whatsapp_message_update
whatsapp_message_delete
whatsapp_reaction
whatsapp_receipt
whatsapp_dialog
whatsapp_participant
whatsapp_media
whatsapp_status
whatsapp_presence
whatsapp_call_metadata
whatsapp_runtime_event
```

Canonical projection targets:

```text
communication_accounts
communication_channels
communication_identities
communication_conversations
communication_conversation_participants
communication_messages
communication_message_versions
communication_message_tombstones
communication_message_reactions
communication_message_refs
communication_attachments
communication_raw_records
communication_raw_payloads
```

Acceptance:

- all inbound events are idempotent;
- source fingerprints are stable;
- every canonical row has evidence/provenance;
- unknown provider events are captured as raw evidence and marked unsupported/degraded, not dropped;
- projection never creates Personas/Tasks/Documents directly.

Current foundation implemented:

- fixture message, dialog, participant, reaction, media metadata and status
  records enter through
  `WhatsAppProviderRuntime`;
- fixture runtime-event records now also enter through
  `WhatsAppProviderRuntime`, are stored as raw Communication evidence,
  generate accepted Signal Hub `signal.accepted.whatsapp.runtime_event`
  events and emit sanitized `whatsapp.runtime.event` realtime updates without
  persisting secret-like runtime metadata in PostgreSQL;
- blocked-safe runtime lifecycle transitions now also materialize canonical
  runtime-event evidence through the same raw/accepted Signal Hub path for
  `session_authorized`, `runtime_start`, `runtime_stop`, `runtime_revoke`,
  `runtime_relink`, `runtime_remove`, `login_qr_start` and
  `login_pair_code_start`, instead of existing only as direct realtime event
  payloads;
- fixture provider-sync chat/history control flows now also materialize
  canonical runtime-event evidence for `sync.started`, `sync.progress` and
  `sync.completed` phases instead of limiting sync lifecycle to ad-hoc realtime
  events;
- fixture runtime-event ingest now also defaults unknown/unsupported provider
  event kinds to degraded/warning markers when the caller omits explicit
  lifecycle/runtime severity fields, so unsupported provider evidence is
  preserved and visibly degraded rather than silently accepted as neutral state;
- fixture records are stored as raw Communication evidence and accepted Signal
  Hub events;
- fixture dialogs project into canonical `communication_channels` and
  `communication_conversations`;
- fixture dialog metadata now carries source-backed unread counts, participant
  counts, avatar/profile-picture metadata and provider labels through
  canonical `communication_conversations.metadata`, and those facts are also
  surfaced by sanitized `whatsapp.dialog.updated` events;
- fixture participants project into `communication_identities` and
  `communication_conversation_participants`;
- provider-neutral `/api/v1/communications/conversations`,
  `/api/v1/communications/conversations/{conversation_id}`,
  `/api/v1/communications/conversations/{conversation_id}/members` and
  `/api/v1/communications/conversations/search` now fall back to canonical
  WhatsApp conversation and participant rows;
- fixture message updates project into canonical
  `communication_message_versions`;
- fixture message deletes project into canonical
  `communication_message_tombstones`;
- fixture message metadata now supports broader structured provider evidence for
  mentions, link previews, polls, location, contact cards, stickers,
  join/leave service updates, system/service messages, ephemeral flags and
  view-once flags through canonical `message_metadata`;
- fixture reply and forward lineage now project into canonical
  `communication_message_refs` when source evidence includes provider reference
  metadata;
- fixture receipts project into canonical
  `communication_messages.delivery_state`;
- provider-neutral Communications lifecycle reads for
  `/api/v1/communications/messages/{message_id}/versions` and
  `/api/v1/communications/messages/{message_id}/tombstones` fall back to
  canonical Communication tables for WhatsApp;
- provider-neutral Communications reads for
  `/api/v1/communications/messages/{message_id}/reply-chain`,
  `/api/v1/communications/messages/{message_id}/forward-chain` and
  `/api/v1/communications/messages/{message_id}/reactions` now fall back to
  canonical WhatsApp refs/reactions;
- provider-neutral `/api/v1/communications/messages/{message_id}/raw-evidence`
  now resolves canonical WhatsApp raw records with redaction;
- fixture reactions project into `communication_message_reactions`;
- fixture media metadata projects into `communication_attachments` with
  `not_scanned` scanner state;
- fixture statuses project as provider-neutral Communication messages marked
  with status metadata.
- fixture status media can already attach through the canonical attachment path
  by targeting the projected provider status id, and fixture replies can target
  projected status messages through canonical reply refs;
- fixture presence records now enter through `WhatsAppProviderRuntime`, emit
  raw and accepted Signal Hub events, and can patch canonical
  `communication_identities.metadata` with observed presence state/last-seen
  facts plus sanitized realtime `whatsapp.presence.changed` events.
- fixture status records can now also materialize source-backed author
  identity evidence into canonical `communication_identities` and status
  message metadata when provider evidence includes explicit author identity
  fields;
- blocked-safe and retried `publish_status` commands now also materialize
  canonical accepted `signal.accepted.whatsapp.runtime_event` evidence for
  `status.publish.requested|failed|started|completed`, while the existing
  provider-observed fixture status path still drives durable command
  reconciliation and canonical status projection;
- projected fixture status messages now also run through the shared
  decision/task candidate refresh path so status evidence can feed Review via
  the existing Communications workflow boundary;
- projected fixture message/status evidence now also reuses the shared
  `message_summary_contract` review path to mirror `new_person`,
  `new_organization` and `knowledge_candidate` items from canonical
  Communication evidence, preserving the event-driven/review-driven boundary
  instead of introducing a direct WhatsApp-to-Personas integration path;
- fixture call-metadata records now enter through `WhatsAppProviderRuntime`,
  emit raw and accepted Signal Hub events, and can upsert the existing calls
  read model with sanitized realtime `whatsapp.call.updated` events.

Still missing for live completion:

- live provider event bridge from an actual WebView/native/business-cloud runtime
  into the accepted WhatsApp runtime-event path; accepted runtime-event
  lifecycle signals now already have a dedicated reconciliation consumer that
  can drive account/session lifecycle state, Signal Hub connection status and
  restorable-session cleanup from the event spine itself;
- live receipt producer coverage from a real runtime into the accepted-event
  path; accepted WhatsApp receipt events now already participate in the shared
  provider-observation reconciliation consumer so later receipt evidence can
  advance durable command delivery-state after message completion has supplied
  a stable `provider_message_id`;
- live reply/forward reconciliation from provider runtime events;
- live dialogs, participants and identity trace reconciliation;
- live runtime mute/unmute, mark-read/unread and full runtime reaction reconciliation;
- live media transfer and blob persistence;
- live status feed handling.
- fixture status lifecycle now covers posted, viewed and deleted observations
  through canonical metadata/tombstone updates and sanitized realtime events.

Current fixture sync foundation implemented:

- `/api/v1/integrations/whatsapp/provider-sync/chats` returns projected
  WhatsApp conversations for fixture/runtime-control flows;
- `/api/v1/integrations/whatsapp/provider-sync/history` returns projected
  WhatsApp message history for a selected conversation;
- both surfaces emit sanitized `whatsapp.sync.started|progress|completed|failed`
  lifecycle events.

## Phase P4 — Dialogs, participants and identity traces

Goal: make WhatsApp conversations useful without corrupting Persona truth.

Deliverables:

- private chat projection;
- group projection;
- community/subgroup projection;
- broadcast/channel/newsletter status policy;
- participant projection;
- phone/wa_id/display-name identity traces;
- Persona candidate workflow.

Acceptance:

- phone traces are evidence only;
- Persona merge is review-driven;
- group roles are source-backed;
- display-name history is preserved;
- unknown contacts go to Radar/Review, not automatic Persona creation.

Current fixture foundation implemented:

- dialog fixture events create source-backed canonical conversation rows with
  `chat_kind`, archive/pin state, community/subgroup linkage, invite-link
  evidence and broadcast/newsletter/community-root flags;
- participant fixture events create source-backed canonical communication
  identities and conversation participants without creating Personas directly;
- phone/wa_id/display-name traces remain evidence only inside Communications;
- participant and status fixture ingest now also create unattached
  compatibility `person_identities` traces for `whatsapp` and `phone` with
  observation links through the existing Persona review boundary, so identity
  handoff remains review-driven instead of auto-merge;
- participant fixture ingest now also records `message_participant` traces for
  group/community member evidence, and message fixture ingest records phone
  traces from contact-card evidence through the same observation-linked
  boundary;
- these compatibility traces now also persist source-backed WhatsApp evidence
  metadata such as push name, business profile, profile photo refs,
  participant role/status and contact-card payload instead of only the raw
  identity string;
- canonical `communication_identities.metadata` now also preserves
  `display_name_history` plus the previous/current observed display names when
  repeated participant or presence evidence changes the visible provider name;
- participant fixture evidence now also carries source-backed `push_name`,
  `business_profile` and `profile_photo_ref` facts through canonical identity
  and participant metadata without creating Personas directly.
- repeated participant fixture evidence can now update a stable canonical
  participant row, preserve previous/current role and status facts in
  participant metadata, and emit sanitized `whatsapp.participant.changed`
  lifecycle events.

## Phase P5 — Media and attachments

Goal: handle media safely.

Deliverables:

- media metadata projection;
- media download command;
- media upload command;
- local blob storage integration;
- scanner state;
- safe preview artifacts;
- media gallery/search read model;
- voice-note playback.

Acceptance:

- no media bytes in PostgreSQL;
- no `clean` scanner verdict without scanner evidence;
- download/upload has progress events;
- media command completion requires provider-observed reconciliation;
- voice transcription remains future/explicit.

Current foundation implemented:

- canonical WhatsApp attachment projection is searchable through the
  provider-neutral `/api/v1/communications/search/media` route;
- provider-neutral `/api/v1/communications/search/messages` now includes
  projected WhatsApp messages;
- blocked-safe media upload/download command endpoints now exist and emit
  sanitized request/fail lifecycle events, and those blocked-safe
  `media.upload.*` / `media.download.*` request-failure phases now also
  materialize canonical accepted `signal.accepted.whatsapp.runtime_event`
  evidence through the shared raw/accepted Signal Hub path;
- the fixture command executor now emits sanitized
  `whatsapp.media.upload.started|progress|completed` and
  `whatsapp.media.download.started|progress|completed` events while completing
  retried media commands through provider-observed evidence, and those
  executor lifecycle phases now also materialize canonical accepted
  `signal.accepted.whatsapp.runtime_event` evidence through the same raw /
  accepted Signal Hub path;
- native live media upload / voice-note upload now reads local blob bytes in the
  application worker, validates size/SHA-256, sends through `wa-rs::Client::upload`
  and records only sanitized provider-submission/progress metadata;
- inbound native media messages now expose hash-only download refs plus the
  `whatsapp_media_download_ref` host-vault purpose while excluding raw
  `media_key`, `direct_path`, `static_url`, URL, caption, filename and bytes;
- live native provider events now materialize raw media download refs to host
  vault before DTO/event redaction;
- live native `download_media` now consumes host-vault
  `whatsapp_media_download_ref` payloads, downloads through
  `wa-rs::Client::download_from_params`, writes bytes to local blob storage and
  emits media lifecycle/runtime-bridge observation evidence without storing raw
  media refs or bytes in PostgreSQL/events/logs/frontend;
- provider-observed upload completion smoke evidence and richer gallery UX are
  still missing.

## Phase P6 — Provider command outbox

Goal: enable sends and provider-side actions safely.

Commands:

```text
send_text
send_media
send_voice_note
reply
forward
edit
delete
react
mark_read
mark_unread
archive
unarchive
mute
unmute
pin
unpin
join_group
leave_group
publish_status
```

Acceptance:

- blocked-safe command endpoints exist for text send, reply, forward, edit and
  delete, plus add/remove reaction, and emit sanitized command-status events
  without message body text;
- command row is durable before execution and mirrored into canonical
  `communication_provider_commands`;
- idempotency key required;
- capability check required;
- destructive commands require confirmation policy;
- audit is redacted;
- manual command list, retry and dead-letter surfaces exist for integration
  runtime state;
- completion requires provider-observed evidence;
- failed commands can retry or dead-letter.

Current foundation implemented:

- blocked-safe text send, reply, forward, edit and delete endpoints;
- blocked-safe voice-note send, status publish, join/leave, mark-read/unread,
  archive/unarchive, mute/unmute and pin/unpin endpoints;
- runtime trait methods for command creation/list/retry/dead-letter;
- idempotent `whatsapp_provider_write_commands` rows;
- canonical `communication_provider_commands` mirror rows with
  `channel_kind = 'whatsapp'`;
- sanitized `whatsapp.command.status_changed` events;
- manual command list, retry and dead-letter endpoints;
- live runtime-bridge command claim now refuses commands without
  `session_restore_available = true`, plus `fixture`, `live_blocked`,
  empty-runtime and unlinked account lifecycle states before moving a command
  into `executing`.

Still missing for live completion:

- fixture background command executor now exists for retried `send_text`,
  `reply`, `forward`, `edit`, `delete`, `send_media`, `download_media`,
  `send_voice_note`, `react`/`unreact`, dialog-state commands,
  `join_group`/`leave_group` and `publish_status`; exact fixture validation
  now also proves provider-observed reconciliation for `send_text`, `reply`,
  `forward`, `send_media`, `download_media`, `send_voice_note` and
  `publish_status`;
- native live execution now covers `send_text`, `reply`, `edit`, `delete`,
  `react`/`unreact`, mark-read, leave-group, `send_media` upload and
  `send_voice_note` upload behind the smoke-gated `WhatsAppProviderRuntime`
  boundary; SDK success remains provider-submission only and completion still
  requires provider-observed evidence;
- native runtime health now also publishes the `wa-rs 0.2.0` command-gap
  manifest: verified SDK calls are named explicitly, including forwarded-text
  reemit for `forward`, while `publish_status`, `archive/unarchive`,
  `mute/unmute`, `pin/unpin`, `mark_unread` and `join_group` remain unsupported
  because no safe public SDK write API was verified. If such commands are
  claimed by the smoke worker, the only allowed outcome is deterministic
  preflight failure before smoke/runtime driver lookup, followed by structured
  terminal dead-letter evidence with no command completion;
- native inbound media download refs are sanitized DTO metadata only; live
  `download_media` consumes host-vault `whatsapp_media_download_ref` refs in
  memory, writes downloaded bytes to local blob storage and completes through
  runtime-bridge media observation evidence, while public availability still
  waits for manual smoke;
- native media upload submissions now persist a sanitized
  `provider_observed_completion_target`, and accepted
  `signal.accepted.whatsapp.media` evidence can complete `send_media` /
  `send_voice_note` when the observed provider message id matches the stored
  `wa-rs` provider request id. Fixture blob-id matching remains only as a
  deterministic fallback;
- accepted WhatsApp observation events now have a dedicated background
  reconciliation consumer for `message`, `reaction`, `media`, `status`,
  `dialog`, `participant`, `message_update` and `message_delete`, which
  reconstructs provider DTOs from raw evidence and drives durable command
  completion through `WhatsAppProviderRuntime`;
- live provider event bridge from a real runtime into that accepted-event path
  across the full command surface;
- full backoff worker policy across all command kinds;
- provider execution adapters;
- direct live provider observation emission into the accepted-event path;
- fixture provider-observed reconciliation for `react`/`unreact`, `edit`,
  `delete`, `archive`/`unarchive`, `pin`/`unpin`, `mute`/`unmute` and
  `mark_read`/`mark_unread` now updates durable command rows and emits
  `whatsapp.command.status_changed` plus `whatsapp.command.reconciled` from
  the event spine;
- command completion/failure events from observed provider state.

## Phase P7 — Realtime and cache patching

Goal: make UI feel alive without turning frontend into a provider-specific data swamp.

Realtime streams:

- runtime state;
- session/link state;
- conversation updates;
- message created/updated/deleted;
- reaction changes;
- media transfer lifecycle;
- command status/reconciliation.

Acceptance:

- Communications data patches Communications caches;
- runtime data patches Integrations caches;
- no broad event includes secrets/raw payload/media bytes;
- frontend can replay from persisted state after reconnect.

Current foundation implemented:

- Communications-facing realtime patching already updates cached WhatsApp
  message lists for `whatsapp.message.*`, `whatsapp.reaction.changed` and
  `whatsapp.receipt.changed`;
- the Communications-facing WhatsApp panel now uses provider-neutral
  `/api/v1/communications/*` routes for projected send/reply/forward/edit/
  delete, plus conversation read/unread, pin/unpin, archive/unarchive and
  mute/unmute, instead of remaining read-only;
- provider-neutral backend handlers now also dispatch WhatsApp conversation
  send and conversation read/unread, pin/unpin, archive/unarchive, mute/unmute
  plus message reply/forward/edit/delete requests into `WhatsAppProviderRuntime`,
  rather than assuming a telegram-only command path behind the same URLs;
- provider-neutral `/api/v1/communications/messages/{message_id}/reactions`
  now also dispatches WhatsApp add/remove flows into the same
  `WhatsAppProviderRuntime` command path, and the WhatsApp Communications
  panel surfaces reaction chips plus a compact add/remove palette over the
  projected message list;
- integration-facing realtime patching now updates cached WhatsApp session
  lists plus account-scoped runtime status caches for
  `whatsapp.runtime.status_changed`, `whatsapp.session.link_state_changed` and
  `whatsapp.runtime.event`;
- Communications-facing realtime patching now also updates cached WhatsApp
  conversation list/detail queries from `whatsapp.dialog.updated`, including
  provider-observed unread, pinned, archived, muted and participant-count
  metadata, instead of relying only on invalidation after reconnect;
- runtime/session/media/command lifecycle events now invalidate the
  integration query families for sessions, runtime status, runtime health and
  account capabilities so the runtime workspace converges after reconnect or
  missed patch detail;
- the realtime bootstrap tests now cover both cache patching and invalidation
  for WhatsApp runtime/session events.

## Phase P8 — Hermes intelligence

Goal: make WhatsApp useful as memory/context, not merely chat display.

Workflows:

```text
communication_to_radar
communication_to_task_candidates
communication_to_obligation_candidates
communication_to_decision_candidates
communication_to_note_candidates
communication_to_document_candidates
communication_to_persona_candidates
communication_to_timeline
communication_to_context_pack
```

Acceptance:

- every AI result has Source, Confidence and Evidence;
- AI creates candidates, not truth;
- Radar catches ambiguous signals;
- Review promotes to target domains;
- Context Packs can include WhatsApp evidence.
- projected WhatsApp message and status evidence now already reuse the shared
  Review candidate path for decision/task/knowledge candidates rather than
  introducing a WhatsApp-specific workflow branch.
- sanitized WhatsApp projection and command/media events now also satisfy the
  generic Timeline replay contract (`subject.entity_id`, `source.kind`,
  `source.source_id`), and exact fixture coverage proves status, participant
  and call events replay through the shared Timeline engine.

## Phase P9 — Business Cloud provider

Goal: add official WhatsApp Business Platform support if needed.

Provider kind:

```text
whatsapp_business_cloud
```

Candidate crates:

- `whatsapp-business-rs`;
- `wacloudapi`;
- plain `reqwest` if SDK quality is insufficient.

Acceptance:

- separate account kind and provider kind;
- setup stores Cloud API access token only in host vault under
  account-scoped `whatsapp_business_cloud_access_token`; webhook app secret and
  verify token use separate host-vault bindings
  `whatsapp_business_cloud_app_secret` and
  `whatsapp_business_cloud_webhook_verify_token`; account config, events, logs
  and frontend payloads carry metadata/binding refs only;
- smoke-gated `business.messages.send_text` provider submission reads the
  access token from host vault only into process memory, uses the durable outbox
  and records sanitized provider-submission metadata without completing the
  command before provider-observed webhook/event reconciliation;
- local runtime-bridge webhook ingest normalizes Business Cloud text messages
  and delivery statuses into existing accepted message/receipt evidence, while
  unsupported webhook entries are preserved as sanitized degraded runtime
  evidence;
- local runtime-bridge webhook verification handles challenge tokens and
  raw-body `X-Hub-Signature-256` HMAC-SHA256 signatures before evidence ingest,
  while keeping Hermes behind ADR-0056 local API auth instead of exposing a
  public unauthenticated backend route;
- protected proxy manifest documents the required external edge behavior for
  Business Cloud webhooks: forward GET challenge query params, forward POST raw
  body byte-for-byte with `X-Hub-Signature-256`, inject `X-Hermes-Secret` only
  on the local Hermes request, and report only account readiness booleans plus
  secret-purpose names without reading host-vault secret values;
- standalone `hermes-whatsapp-business-cloud-edge-proxy` exposes the public
  `/webhooks/whatsapp/business-cloud` path, forwards GET challenge queries and
  exact POST raw bodies/signatures to protected Hermes, injects local
  `X-Hermes-Secret` only on the Hermes request, sanitizes upstream failures,
  does not parse webhook JSON and does not read host-vault secret values;
- the edge proxy is packaged as an opt-in Docker Compose profile under
  `docker/` with a dedicated runtime image target, loopback default bind,
  non-secret `.env.example` placeholders, health check and Makefile
  config/up/stop/logs targets. This packages the local bridge without exposing
  Hermes itself as a public service;
- edge proxy behavior has executable coverage: readiness checks the protected
  manifest without account query state, GET challenge forwarding may append
  optional account scope, POST webhook delivery forwards raw body/signature
  without account query state, and unsigned POST requests are rejected before
  reaching Hermes;
- smoke-gated `send_text` provider failures map HTTP 429 / `Retry-After` into
  structured outbox retry metadata without storing raw provider payloads;
- smoke-gated `send_template`, `send_media` and `send_voice_note` provider
  submissions use the durable outbox and Graph messages/media endpoint shapes;
- Graph `send_text`, `send_template`, `send_media` and `send_voice_note`
  submissions persist a sanitized provider-observed completion target; webhook
  `statuses[]` receipt evidence must match the stored Graph message id/provider
  request id before the durable command can complete;
- media submissions read bytes only from local blob storage in worker memory,
  validate size/SHA-256, then exclude raw bytes, filenames, captions and raw
  provider payloads from result/event metadata;
- WABA/phone-number/webhook semantics documented;
- not used as personal WhatsApp provider;
- official business policy and rate limits represented in capability model.

## Test strategy

### Unit tests

- provider id mapping;
- source fingerprints;
- capability transitions;
- event payload redaction;
- command state machine;
- media metadata mapping;
- phone identity trace mapping.

### Integration tests

Use `testcontainers-rs` and fixture providers only:

```text
Container
  -> Migration
  -> Fixture Account
  -> Fixture Provider Events
  -> Projection
  -> Assertions
  -> Destroy
```

No live WhatsApp account in CI.

### Snapshot tests

Use `insta` for:

- event envelopes;
- command payloads;
- source record payloads;
- API responses;
- frontend state contracts.

### Mock tests

Use `mockall` for provider runtime traits:

- QR timeout;
- reconnect;
- duplicate events;
- provider write failure;
- media download progress;
- reconciliation timeout.

### Manual smoke tests

Live runtime smoke tests are manual and local:

- link account;
- receive message;
- send owner-confirmed text;
- download media;
- add reaction;
- logout/revoke;
- relink;
- verify no secrets in DB/logs/events.

## First implementation slice after docs

The next code slice should not start with live WhatsApp protocol. Start with contracts:

1. Add `backend/src/integrations/whatsapp/runtime/provider_runtime.rs` trait.
2. Add runtime capability DTOs.
3. Add `GET /api/v1/integrations/whatsapp/runtime/status` returning fixture/blocked status.
4. Add `POST /api/v1/integrations/whatsapp/runtime/start` with fixture-safe and live-runtime-aware status resolution, while keeping unsupported live shapes blocked until an explicit provider runtime is selected.
5. Add command outbox schema/docs for WhatsApp provider writes.
6. Add fixture events for reactions, media metadata and status.
7. Project those fixture events into canonical Communications tables.

That gives Hermes the skeleton for full functionality without immediately wrestling the protocol dragon. A rare moment of restraint. Historic, really.
