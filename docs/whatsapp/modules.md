# WhatsApp Modules

Статус: target module map на 2026-06-17.

WhatsApp остаётся Communication Channel. Модули ниже не создают отдельный
product domain.

This module map intentionally treats production WhatsApp Channel implementation
as not yet existing. Any compatibility or fixture files that may exist in the
repository are not counted as production coverage until a later audit aligns
them with this package.

## Backend Modules

| Module | Target files | Назначение | Status |
|---|---|---|---|
| `api` | `backend/src/integrations/whatsapp/api/` | Protected HTTP handlers for capabilities, accounts, sessions, chats, messages, media, statuses and commands | MISSING |
| `accounts` | `backend/src/integrations/whatsapp/client/accounts/` | Account metadata, lifecycle, secret/session references and account kind validation | MISSING |
| `capabilities` | `backend/src/domains/api_support/` | Operation-level capability matrix with account/runtime overrides | MISSING |
| `runtime_webview` | `backend/src/integrations/whatsapp/runtime/` | Account-scoped desktop WebView companion runtime and runtime event bridge | MISSING |
| `sessions` | `backend/src/integrations/whatsapp/client/sessions/` | Local WebView session metadata and lifecycle state | MISSING |
| `adapter` | `backend/src/integrations/whatsapp/client/` | Provider event normalization, validation and source-record conversion | MISSING |
| `dialogs` | WhatsApp client/store projection module | Private, group, community, broadcast and status dialog projection | MISSING |
| `messages` | WhatsApp client/store message module | Raw record ingestion and canonical Communication message projection | MISSING |
| `message_versions` | WhatsApp lifecycle module | Observed edit/update versions and diff metadata | MISSING |
| `message_tombstones` | WhatsApp lifecycle module | Delete evidence, local visibility and tombstone history | MISSING |
| `reactions` | WhatsApp reactions module | Reaction evidence, commands and reconciliation | MISSING |
| `participants` | WhatsApp participants module | Phone-centric participant and role evidence | MISSING |
| `identity_traces` | WhatsApp identity module | Phone number, `wa_id`, display-name and contact-card traces | MISSING |
| `statuses` | WhatsApp status module | Status source evidence, media evidence and timeline signals | MISSING |
| `media` | WhatsApp media module | Provider media metadata, download/upload and projection refresh | MISSING |
| `attachments` | Communication attachment/blob boundary | Local blob persistence, metadata and scanner state | MISSING |
| `voice_notes` | WhatsApp voice module | Voice metadata, download/playback and future transcript handoff | MISSING |
| `calls` | Platform calls integration | Call metadata, call evidence and Timeline entries | MISSING |
| `command_outbox` | WhatsApp provider command module | Durable provider-write queue, retry and reconciliation | MISSING |
| `realtime_events` | shared event bus integration | Sanitized `whatsapp.*` event contracts | MISSING |
| `search` | WhatsApp search module | Message/media/status/participant search over local projections | MISSING |
| `audit` | platform audit boundary | Redacted lifecycle, capability and provider-write audit records | MISSING |

## Missing / Target Backend Modules

| Module | Назначение | Why needed |
|---|---|---|
| `webview_companion_runtime` | Owner-linked WhatsApp Web session | Required before live sync or provider writes |
| `capability_matrix` | Per-operation capability model | Required before exposing UI commands |
| `command_outbox` | Durable provider-write command queue | Required for send/reply/forward/reaction/delete/media/status/voice commands |
| `provider_observed_reconciliation` | Confirm provider state after command execution | Required before marking commands `completed` |
| `status_projection` | WhatsApp Status evidence projection | Required for status read/publish and Timeline evidence |
| `phone_identity_trace_projection` | Phone/wa_id/display-name evidence | Required for Persona candidates without implementing Persona lifecycle |
| `media_transfer` | Download/upload orchestration through runtime | Required for attachments, media gallery and voice notes |
| `call_metadata_projection` | Call source evidence only | Required without enabling live call handling |
| `attachment_scanner_integration` | Scanner-backed verdicts | Required before safe preview can mark files `clean` |
| `provider_search` | Optional WhatsApp provider search | Required only after WebView runtime can support it safely |

## Frontend Modules

