# ADR-0101 WhatsApp Provider Runtime Selection

Status: Proposed
Date: 2026-06-24

## Context

Hermes needs full WhatsApp functionality plus Hermes-specific memory, Radar, Review, Timeline, Search, AI and workflow features.

The current repository already contains a WhatsApp fixture/runtime foundation:

- provider kind `whatsapp_web`;
- fixture account/session/message ingestion;
- Signal Hub trace path;
- Communications projection;
- frontend runtime panel;
- ADR-0051 for a visible WhatsApp Web companion boundary;
- ADR-0097 stating that channels are integrations and Communications owns business state.

Full WhatsApp support requires provider capabilities beyond the current fixture foundation:

- QR/pair-code linking;
- persistent sessions;
- live inbound messages;
- media transfer;
- reactions;
- statuses;
- group/community metadata;
- provider commands;
- reconciliation;
- runtime health.

Writing all of this from scratch would mean reimplementing a large portion of WhatsApp Web/multi-device protocol behavior, session storage, E2E encryption flow, media encryption/decryption and provider event handling. That is a charming way to turn a personal OS into a protocol archaeology dig.

Research found these Rust candidates:

- `whatsapp-rust`, a broad async Rust WhatsApp Web API implementation;
- `wa-rs`, a fork of `whatsapp-rust` claiming stable Rust support;
- `whatsappweb-rs`, older and heavily WIP;
- `whatsapp-business-rs` and `wacloudapi`, Rust SDKs for official Meta Business/Cloud API.

Official Business Cloud API remains a separate business provider shape. It does not replace personal WhatsApp account support.

## Decision

Hermes will use a **multi-provider runtime boundary** for WhatsApp.

Provider shapes:

```text
whatsapp_web_companion
whatsapp_native_md
whatsapp_business_cloud
```

Compatibility:

- existing `whatsapp_web` may remain as compatibility provider kind during migration;
- new provider shape naming must be explicit in runtime capability metadata;
- no provider shape may bypass Communications ownership.

### Primary experimental native provider

Use `whatsapp-rust` as the first Rust native multi-device provider candidate.

Conditions:

- feature-flagged;
- disabled by default;
- owner-visible opt-in;
- isolated under `backend/src/integrations/whatsapp/runtime/native_md`;
- no direct dependency from domains/workflows/engines/frontend domains;
- no session secrets in PostgreSQL/events/logs/frontend;
- no live runtime requirement in CI;
- no provider writes without durable command outbox and capability checks.

### Fallback native provider

Evaluate `wa-rs` only if `whatsapp-rust` fails the Rust 1.88/toolchain/stability spike.

### Rejected foundation

Do not use `whatsappweb-rs` as the main provider foundation because it is old, heavily WIP and has no release posture suitable for Hermes.

### Future official business provider

For official business accounts, evaluate:

- `whatsapp-business-rs`;
- `wacloudapi`;
- direct `reqwest` implementation if SDKs do not fit.

This provider must use:

```text
provider_kind = whatsapp_business_cloud
```

and must follow WABA/template/webhook/business policy semantics.

## Boundary

Third-party provider libraries live only behind:

```text
backend/src/integrations/whatsapp/runtime/*
```

They may produce:

- runtime lifecycle events;
- provider observations;
- provider command execution results;
- media transfer results;
- sanitized health diagnostics.

They may not produce or mutate:

- Tasks;
- Personas;
- Organizations;
- Documents;
- Notes;
- Knowledge;
- Decisions;
- Obligations;
- Memory;
- Search truth;
- AI truth.

Inbound flow:

```text
External provider
  -> WhatsApp runtime
  -> WhatsApp adapter
  -> observation/signal event
  -> Communications projection
  -> Radar/Review/Timeline/Search/Engines
```

Outbound flow:

```text
UI/App
  -> Communications command
  -> communication.outbox.queued
  -> communication.provider_command.requested
  -> WhatsApp integration command consumer
  -> provider execution
  -> provider-observed evidence
  -> communication.provider_command.completed/failed
```

## Capability defaults

All live capabilities default to blocked:

```text
whatsapp.native_md.runtime = blocked
whatsapp.native_md.send = blocked
whatsapp.native_md.media_download = blocked
whatsapp.native_md.media_upload = blocked
whatsapp.native_md.reaction = blocked
whatsapp.native_md.status_publish = blocked
```

They can become available only after:

- runtime spike passes;
- owner opt-in UX exists;
- capability policy exists;
- outbox/reconciliation exists for writes;
- redaction tests pass;
- manual smoke checklist exists.

## Consequences

Positive:

- Hermes avoids writing the WhatsApp protocol from scratch.
- Provider instability is isolated.
- The existing Communications/Radar/Review architecture remains intact.
- Business Cloud support can be added later without corrupting personal provider assumptions.

Negative:

- Unofficial provider risk remains.
- Protocol drift may break live runtime.
- Account suspension risk must be surfaced to the owner.
- Runtime testing requires manual live smoke tests outside CI.
- The adapter boundary becomes critical infrastructure.

## Risk handling

- Explicit owner-facing warnings for unofficial runtime.
- No bulk messaging, auto-messaging or hidden automation.
- No hidden scraping.
- Feature flag and capability gate.
- Provider writes require confirmation/audit/reconciliation.
- Session secrets stay in host/local runtime storage only.
- Live failures degrade capabilities instead of corrupting canonical state.
- AI output remains candidates with Source, Confidence and Evidence.

## Acceptance criteria

This ADR can move from Proposed to Accepted only when:

1. `whatsapp-rust` compiles on Hermes Rust toolchain or fallback is selected.
2. QR/pair-code session lifecycle can be surfaced without leaking secrets.
3. Inbound message events can be mapped to source-backed observations.
4. Media metadata can be mapped without storing bytes in PostgreSQL.
5. Provider write execution can be routed through the outbox contract.
6. Multiple account isolation is demonstrated.
7. Redaction tests exist.
8. Manual live smoke-test checklist exists.
9. Architecture guard prevents provider library imports outside integration runtime.
10. Documentation clearly distinguishes personal unofficial runtime from official Business Cloud API.

## External references

- `whatsapp-rust`: <https://github.com/oxidezap/whatsapp-rust>
- `wa-rs`: <https://github.com/homun-app/wa-rs>
- `whatsappweb-rs`: <https://github.com/wiomoc/whatsappweb-rs>
- `whatsapp-business-rs`: <https://docs.rs/whatsapp-business-rs>
- `wacloudapi`: <https://docs.rs/wacloudapi>
- WhatsApp Business Developer Hub: <https://whatsappbusiness.com/developers/developer-hub/>
- WhatsApp Terms of Service: <https://www.whatsapp.com/legal/terms-of-service>
