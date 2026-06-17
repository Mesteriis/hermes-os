# WhatsApp Gap Analysis

Status date: 2026-06-17.

Labels:

- `IMPLEMENTED` — implemented and confirmed by current files/tests/docs audit.
- `PARTIAL` — planned/documented only in this starting package.
- `BROKEN` — implementation exists but current evidence shows it does not work.
- `MISSING` — durable production implementation is not counted in this audit.
- `REGRESSION` — current behavior is worse than previously documented behavior.
- `UNSUPPORTED` — intentionally out of current scope or conflicts with policy.

Evidence sources: ADR-0051, ADR-0018, ADR-0052, ADR-0074, ADR-0085,
Communications domain docs and the existing channel documentation structure.

This audit intentionally starts from:

```text
IMPLEMENTED = 0
PARTIAL = planned only
MISSING = all production capabilities
```

No confirmed `BROKEN` or `REGRESSION` capability is listed because no production
WhatsApp Channel implementation is counted in this audit.

## Accounts / Runtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| `whatsapp_personal` account | MISSING | Account kind is required but not counted as implemented. |
| `whatsapp_business` account | MISSING | Business App via WhatsApp Web companion is required; Meta Cloud is separate future provider. |
| Multiple accounts | MISSING | Account-scoped metadata, sessions and runtime actors are required. |
| Secret/session boundary | MISSING | Session protection must use `account_id + secret_purpose`; no secrets in PostgreSQL. |
| WebView companion runtime | MISSING | ADR-0051 requires owner-visible desktop runtime before live support. |
| Fixture/manual runtime | MISSING | Needed for deterministic validation before live provider work. |
| Runtime health | MISSING | Need blocked/degraded diagnostics for session, WebView, storage and validation. |
| Meta Business Cloud provider | UNSUPPORTED | Future provider shape only, not a substitute for personal WhatsApp Web. |

## Dialogs / Chat Management

| Capability | Status | Evidence / Gap |
|---|---|---|
| Private chats | MISSING | Need account-scoped dialog projection and message timeline. |
| Groups | MISSING | Need group projection, participant evidence and join/leave commands. |
| Communities | MISSING | Need community membership and source evidence model. |
| Broadcasts | MISSING | Need broadcast evidence projection without treating it as a separate domain. |
| Status dialog surface | MISSING | Need Status evidence projection and Timeline integration. |
| Unread/read state | MISSING | Requires provider-observed evidence and command model if mirrored to provider. |
| Pin/archive/mute overlays | MISSING | Need local-vs-provider state distinction and capability states. |
| Join/leave | MISSING | Must be provider-write commands with outbox and reconciliation. |

## Messaging

| Capability | Status | Evidence / Gap |
|---|---|---|
| Text messages | MISSING | Need raw records, projection and `whatsapp.message.created`. |
| Replies | MISSING | Need reply target projection and reply command. |
| Forwards | MISSING | Need forward attribution when provider exposes it and command path. |
| Deletes | MISSING | Need tombstones, delete evidence and destructive command path. |
| Edits | MISSING | Only if provider/runtime supports it; must store observed versions only. |
| Raw provider evidence | MISSING | Need sanitized raw evidence route and append-only source records. |
| Message identity | MISSING | Need account/chat/message/sender identifiers and raw record links. |

## Reactions

| Capability | Status | Evidence / Gap |
|---|---|---|
| Reaction projection | MISSING | Need source-backed reaction state. |
| Add/change reaction | MISSING | Provider-write command required. |
| Remove reaction | MISSING | Provider-write command required. |
| Reaction realtime | MISSING | Need `whatsapp.reaction.changed`. |
| Provider reconciliation | MISSING | Need observed state before commands become `completed`. |

## Media

| Capability | Status | Evidence / Gap |
|---|---|---|
| Photos | MISSING | Need metadata, download, preview and local blob path. |
| Videos | MISSING | Need metadata, download, preview and local blob path. |
| Documents | MISSING | Need metadata, scanner state and safe preview route. |
| Audio | MISSING | Need download/playback through local blob. |
| Voice notes | MISSING | Need metadata, attachment and playback; STT out of scope. |
| Contacts | MISSING | Contact cards are identity evidence, not Persona lifecycle. |
| Locations | MISSING | Location evidence must be source-backed and timeline-capable. |
| Stickers | MISSING | Need media metadata and preview after download. |
| GIF | MISSING | Need animation metadata and preview after download. |
| Media upload | MISSING | Must use local attachment/blob and provider-write outbox. |
| Media download | MISSING | Must emit started/progress/completed/failed events. |
| Media gallery/search | MISSING | Needs projection-backed local search. |

## Attachments

| Capability | Status | Evidence / Gap |
|---|---|---|
| Local blob storage | MISSING | Media bytes must stay outside PostgreSQL. |
| Attachment metadata | MISSING | Need provider refs, hash, scanner state and local refs. |
| Deduplication | MISSING | Needs shared blob hash semantics. |
| Scanner-backed clean verdict | MISSING | No attachment may be marked `clean` without scanner backend. |
| Preview artifacts | MISSING | Need safe preview route and generated artifacts. |

## Participants / Identity

| Capability | Status | Evidence / Gap |
|---|---|---|
| Phone traces | MISSING | Phone number is evidence, not Persona truth. |
| `wa_id` traces | MISSING | Provider-specific identity evidence required. |
| Display names | MISSING | Mutable labels need observed history/source evidence. |
| Contact resolution | MISSING | Candidate handoff only; no Persona lifecycle in WhatsApp. |
| Persona candidates | MISSING | Source-backed candidate flow required. |
| Group member evidence | MISSING | Need role/status/admin/member evidence. |
| Community member evidence | MISSING | Need community-scoped evidence. |

