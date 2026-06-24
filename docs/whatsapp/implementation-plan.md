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

- `docs/whatsapp/current-audit-2026-06-24.md`;
- `docs/whatsapp/full-functionality-target.md`;
- `docs/whatsapp/rust-provider-research.md`;
- `docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md`;
- update `docs/whatsapp/README.md` with the new document set;
- update `docs/whatsapp/api.md` after route design is accepted.

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
pub trait WhatsappRuntime: Send + Sync {
    fn provider_shape(&self) -> WhatsappProviderShape;
    async fn start(&self, account_id: &str) -> Result<RuntimeStatus, RuntimeError>;
    async fn stop(&self, account_id: &str) -> Result<RuntimeStatus, RuntimeError>;
    async fn link_qr(&self, account_id: &str) -> Result<QrLinkSession, RuntimeError>;
    async fn link_pair_code(&self, account_id: &str, phone: &str) -> Result<PairCodeSession, RuntimeError>;
    async fn health(&self, account_id: &str) -> Result<RuntimeHealth, RuntimeError>;
}
```

Acceptance:

- all live capabilities default to `blocked`;
- session storage is account-scoped;
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
| Compile on Rust 1.88 | Pass without changing global toolchain. |
| QR callback | Can surface QR code through runtime API. |
| Pair-code callback | Can surface pair code if provider supports it. |
| Session persistence | Can store under account-scoped local path. |
| Receive text | Emits provider event with stable ids. |
| Receive media metadata | Emits media ref without storing bytes in DB. |
| Receive reaction/delete/edit | Emits lifecycle event or known unsupported marker. |
| Send text dry path | Can execute under explicit manual smoke mode. |
| Reconnect | Emits degraded/recovered status. |
| Multiple accounts | Isolates sessions. |

Acceptance:

- choose `whatsapp-rust`, `wa-rs`, or reject both;
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

- command row is durable before execution;
- idempotency key required;
- capability check required;
- destructive commands require confirmation policy;
- audit is redacted;
- completion requires provider-observed evidence;
- failed commands can retry or dead-letter.

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
- WABA/phone-number/template/webhook semantics documented;
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
4. Add `POST /api/v1/integrations/whatsapp/runtime/start` that returns blocked until provider selected.
5. Add command outbox schema/docs for WhatsApp provider writes.
6. Add fixture events for reactions, media metadata and status.
7. Project those fixture events into canonical Communications tables.

That gives Hermes the skeleton for full functionality without immediately wrestling the protocol dragon. A rare moment of restraint. Historic, really.