| Module | Target files | Назначение | Status |
|---|---|---|---|
| `page` | `frontend/src/domains/whatsapp/views/` | Desktop WhatsApp workbench | MISSING |
| `api` | `frontend/src/domains/whatsapp/api/` | Typed backend route calls | MISSING |
| `queries` | `frontend/src/domains/whatsapp/queries/` | TanStack Query hooks/mutations | MISSING |
| `store` | `frontend/src/domains/whatsapp/stores/` | Local UI state, filters and selected context | MISSING |
| `account_setup` | WhatsApp account/session components | Account setup, link flow, logout/remove and runtime status | MISSING |
| `dialogs` | Dialog list components | Private/group/community/broadcast/status list | MISSING |
| `messages` | Message thread components | Source-backed timeline, reply/forward/delete/reaction UI | MISSING |
| `composer` | Composer components | Capability-gated send/reply/media/voice/status commands | MISSING |
| `participants` | Participant inspector components | Phone-centric participant evidence and role labels | MISSING |
| `media_viewer` | Media components | Photo/video/document/audio/sticker/gif preview and download state | MISSING |
| `voice_player` | Voice-note components | Local playback and transcript placeholder without STT | MISSING |
| `status_view` | Status components | Status evidence timeline and publish command surface | MISSING |
| `calls_panel` | Call evidence components | Read-only call metadata and Timeline evidence | MISSING |
| `command_audit` | Command inspector | Outbox status, retry/dead-letter diagnostics | MISSING |
| `realtime_status` | Shared realtime status consumption | WS/SSE/long-poll state and cache patch diagnostics | MISSING |

## Missing / Target Frontend Modules

| Module | Назначение |
|---|---|
| `webview_link_flow` | Owner-visible QR/link/session lifecycle |
| `runtime_status` | Companion runtime blocked/degraded/available diagnostics |
| `capability_matrix` | Account-scoped operation status and explanations |
| `dialog_inspector` | Participants, phone traces, admin/member evidence |
| `message_actions` | Send/reply/forward/reaction/delete with capability gates |
| `media_gallery` | Media grouped by chat, type and local availability |
| `status_surface` | WhatsApp Status read/publish evidence surface |
| `voice_note_player` | Voice playback without automatic STT |
| `call_evidence_panel` | Metadata-only call timeline entries |
| `outbox_panel` | Command queue status, retry and dead-letter review |
| `realtime_patches` | Cache patching for `whatsapp.*` events |

## Functional Module Map

| Capability module | Назначение | Current production status |
|---|---|---|
| `accounts` | Account metadata, lifecycle and secret/session refs | MISSING |
| `runtime_fixture` | Deterministic local/test runtime | MISSING |
| `runtime_webview` | Desktop WhatsApp Web companion runtime | MISSING |
| `runtime_business_cloud` | Future Meta Business provider | UNSUPPORTED |
| `dialogs` | Private/group/community/broadcast/status projection | MISSING |
| `private_chats` | 1:1 chat projection | MISSING |
| `groups` | Group chat projection and participant evidence | MISSING |
| `communities` | Community evidence and member projection | MISSING |
| `broadcasts` | Broadcast evidence projection | MISSING |
| `statuses` | WhatsApp Status source/timeline evidence | MISSING |
| `messages` | Source-backed message projection | MISSING |
| `message_versions` | Observed edits/updates | MISSING |
| `message_tombstones` | Deletes/local visibility history | MISSING |
| `replies` | Reply target projection | MISSING |
| `forwards` | Forward attribution when provider exposes it | MISSING |
| `reactions` | Reaction evidence and provider commands | MISSING |
| `participants` | Phone-centric member/admin/community evidence | MISSING |
| `identity_traces` | Phone, `wa_id`, display name and contact-card traces | MISSING |
| `media` | Media metadata, download/upload and gallery | MISSING |
| `attachments` | Communication attachment rows and local blobs | MISSING |
| `voice_notes` | Voice attachment, playback and transcript handoff | MISSING |
| `calls` | Metadata-only call evidence | MISSING |
| `search` | Local projection search | MISSING |
| `provider_search` | WhatsApp provider-side search | MISSING |
| `sync` | Chat/message/status/media sync | MISSING |
| `realtime` | Shared realtime event contracts and cache patching | MISSING |
| `outbox` | Durable provider-write command lifecycle | MISSING |
| `audit` | Redacted provider-write/lifecycle audit | MISSING |
| `ai` | Shared-engine integration points only | MISSING |

## Module Boundary Rules

WhatsApp code may depend on:

```text
Communications
Events
Timeline interfaces
Shared attachment/blob boundary
Search engine interface
Risk/enrichment candidate interfaces
Audit
Secret resolver / host vault
Capability runtime
```

WhatsApp code must not own or implement:

```text
Obligation lifecycle
Decision lifecycle
Memory lifecycle
Knowledge lifecycle
Persona Intelligence
Organization Intelligence
Project Intelligence
```

WhatsApp may produce evidence and candidates for those systems only.

## Naming Rules

Use provider/product names precisely:

- `WhatsApp Channel` for the Hermes communication channel.
- `WhatsApp Web` for the primary provider source.
- `whatsapp_web` for the provider kind.
- `whatsapp_personal` and `whatsapp_business` for account kinds.
- `whatsapp_business_cloud` only for a future Meta Business API provider.

Do not describe this package as a CRM, contact manager, task tracker, note app
or standalone WhatsApp client.
