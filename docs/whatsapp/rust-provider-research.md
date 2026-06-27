# Rust WhatsApp Provider Research

Status: research note.
Date: 2026-06-26.

Goal: identify existing Rust projects that can reduce custom WhatsApp protocol work for Hermes.

This is not a final dependency decision. WhatsApp personal-account automation has policy and account-risk implications. Any unofficial provider must stay behind explicit owner-controlled capability gates and must not become invisible infrastructure.

## Short conclusion

Recommended direction:

```text
Primary experiment result: whatsapp-rust blocked on stable Rust compile spike
Selected compile-boundary fallback: wa-rs
Do not use: old whatsappweb-rs as foundation
Future official business provider: whatsapp-business-rs or wacloudapi
Reference only: whatsmeow / Baileys
```

Reason:

- `whatsapp-rust` is the most complete native Rust candidate found for WhatsApp Web multi-device style functionality, but `0.6.0` does not pass the current stable compile spike.
- `wa-rs` is a fork with stable Rust support and bug fixes; `0.2.0` passes a compile-only spike with SDK SQLite storage disabled.
- `whatsappweb-rs` is old and marked heavily WIP/no releases.
- `whatsapp-business-rs` and `wacloudapi` target the official Meta Business/Cloud API, which is useful later but does not solve personal WhatsApp account ingestion.
- `whatsmeow` and Baileys are mature references, but they are Go/TypeScript, not Rust.

## Spike result, 2026-06-26

Local toolchain: `rustc 1.93.1`; Hermes backend now declares
`rust-version = "1.89"` for the native WhatsApp runtime boundary.

Compile-only results:

| Candidate | Feature set | Result | Notes |
|---|---|---|---|
| `whatsapp-rust 0.6.0` | default | Failed | `wacore-binary` enables `portable_simd`, which is not available on stable. |
| `whatsapp-rust 0.6.0` | `default-features = false`, `sqlite-storage`, `tokio-transport`, `tokio-runtime`, `tokio-native`, `ureq-client`, `signal` | Failed | `wacore 0.6.0` uses experimental `if let` guards on stable Rust. |
| `wa-rs 0.2.0` | default | Passed | Useful proof of crate health, but default pulls SDK SQLite storage. |
| `wa-rs 0.2.0` | `default-features = false`, `tokio-native`, `tokio-transport`, `ureq-client` | Passed | Selected Hermes compile boundary; avoids SDK SQLite storage so session material can remain behind Hermes host-vault/runtime storage design. |
| `wa-rs 0.2.0` | `default-features = false`, `tokio-native`, `tokio-transport`, `ureq-client` on Rust 1.88 | Failed | Transitive `tokio-websockets 0.13.3` requires Rust 1.89. |
| `wa-rs 0.2.0` | `default-features = false`, `tokio-native`, `tokio-transport`, `ureq-client` on Rust 1.89 | Passed | Validated with `cargo +1.89.0 check --manifest-path backend/Cargo.toml --features whatsapp-native-md-runtime --lib`. |

Current implementation implication:

- `backend/Cargo.toml` wires `whatsapp-native-md-runtime` to optional `wa-rs`.
- The dependency is disabled by default and compiled only under the native
  runtime feature.
- Backend, Tauri and Docker development toolchain declarations now use Rust
  1.89 so the selected native transport dependency is not hidden behind a newer
  developer machine toolchain.
- The native compile feature intentionally does not make provider capabilities
  publicly available. Runtime availability stays blocked until provider-observed
  live smoke evidence flows through the Signal Hub contract.
- `backend/src/integrations/whatsapp/runtime/native_md.rs` contains a
  smoke-gated driver descriptor for the selected SDK boundary. Under the feature
  it reports `wa-rs` as present but blocked by
  `whatsapp_native_md_public_availability_blocked`; without the feature it
  remains `whatsapp_native_md_runtime_feature_disabled`.
- The same module now defines an account-scoped runtime actor contract. It binds
  to real `wa-rs` API types (`Bot`, `BotBuilder`, provider `Event`, storage
  `Backend`, `TransportFactory`, `HttpClient`, `Device`, `MessageInfo`,
  `PairCodeOptions`) while keeping public capability gated by smoke evidence.
  Commands are constrained to Hermes durable provider outbox claims, events to
  Signal Hub raw evidence, and provider session material to host-vault metadata
  bindings.
