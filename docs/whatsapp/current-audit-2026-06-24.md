# WhatsApp Current Audit — 2026-06-24

Status: current repository audit based on `hermes-os-main (3).zip`.

This document records what exists now before starting the full WhatsApp target implementation. It intentionally separates:

- implemented fixture/runtime foundation;
- provider-neutral Communications foundation already present;
- missing production WhatsApp functionality;
- architectural risks that must not be bypassed.

## Executive summary

WhatsApp is already present as a **provider/runtime foundation**, not as a full channel implementation.

The current repository contains:

- WhatsApp documentation under `docs/whatsapp/`;
- ADR-0051 for the WhatsApp Web companion boundary;
- ADR-0097 for channel-to-integration ownership;
- backend integration skeleton under `backend/src/integrations/whatsapp/`;
- runtime/setup routes under `/api/v1/integrations/whatsapp/*`;
- provider-neutral communication read route support through `/api/v1/communications/messages`;
- PostgreSQL migrations for `whatsapp_web` provider kind, session state, observation kind and canonical Communication tables;
- frontend runtime/setup panel under `frontend/src/integrations/whatsapp/`;
- tests proving fixture ingestion can produce raw signal events, accepted signal events, canonical `communication.message.recorded` traces and downstream candidate extraction.

The repository does **not** yet contain live WhatsApp sync, QR/pair-code login, native protocol runtime, WebView runtime, provider-write outbox, media transfer, reactions, statuses, group/community lifecycle, identity trace projection, message lifecycle commands or business cloud API support.

## Current architecture posture

Current target posture is still correct:

```text
WhatsApp provider/runtime
  -> Integration boundary
  -> Provider observation / signal
  -> Communications projection
  -> Timeline / Search / Radar / Review / shared engines
  -> target domains through workflows only
```

WhatsApp must not become:

- a backend domain;
- a frontend product domain;
- a Persona implementation;
- a Task implementation;
- a Knowledge implementation;
- an AI source of truth;
- a hidden automation/scraping daemon.

## Existing repository assets

### Documentation

| File | Current role |
|---|---|
| `docs/whatsapp/README.md` | Channel framing and principles. |
| `docs/whatsapp/architecture.md` | Target architecture and trace flow. |
| `docs/whatsapp/api.md` | Runtime/setup API foundation. |
| `docs/whatsapp/modules.md` | Target module inventory. |
| `docs/whatsapp/status.md` | Earlier zero-based production audit. |
| `docs/whatsapp/gap-analysis.md` | Missing capability matrix. |
| `docs/whatsapp/blockers.md` | Architectural blockers. |
| `docs/adr/ADR-0051-v5-whatsapp-web-companion-boundary.md` | WhatsApp Web companion boundary. |
| `docs/adr/ADR-0097-communications-channel-domains-to-integrations.md` | Channels are integrations, Communications owns domain state. |

### Backend source

| Path | Current role |
|---|---|
| `backend/src/integrations/whatsapp/mod.rs` | WhatsApp integration module root. |
| `backend/src/integrations/whatsapp/client.rs` | Public re-exports for the current client/store foundation. |
| `backend/src/integrations/whatsapp/client/models.rs` | Fixture account, session and message DTOs. |
| `backend/src/integrations/whatsapp/client/store.rs` | Store constructor and dependencies. |
| `backend/src/integrations/whatsapp/client/store/accounts.rs` | Fixture account setup. |
| `backend/src/integrations/whatsapp/client/store/sessions.rs` | Session upsert/list and session observations. |
| `backend/src/integrations/whatsapp/client/store/ingestion.rs` | Fixture message ingestion and projection path. |
| `backend/src/integrations/whatsapp/client/store/queries.rs` | Recent projected WhatsApp message query. |
| `backend/src/app/provider_runtime_handlers/whatsapp.rs` | Runtime/setup handlers. |
| `backend/src/app/api_support/messaging_integrations.rs` | WhatsApp capability response and DTO wrappers. |
| `backend/src/app/router/routes/messaging.rs` | Integration routes for WhatsApp fixture/setup/session operations. |

