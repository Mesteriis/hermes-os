# Communications Review Report

Date: 2026-06-18

Scope: Communications, Telegram and WhatsApp architecture review only. No code,
API or schema changes were made.

## Review Question

Do Telegram and WhatsApp belong to Communications as subdomains/channels rather
than standalone platforms inside Hermes?

Audit answer: yes.

Both Telegram and WhatsApp should remain Communication Channels. They supply
source evidence, provider commands, realtime events, identity traces, timeline
evidence and media evidence. They must not own Memory, Knowledge, Persona,
Organization, Project, Task, Decision or Obligation lifecycle.

## Target Structure

```text
communications/
  conversations/
  messages/
  participants/
  attachments/
  search/
  realtime/

channels/
  email/
  telegram/
  whatsapp/
```

Current docs use `docs/mail`, `docs/telegram` and `docs/whatsapp`. Treat these
as channel capability specs until a future documentation restructuring pass.

## Current Evidence

| Area | Current evidence | Audit conclusion |
|---|---|---|
| Email | `docs/mail/*`, backend `domains/mail/*`, `/api/v1/communications/*`. | Email is a mature channel with mail-heavy implementation names. |
| Telegram | `docs/telegram/*`, ADR-0083, ADR-0091, ADR-0094, backend `integrations/telegram/*`, frontend `domains/telegram/*`. | Telegram is a completed base Communication Channel for supported scope. |
| WhatsApp | `docs/whatsapp/*`, ADR-0051, backend `integrations/whatsapp/*`, frontend `domains/whatsapp/*`. | WhatsApp is a target channel spec with production capabilities mostly missing or blocked. |
| Shared storage | Mail blob/attachment boundary, communication attachment imports, channel media docs. | Attachments should be shared under Communications. |
| Realtime | Shared event bus and frontend bootstrap consumed by Telegram. | Channel-specific realtime events should feed shared cache patching. |
| Capabilities | Telegram capability matrix; WhatsApp target matrix; mail provider tiers draft. | Capability model should be shared and account-scoped. |

## Duplication Review

### Model Duplication

Potential duplication:

- channel account metadata;
- conversation/chat/thread/dialog projections;
- message identity and lifecycle;
- participant/member/sender evidence;
- attachment/media metadata;
- provider command outbox;
- search result shape;
- realtime event patching;
- capability state and action classes.

Conclusion: these are shared Communications abstractions with channel-specific
provider details. They should not be re-invented per channel as independent
domain models.

### Lifecycle Duplication

Common lifecycle:

```text
provider event or fixture
  -> raw source record
  -> canonical communication projection
  -> event/realtime notification
  -> engine/candidate refresh
  -> UI query/cache update
```

Common provider-write lifecycle:

```text
owner command
  -> capability check
  -> durable provider command
  -> runtime dispatch
  -> provider-observed reconciliation
  -> projection refresh
  -> audit and realtime event
```

Conclusion: Telegram has the strongest current lifecycle implementation.
WhatsApp should reuse the same pattern instead of creating a parallel command
model.

### API Duplication

Current channel APIs are necessarily provider-specific for runtime and command
details. The duplicated risk is not the route prefix itself. The risk is
duplicating shared semantics:

- `/messages`;
- `/attachments`;
- `/search`;
- `/commands`;
- `/capabilities`;
- `/events`;
- `/participants`.

Conclusion: keep provider route groups when needed, but normalize response
shapes and ownership vocabulary around Communications.

### Capability Duplication

Telegram, WhatsApp and mail all need capability reporting. Shared action
classes should include:

- `read`;
- `local_write`;
- `provider_write`;
- `destructive`;
- `export`;
- `secret_access`;
- `recording`;
- `automation`.

Telegram adds `planned`; ADR-0094 makes it part of the channel capability
contract. Other channel capability specs should either adopt `planned` or
explicitly document why they do not.

## Shared Abstractions To Extract In Documentation First

| Abstraction | Purpose | Notes |
|---|---|---|
| ChannelAccount | Account-scoped provider metadata and lifecycle. | Credentials stay behind secret references and host vault. |
| ConversationProjection | Provider thread/chat/dialog mapped to Hermes conversation semantics. | Channel-specific provider IDs remain in metadata. |
| MessageProjection | Canonical communication message with source evidence. | Must remain projection from raw records. |
| ParticipantEvidence | Observed sender/recipient/member/caller/attendee role. | Does not equal Persona truth. |
| AttachmentBoundary | Shared blob metadata, scanner state, preview/search and Document promotion. | Bytes stay out of PostgreSQL. |
| ProviderCommandOutbox | Durable lifecycle for provider writes. | Completion requires provider-observed evidence. |
| CapabilityMatrix | Account/runtime operation status. | UI consumes backend authority. |
| RealtimePatchEvent | Sanitized event for query cache updates. | Shared frontend bootstrap should consume channel events. |
| ChannelSearch | Local projection search plus provider refresh where supported. | Search indexes remain derived. |
| AuditRecord | Redacted high-risk action metadata. | No bodies, document contents or secrets. |

## Telegram Findings

Telegram is documented as a Communication Channel and ADR-0094 marks the base
channel scope complete for supported desktop work. Deferred initiatives are not
base gaps. Future Telegram work should start as separate initiatives, not as
open-ended Telegram domain expansion.

Key guardrails:

- no raw TDLib payloads in UI;
- no provider ACK as success without provider-observed evidence;
- no component-level fetch for server state;
- no hidden recording;
- no Telegram-owned Memory, Knowledge, Persona, Organization, Project,
  Obligation or Decision lifecycle.

## WhatsApp Findings

WhatsApp documentation is correctly framed as a Communication Channel. The
current target is a WhatsApp Web desktop companion boundary, not hidden
scraping and not Meta Business API by default.

Before implementation, WhatsApp needs:

- account/session lifecycle;
- owner-visible runtime status;
- operation-level capabilities;
- append-only raw records;
- canonical message/dialog/status projections;
- shared attachment/media handling;
- durable provider command outbox;
- provider-observed reconciliation;
- sanitized realtime events;
- redacted audit.

## Recommended Next Steps

1. Keep `docs/telegram` and `docs/whatsapp` as channel capability specs for now.
2. Add a Communications shared abstractions document before any code refactor.
3. Create a WhatsApp implementation RFC using Telegram lifecycle as reference,
   without copying Telegram provider assumptions.
4. Normalize capability vocabulary across mail, Telegram and WhatsApp.
5. Only after docs are accepted, design code-level Communications refactoring.