- `wa-rs` requires a full backend implementation before live startup: the
  backend must implement `SignalStore`, `AppSyncStore`, `ProtocolStore` and
  `DeviceStore`. `native_md` now provides `NativeMdHostVaultBackend` for those
  store families and persists their secret material as an encrypted,
  account-scoped host-vault snapshot under the `whatsapp_web_session_key`
  binding, with SDK SQLite disabled and PostgreSQL secret payloads forbidden.
- The native adapter now also has a compile-checked client factory for the real
  `wa-rs` builder surface. `NativeMdWaRsClientFactory::configured_builder`
  wires `NativeMdHostVaultBackend`, `TokioWebSocketTransportFactory`,
  `UreqHttpClient`, optional `PairCodeOptions` and a sanitized event-handler
  DTO path into `wa_rs::bot::BotBuilder`. The factory deliberately returns the
  builder and does not call `build()` in health/compile paths.
- `NativeMdLiveDriver` now compile-checks the live lifecycle surface around
  that builder: `build().await`, `Bot::run().await`, `Client::disconnect().await`
  and task abort cleanup. Its event path accepts only owned sanitized DTOs
  through the shared `WhatsAppRuntimeEventSink`, preserving the
  runtime-to-event-spine boundary.
- `WhatsappRuntimeSignalIngestService` is now the first real writer behind that
  sink contract. It persists sanitized native DTOs as append-only raw
  communication evidence and dispatches them through Signal Hub accepted events
  without importing the native provider SDK outside the adapter boundary.
- `native_md` now has a smoke-gated account manager behind
  `WhatsAppProviderRuntime` lifecycle hooks. It requires explicit
  `native_md_live_smoke_enabled` account config plus a restored
  `whatsapp_web_session_key` host-vault binding before starting the
  feature-gated driver, and reports manager state in runtime health metadata.
- QR/pair-code startup is now vault-aware through `WhatsAppProviderRuntime`.
  The native manager can create the `whatsapp_web_session_key` host-vault
  bootstrap snapshot and account binding before starting the feature-gated
  driver.
- QR/pair-code artifacts are now captured from provider-observed
  `PairingQrCode` / `PairingCode` events into a one-time in-memory start
  response channel. Sanitized runtime DTOs and health payloads still redact the
  raw artifacts; PostgreSQL, Signal Hub events and logs do not receive them.
- Startup restore reconciliation now attempts eligible native runtime startup
  from the account-scoped host-vault `whatsapp_web_session_key` binding through
  `WhatsAppProviderRuntime::start_runtime`. The attempt remains smoke-gated and
  produces only sanitized status/session events.
- Native reconnect policy is now account-scoped and tick-driven. Provider
  connection failures schedule bounded reconnect from the same vault-bound
  session, provider-observed `Connected` emits recovered lifecycle evidence,
  and manager restart attempts emit sanitized lifecycle events through the same
  `WhatsAppRuntimeEventSink`.
- `native_md` also has a feature-gated `wa-rs` event classifier. It maps real
  `wa-rs::types::events::Event` variants to Hermes raw record kinds and
  accepted Signal Hub event families, including protobuf inspection for message
  reactions, media, calls, edits and deletes. Unknown/raw provider notifications
  stay as unsupported runtime evidence rather than being discarded.
- Classified native events now pass through a `NativeMdRawEvidenceEnvelope`
  contract that records provider shape, runtime driver, raw record kind, raw
  Signal Hub event kind, accepted event kind and stable source-fingerprint seed.
  The envelope is sanitized-metadata-only and explicitly excludes session
  material, tokens, cookies, raw secrets, message bodies and media bytes.
- A compile-checked `NativeMdSanitizedProviderEventDto` now sits after
  classification/envelope construction for real `wa-rs::types::events::Event`
  values. It keeps metadata needed for idempotency and later projection
  routing, including ids, JIDs, timestamps, presence/receipt state, sync counts
  and payload-shape flags, while excluding QR codes, pair codes, raw SDK nodes,
  protobuf action payloads, history-sync payloads, about text, push names,
  session material, message bodies and media bytes. The DTO also carries a
  compile-checked dispatch target for the existing runtime-bridge endpoint
  family, so the future native actor has a fixed event-spine path instead of an
  ad-hoc direct domain call.