### Frontend source

| Path | Current role |
|---|---|
| `frontend/src/integrations/whatsapp/api/whatsapp.ts` | Typed runtime/setup client. |
| `frontend/src/integrations/whatsapp/queries/useWhatsappQuery.ts` | Runtime/session query hooks. |
| `frontend/src/integrations/whatsapp/stores/whatsapp.ts` | Integration runtime UI state. |
| `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue` | Fixture/manual runtime panel. |
| `frontend/src/integrations/whatsapp/components/*` | Runtime rail, session list and status messages. |
| `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue` | Communications-facing provider surface. |
| `frontend/src/domains/communications/api/whatsappBusinessApi.ts` | Legacy/current business-facing WhatsApp helper, needs alignment with provider-neutral API rules. |

### Migrations

| Migration | Current role |
|---|---|
| `0021_create_v5_whatsapp_web_foundation.sql` | Adds `whatsapp_web`, `whatsapp_web_session_key`, `whatsapp_web_sessions`. |
| `0125_add_whatsapp_session_observation_kind.sql` | Adds `WHATSAPP_WEB_SESSION` observation kind. |
| `0149_create_canonical_communication_tables.sql` | Adds canonical Communication table family. |

## Existing API surface

Implemented foundation routes:

| Method | Path | Status |
|---|---|---|
| `GET` | `/api/v1/integrations/whatsapp/capabilities` | Implemented foundation. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/accounts` | Implemented fixture-only setup. |
| `GET` | `/api/v1/integrations/whatsapp/sessions` | Implemented session list. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/messages` | Implemented fixture-only message ingest. |
| `GET` | `/api/v1/communications/messages?channel_kind=whatsapp_web` | Provider-neutral read path through Communications. |

Not implemented yet:

- `/api/v1/integrations/whatsapp/runtime/start`;
- `/api/v1/integrations/whatsapp/runtime/stop`;
- `/api/v1/integrations/whatsapp/login/qr/start`;
- `/api/v1/integrations/whatsapp/login/pair-code/start`;
- `/api/v1/integrations/whatsapp/provider-sync/*`;
- `/api/v1/integrations/whatsapp/provider-commands/*`;
- WhatsApp media download/upload endpoints;
- WhatsApp status read/publish endpoints;
- WhatsApp group/community/provider-management endpoints;
- WhatsApp command outbox/retry/dead-letter endpoints.

## Current capability status

| Capability area | Current status | Notes |
|---|---|---|
| Channel framing | Implemented in docs | Correctly treats WhatsApp as integration/channel. |
| Provider kind `whatsapp_web` | Implemented | Present in migration and provider kind model/tests. |
| Secret purpose `whatsapp_web_session_key` | Implemented foundation | Validated in tests; live host-vault session storage still missing. |
| Fixture account setup | Implemented | Local/dev foundation only. |
| Session metadata table | Implemented | `whatsapp_web_sessions`; no live WebView/native session yet. |
| Capability matrix route | Implemented foundation | Reports fixture/manual available and live runtime blocked. |
| Fixture message ingest | Implemented foundation | Creates raw evidence/projection path. |
| Provider-neutral message read | Implemented foundation | Via `/api/v1/communications/messages`. |
| Signal Hub connection sync | Implemented foundation | Fixture account setup syncs provider account signal connection. |
| Trace causation/correlation for fixture message | Implemented in tests | Observation -> raw signal -> accepted signal -> communication recorded. |
| Decision/obligation/task candidate extraction from fixture message | Implemented in tests | Downstream engines/workflows already receive communication evidence. |
| Frontend runtime panel | Implemented foundation | Fixture/manual workspace only. |
| Live runtime | Missing | No WebView/native protocol runtime. |
| QR/pair-code login | Missing | Needed for real account linking. |
| Message send/reply/forward/delete/edit | Missing | Must use provider command outbox. |
| Reactions | Missing | Projection and commands not implemented. |
| Media | Missing | Metadata, download, upload, scanner and blob refs missing. |
| Voice notes | Missing | Attachment-first model needed; STT future only. |
| Status | Missing | Status evidence and publish command missing. |
| Groups/communities/broadcasts | Missing | Projection and commands missing. |
| Phone identity traces | Missing | Need Persona candidate handoff only. |
| Search/media gallery | Missing | Needs projection and indexes. |
| Calls | Unsupported except metadata future | No call capture/recording/live handling without ADR. |
| Business Cloud API | Future provider only | Must not replace personal WhatsApp Web/native provider. |