## WhatsApp Statuses

| Capability | Status | Evidence / Gap |
|---|---|---|
| Status read | MISSING | Provider read capability required. |
| Status publish | MISSING | Provider-write command required. |
| Status media | MISSING | Media attachment linkage required. |
| Status identity signal | MISSING | Identity traces must remain evidence/candidates only. |
| Status timeline entry | MISSING | Timeline evidence integration required. |

WhatsApp Status is not a separate domain.

## Calls

| Capability | Status | Evidence / Gap |
|---|---|---|
| Call metadata | MISSING | First-version scope allows metadata only. |
| Call evidence | MISSING | Need source-backed call records. |
| Call timeline entries | MISSING | Need Timeline evidence projection. |
| Audio capture | UNSUPPORTED | Out of scope until future ADR. |
| Video capture | UNSUPPORTED | Out of scope until future ADR. |
| Call control | UNSUPPORTED | Out of scope until future ADR. |
| Recording | UNSUPPORTED | Hidden recording is forbidden. |
| Live call handling | UNSUPPORTED | Out of scope until future ADR. |
| STT | UNSUPPORTED | Out of first-version scope. |

## Voice Notes

| Capability | Status | Evidence / Gap |
|---|---|---|
| Voice metadata | MISSING | Need provider metadata projection. |
| Voice attachment | MISSING | Need local blob and attachment row. |
| Voice playback | MISSING | Playback from local blob only after download. |
| Voice send | MISSING | Provider-write command from local blob. |
| Future transcript integration | PARTIAL | Planned integration point only; no STT in first version. |

## Provider Command Outbox

| Capability | Status | Evidence / Gap |
|---|---|---|
| Durable command rows | MISSING | Required before provider writes. |
| `queued` state | MISSING | Required command state. |
| `executing` state | MISSING | Required command state. |
| `retrying` state | MISSING | Required command state. |
| `completed` state | MISSING | Must require provider-observed state. |
| `failed` state | MISSING | Required command state. |
| `dead_letter` state | MISSING | Required command state. |
| Retry policy | MISSING | Needed for transient WebView/provider failures. |
| Reconciliation | MISSING | Needed before marking commands complete. |
| Redacted audit | MISSING | Needed for all provider-write/destructive commands. |

## Realtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| Generic event transport | MISSING | Shared transport exists in Hermes, but WhatsApp contracts are not counted. |
| New message event | MISSING | Need `whatsapp.message.created`. |
| Updated message event | MISSING | Need `whatsapp.message.updated`. |
| Deleted message event | MISSING | Need `whatsapp.message.deleted`. |
| Chat update event | MISSING | Need `whatsapp.chat.updated`. |
| Reaction event | MISSING | Need `whatsapp.reaction.changed`. |
| Media download lifecycle | MISSING | Need started/progress/completed/failed. |
| Command status | MISSING | Need status_changed/reconciled. |
| Frontend cache patching | MISSING | Need shared realtime bootstrap consumers. |

## API / UI

| Capability | Status | Evidence / Gap |
|---|---|---|
| API route set | MISSING | Target routes documented only. |
| Frontend API client | MISSING | Target modules documented only. |
| Workbench | MISSING | Desktop-first UI target only. |
| Account/session setup | MISSING | Required before live runtime. |
| Capability UX | MISSING | Needed before command controls. |
| Command audit panel | MISSING | Needed for outbox visibility. |
| Status surface | MISSING | Needed for WhatsApp Status evidence. |
| Media viewer | MISSING | Needed for attachment preview/playback. |

## AI / Shared Engines

| Capability | Status | Evidence / Gap |
|---|---|---|
| Summary | MISSING | Shared-engine candidate only, not WhatsApp-owned memory. |
| Translation | MISSING | Future shared-engine UX. |
| Task extraction | MISSING | Candidate handoff only. |
| Obligation extraction | MISSING | Candidate handoff only. |
| Decision extraction | MISSING | Candidate handoff only. |
| Persona extraction | MISSING | Identity trace candidate only. |
| Polygraph evidence | MISSING | Source-backed contradiction input only. |
| AI state lifecycle | MISSING | No WhatsApp-specific lifecycle intended. |

## Scope Boundary

| Capability | Status | Evidence / Gap |
|---|---|---|
| Memory Engine | MISSING | Not owned by WhatsApp. |
| Knowledge Engine | MISSING | Not owned by WhatsApp. |
| Persona Intelligence | MISSING | Identity traces only. |
| Organization Intelligence | MISSING | Candidate evidence only. |
| Task lifecycle | MISSING | Candidate evidence only. |
| Decision lifecycle | MISSING | Candidate evidence only. |
| Obligation lifecycle | MISSING | Candidate evidence only. |

## Priority Recommendations

### P0 — Documentation-aligned foundation

1. Keep WhatsApp under Communications and source evidence.
2. Resolve hidden WebView wording against ADR-0051 with owner-visible controls.
3. Define account/session/capability contracts before runtime work.
4. Build fixture/manual evidence path before live WebView automation.

### P1 — Read-only provider evidence

1. Dialog/message/status raw records and projections.
2. Phone-centric identity traces.
3. Media metadata and conservative download.
4. Realtime message/chat/media events.

### P2 — Provider-write parity

1. Durable outbox.
2. Command audit and retry/dead-letter UX.
3. Send/reply/forward/reaction/delete/media/status/voice commands.
4. Provider-observed reconciliation.

### P3 — ADR-blocked future work

1. Live call handling.
2. Audio/video capture.
3. Recording.
4. STT.
5. Meta Business Cloud provider.