- Native session restore is still required to use the account-scoped
  `whatsapp_web_session_key` host-vault binding. The four `wa-rs` store
  families, concrete builder wiring, live driver lifecycle and smoke-gated
  account manager, vault-aware link startup, transient QR/pair-code response
  channel, startup restore attempt, reconnect policy and verified-subset command
  execution boundary now compile against the selected SDK; manual smoke plus
  media/status/archive/mute/pin/join/unread/forward command coverage are still
  pending.
- Runtime health now reports the verified SDK submission subset and unsupported
  live command set explicitly, so SDK presence cannot be misread as full command
  availability.

## Candidate comparison

| Project | Language | Provider type | Strengths | Risks | Hermes recommendation |
|---|---|---|---|---|---|
| `oxidezap/whatsapp-rust` / `whatsapp-rust` crate | Rust | Unofficial WhatsApp Web/native multi-device protocol | Broad feature set: auth, E2E messaging, media, groups, communities, status, contacts, presence, chat actions, privacy; modular storage/transport/runtime | Unofficial; ToS/account risk; protocol drift; needs adapter isolation | Use as first experimental native provider behind feature flag and ADR. |
| `homun-app/wa-rs` / `wa-rs` crate | Rust | Fork of `whatsapp-rust` | Stable Rust support claim, QR/pair-code, persistent sessions, messaging/media/groups/presence; MIT | Very small history compared with upstream; fork divergence | Evaluate if upstream requires nightly or breaks current toolchain. |
| `wiomoc/whatsappweb-rs` / `whatsappweb` crate | Rust | Older WhatsApp Web reverse-engineered client | Some text/media/group/contact features | Marked heavily WIP, no releases, old architecture | Do not use as foundation. Reference only if some protocol detail is useful. |
| `veecore/whatsapp-business-rs` | Rust | Official Meta WhatsApp Business Platform SDK | Type-safe Business Platform SDK: messages, webhooks, WABA, catalogs, onboarding flows | Business-only; does not support personal account history; policy/template semantics differ | Use only for future `whatsapp_business_cloud`. |
| `wacloudapi` | Rust | Official Meta WhatsApp Cloud API SDK | Type-safe async SDK; messages, media, templates, phone numbers, products, flows, analytics, QR, webhooks | v0.1.0; business-only; not a personal local-first provider | Evaluate for future official cloud provider. |
| `tulir/whatsmeow` | Go | Unofficial WhatsApp Web multi-device library | Mature reference implementation with many core features | Go dependency/sidecar or rewrite needed; MPL-2.0 license | Reference architecture/protocol behavior only. Avoid Go sidecar unless Rust path fails. |
| `WhiskeySockets/Baileys` | TypeScript | Unofficial socket-based WhatsApp Web library | Very mature ecosystem reference | Node sidecar; JS runtime; protocol drift; license/ops overhead | Reference only, not target for Hermes Rust backend. |

## Recommended provider architecture

Do not embed a third-party WhatsApp library directly into domain code.

Use a strict adapter boundary:

```text
whatsapp-rust / wa-rs
  -> integrations/whatsapp/runtime/native_md
  -> integrations/whatsapp/adapter
  -> platform/contracts/events
  -> platform/events or observations
  -> Communications consumer/projection
```

The third-party library must be treated as volatile provider/runtime code.

It may own:

- socket/protocol connection;
- QR/pair-code pairing;
- local session database;
- encryption/decryption runtime details;
- provider-side send execution;
- raw provider event stream.

It must not own:

- canonical messages;
- tasks;
- notes;
- decisions;
- obligations;
- personas;
- relationship truth;
- memory;
- search truth;
- AI truth.

## Source notes

### `whatsapp-rust`

Research summary:

- GitHub describes it as a high-performance async Rust library for the WhatsApp Web API inspired by whatsmeow and Baileys.
- The README lists authentication, E2E messaging, media, groups/communities, newsletters, status posts, contacts, presence, chat actions, profile and privacy functionality.
- It exposes modular storage, transport, HTTP client and async runtime traits, with SQLite/Tokio defaults.
- The README explicitly warns that it is unofficial and that custom WhatsApp clients may violate Meta terms and risk account suspension.