## Current trace evidence

The fixture message test confirms the target trace shape:

```text
observation.captured.v1
  -> signal.raw.whatsapp.message.observed
  -> signal.accepted.whatsapp.message
  -> communication.message.recorded
```

The same test asserts that legacy `integration.whatsapp.%` events are not emitted for the current Signal Hub trace path. That means the live runtime should publish into the same canonical observation/signal path rather than inventing a second WhatsApp event dialect.

## Architecture debt observed

### 1. Integration store currently depends on Communications stores

`whatsapp_provider_runtime_store(pool)` constructs `WhatsappWebStore` with Communication stores directly. This exists as an application-level wiring shortcut, not as a target architecture precedent.

Target direction:

```text
Integration runtime captures provider event
  -> platform event / observation contract
  -> Communications consumer/projection
```

The integration must not own Communications tables or bypass Communications event consumers.

### 2. WhatsApp docs are zero-based while code has foundation work

`docs/whatsapp/status.md` still says production WhatsApp capability is counted as zero. That was true for the earlier audit baseline, but the current code already has fixture/provider foundation. Keep that older doc as historical baseline and use this audit for current state.

### 3. Frontend has mixed naming

Runtime UI correctly lives under `frontend/src/integrations/whatsapp/`.

Any user-facing Communication workspace should stay under:

```text
frontend/src/domains/communications/*
```

Provider-specific Communication cache keys must remain rooted under `['communications', ...]`, while runtime/setup caches may use `['integrations', 'whatsapp', 'runtime', ...]`.

### 4. Full functionality conflicts with ADR-0051 if interpreted as hidden automation

Full WhatsApp functionality requires either:

- a visible WebView companion runtime;
- an unofficial native multi-device protocol runtime;
- a future official business cloud provider.

These are three different provider shapes. They must not be blurred into one `whatsapp` adapter because that would turn the integration into a junk drawer with ambitions.

## Required next documentation decisions

Before implementing live support, add or accept:

1. Provider runtime selection ADR.
2. Full functionality target spec.
3. Provider command/outbox contract for WhatsApp.
4. Raw evidence and canonical projection contracts for every source record kind.
5. Media/blob/scanner contract.
6. Phone identity trace contract.
7. Runtime/session/secret storage contract.
8. Privacy/consent/capability UX contract.
9. Fixture-first test matrix.
10. Live manual smoke-test matrix outside CI.

## Recommended immediate implementation order

```text
P0 Documentation and ADR
P1 Provider runtime boundary and capability model
P2 Native/WebView fixture adapter contract
P3 Inbound messages/dialogs/participants/status/media metadata
P4 Provider command outbox for writes
P5 Hermes intelligence: Radar, Review, Timeline, Search, Context Packs
P6 Business Cloud provider only if explicitly needed
```

## Non-negotiables

- No hidden scraping.
- No provider write without capability, owner-visible intent, audit and reconciliation.
- No session secrets in PostgreSQL, event payloads, logs or frontend state.
- No direct domain mutation from WhatsApp integration.
- No AI-created business entities without evidence, confidence and review/promotion.
- No live account dependency in CI.
- No fake production status. Fixture is fixture. A civilization collapses one euphemism at a time.
