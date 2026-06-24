# Rust WhatsApp Provider Research

Status: research note.
Date: 2026-06-24.

Goal: identify existing Rust projects that can reduce custom WhatsApp protocol work for Hermes.

This is not a final dependency decision. WhatsApp personal-account automation has policy and account-risk implications. Any unofficial provider must stay behind explicit owner-controlled capability gates and must not become invisible infrastructure.

## Short conclusion

Recommended direction:

```text
Primary experiment: whatsapp-rust
Fallback experiment: wa-rs only if stable-Rust/toolchain issues appear
Do not use: old whatsappweb-rs as foundation
Future official business provider: whatsapp-business-rs or wacloudapi
Reference only: whatsmeow / Baileys
```

Reason:

- `whatsapp-rust` is the most complete native Rust candidate found for WhatsApp Web multi-device style functionality.
- `wa-rs` is a fork with stable Rust support and bug fixes, but appears much younger/smaller.
- `whatsappweb-rs` is old and marked heavily WIP/no releases.
- `whatsapp-business-rs` and `wacloudapi` target the official Meta Business/Cloud API, which is useful later but does not solve personal WhatsApp account ingestion.
- `whatsmeow` and Baileys are mature references, but they are Go/TypeScript, not Rust.

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

- Does it compile on Hermes Rust 1.88 without nightly?
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

- compile on Rust 1.88;
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

- send text;
- reply;
- reaction;
- media download;
- media upload;
- delete if supported.

### Spike 4: media

Test:

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