Potential Hermes fit:

```text
provider_kind = whatsapp_native_md
runtime = integrations/whatsapp/runtime/native_md
storage = account-scoped local provider session store
Hermes projection = event/observation output only
```

Open questions:

- Does it compile on Hermes Rust 1.89 without nightly?
- Can we disable or replace default SQLite storage with Hermes-controlled local runtime storage?
- Can event callbacks provide enough raw/provider identifiers for idempotent projection?
- Can media APIs expose bytes/metadata without leaking secrets into logs?
- Can command reconciliation be source-observed rather than optimistic?
- Can the library run multiple accounts safely in one process?
- How does it handle protocol drift and reconnect/backoff?

### `wa-rs`

Research summary:

- Described as a fork of `whatsapp-rust` with stable Rust support and bug fixes.
- README lists QR pairing, pair-code linking, persistent sessions, E2E messaging, editing/reactions, receipts, media, contacts/groups, presence and modular architecture.

Potential Hermes fit:

- Use only if `whatsapp-rust` has unstable/nightly or portability issues.
- Run the same spike tests as `whatsapp-rust`.

Open questions:

- Fork maintenance velocity.
- API compatibility with upstream.
- Security/bug-fix strategy.
- Whether stable Rust fixes have been merged upstream.

### `whatsappweb-rs`

Research summary:

- Older Rust WhatsApp Web client.
- README says “Heavily WIP”.
- It lists send/receive text/image/audio, groups, user info, contacts/chats, acknowledgements, app state changes, relogin, older message queries and reconnect.
- It has no published GitHub releases.

Hermes decision:

```text
Do not build Hermes on this unless every newer option fails.
```

### `whatsapp-business-rs`

Research summary:

- Type-safe async Rust SDK for the Meta WhatsApp Business Platform.
- Features include message management, webhook server, app management, WABA management and catalog management.

Hermes fit:

```text
provider_kind = whatsapp_business_cloud
scope = future official business accounts only
```

This does not solve personal WhatsApp local-first memory, but it may become useful for owner-controlled business inboxes.

### `wacloudapi`

Research summary:

- Rust SDK for the WhatsApp Cloud API hosted by Meta.
- Docs list messages, media, templates, phone numbers, products, flows, analytics, QR codes, block users, WABA management and webhooks.

Hermes fit:

- Future `whatsapp_business_cloud` provider candidate.
- Compare against `whatsapp-business-rs` for coverage, maintenance and type model.

### `whatsmeow`

Research summary:

- Go library for the WhatsApp Web multi-device API.
- README says most core features are present: sending text/media to private chats and groups, receiving all messages, group management, invite links, typing notifications, delivery/read receipts, app state, retry receipts and experimental status messages. Calls are listed as not implemented.

Hermes fit:

- Use as reference implementation.
- Avoid Go sidecar unless Rust native experiments fail.

## Recommended spike plan

### Spike 1: compile and lifecycle

Create a throwaway local crate outside production code:

```text
crates/whatsapp-native-spike/
```

Test:

- compile on Rust 1.89;
- QR/pair-code callback can be surfaced;
- persistent local session can be stored under ignored data path;
- session state can be wiped/relinked;
- reconnect/backoff events can be observed;
- multiple account instances can be isolated.

### Spike 2: inbound evidence

Test:

- receive text message;
- receive media metadata;
- receive reaction/update/delete;
- receive group event;
- map every event into Hermes raw observation DTO;
- generate stable source fingerprint;
- replay without duplication.

### Spike 3: provider writes

Test command path only in fixture/manual mode first:

```text
communication.provider_command.requested
  -> native runtime execute
  -> provider observed confirmation
  -> communication.provider_command.completed
```

Commands:

- send text, reply, edit, delete/revoke;
- react/unreact;
- mark-read when provider message ids are present;
- leave group;
- media upload / voice-note upload through local blob bytes and `wa-rs::Client::upload`;
- inbound media download refs are now extracted only as sanitized hashes and
  `whatsapp_media_download_ref` host-vault purpose metadata, while raw provider
  refs are materialized to host vault before event redaction;
- live media download now has a smoke-gated `wa-rs::Client::download_from_params`
  path that consumes host-vault `whatsapp_media_download_ref` payloads and writes
  bytes only to local blob storage; forward, status, archive/mute/pin/join/unread
  remain unverified.

### Spike 4: media

Current compile-checked upload path:

- application worker reads local blob bytes and verifies stored SHA-256/size;
- runtime receives bytes only through a redacted in-memory field excluded from serialization;
- `wa-rs::Client::upload` builds provider media refs for image/video/audio/document messages;
- inbound image/video/audio/document/sticker protobuf payloads produce
  hash-only download refs; raw `media_key`, `direct_path`, `static_url`, URL,
  caption, filename and bytes are excluded from DTOs/events/log-like payloads;
- raw provider media refs are stored only in host vault under deterministic
  `whatsapp_media_download_ref` secret refs;
- actual media download still requires manual live smoke before public
  availability can be opened.
- SDK success records sanitized provider-submission metadata only;
- command completion still requires provider-observed evidence.

Remaining media test:

- download image/document/audio;
- store bytes in local blob path;
- compute hash;
- produce scanner state `not_scanned`;
- never put bytes into Postgres/event payload;
- preview only through safe route.

### Spike 5: failure and risk

Test:

- expired session;
- QR timeout;
- provider disconnect;
- message decrypt failure;
- rate-limited or blocked write;
- duplicate event;
- unknown message kind;
- invalid media payload;
- command reconciliation timeout.

## Dependency policy

Production adoption requires:

1. ADR accepted.
2. License review.
3. Security review.
4. Minimal wrapper API in `integrations/whatsapp/runtime/native_md`.
5. No direct dependency from domains/workflows/engines to the provider library.
6. Feature flag:

```toml
[features]
whatsapp-native-md = []
```

7. Runtime capability defaults:

```text
native_md_runtime = blocked
native_md_send = blocked
native_md_media = blocked
```

8. Manual owner action to enable.
9. CI fixture tests only.
10. Manual live smoke tests documented separately.

## Risk matrix

| Risk | Impact | Mitigation |
|---|---|---|
| WhatsApp protocol drift | Runtime breakage | Adapter isolation, capability degradation, fixtures, smoke tests. |
| Account suspension / ToS risk | User account loss | Explicit warnings, owner-controlled opt-in, no bulk/auto messaging, business cloud for official use. |
| Secret leakage | Severe privacy/security breach | Host vault/local runtime store, redaction tests, no secrets in DB/events/logs/frontend. |
| Data overcollection | Privacy violation | Source-backed, local-first, minimal payloads, export/delete policy. |
| Hidden automation | Product/legal risk | Visible runtime, capability confirmations, audit, no hidden scraping. |
| AI hallucinated memory | Memory corruption | AI candidates only, evidence/confidence required, Radar/Review promotion. |
| Library abandonment | Maintenance risk | Adapter boundary, fallback fork, reference implementations. |

## Final recommendation

Adopt this decision path:

```text
1. Keep existing `whatsapp_web` fixture foundation.
2. Add ADR for `whatsapp_native_md` experimental provider.
3. Spike `whatsapp-rust` first.
4. Evaluate `wa-rs` only if toolchain/stability requires it.
5. Keep `whatsapp_business_cloud` separate for official business accounts.
6. Never let any provider library cross into Hermes domains.
```

This gives Hermes a realistic path to full WhatsApp functionality without spending months reimplementing Signal protocol, media encryption, WhatsApp binary framing and session management by hand. One small mercy in the software swamp.

## External references checked

- `whatsapp-rust`: <https://github.com/oxidezap/whatsapp-rust>
- `whatsapp-rust` crate: <https://crates.io/crates/whatsapp-rust>
- `wa-rs`: <https://github.com/homun-app/wa-rs>
- `wa-rs` crate: <https://crates.io/crates/wa-rs>
- `whatsappweb-rs`: <https://github.com/wiomoc/whatsappweb-rs>
- `whatsmeow`: <https://github.com/tulir/whatsmeow>
- `whatsapp-business-rs`: <https://docs.rs/whatsapp-business-rs>
- `wacloudapi`: <https://docs.rs/wacloudapi>
- WhatsApp Business Developer Hub: <https://whatsappbusiness.com/developers/developer-hub/>
- WhatsApp Terms of Service: <https://www.whatsapp.com/legal/terms-of-service>
