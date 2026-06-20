# WhatsApp Implementation Status

Статус на 2026-06-17.

This is a starting production-channel audit. Per request, the WhatsApp Channel
implementation is treated as not yet existing for this package.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

Implementation audit totals:

```text
IMPLEMENTED = 0
PARTIAL = planned only
MISSING = all production capabilities
```

Percentages below describe WhatsApp Channel only. They are not an evaluation of
Communications, Memory, Knowledge, Obligations, Decisions, Personas,
Organizations or Timeline.

## Summary Table

| § | Раздел | Статус | % |
|---|---|---|---:|
| 1 | Channel framing and ADR alignment | planned | 0 |
| 2 | Provider/account model | MISSING | 0 |
| 3 | Secret references and host-vault boundary | MISSING | 0 |
| 4 | Capability contract | MISSING | 0 |
| 5 | Fixture/manual runtime | MISSING | 0 |
| 6 | WebView companion runtime | MISSING | 0 |
| 7 | Meta Business future provider separation | MISSING | 0 |
| 8 | Dialog/chat list | MISSING | 0 |
| 9 | Private chats | MISSING | 0 |
| 10 | Groups/communities/broadcasts | MISSING | 0 |
| 11 | WhatsApp Statuses | MISSING | 0 |
| 12 | Message ingestion/projection | MISSING | 0 |
| 13 | Message lifecycle commands | MISSING | 0 |
| 14 | Replies/forwards | MISSING | 0 |
| 15 | Reactions | MISSING | 0 |
| 16 | Message history/versioning | MISSING | 0 |
| 17 | Media metadata/download/upload | MISSING | 0 |
| 18 | Attachments preview/search/dedup | MISSING | 0 |
| 19 | Voice notes | MISSING | 0 |
| 20 | Calls metadata | MISSING | 0 |
| 21 | Phone-centric identity traces | MISSING | 0 |
| 22 | Search | MISSING | 0 |
| 23 | Realtime | MISSING | 0 |
| 24 | Frontend workbench | MISSING | 0 |
| 25 | Timeline/shared engine integration points | MISSING | 0 |
| 26 | Offline/outbox/export/session bundles | MISSING | 0 |
| 27 | Dialog management: unread/pinned/archive/mute | MISSING | 0 |
| 28 | Provider-write command model | MISSING | 0 |
| 29 | Privacy/security/capability UX | MISSING | 0 |
| 30 | Documentation and audit set | planned | 0 |

## Legend

- `IMPLEMENTED` — implemented and confirmed by current files/tests/docs audit.
- `PARTIAL` — planned/documented only in this starting WhatsApp package.
- `MISSING` — durable production implementation is not counted in this audit.
- `UNSUPPORTED` — intentionally out of current scope or conflicts with policy.

No production WhatsApp Channel capability is marked `IMPLEMENTED`.

## Details

### §2 Provider/account model — 0%

Target account kinds:

```text
whatsapp_personal
whatsapp_business
```

Target provider kind:

```text
whatsapp_web
```

Missing:

- account metadata lifecycle;
- account-scoped runtime state;
- account-scoped capability matrix;
- logout/remove preserving evidence;
- multiple account validation;
- Meta Business API separation as future provider.

### §3 Secret references and host-vault boundary — 0%

Missing:

- `account_id + secret_purpose` lookup for WebView session protection material;
- host-vault-backed session secret policy;
- local WebView profile storage policy;
- secret redaction tests;
- session revocation and relink UX.

Secrets must not be stored in account config, audit records, event payloads or
frontend state.

### §4 Capability contract — 0%

Missing operation-level states for:

- send;
- reply;
- forward;
- reaction;
- delete;
- media upload;
- media download;
- join group;
- leave group;
- status read;
- status publish;
- voice send.

Required states:

```text
available
degraded
blocked
unsupported
```

Initial production state is `blocked` for architecturally allowed live features
and `unsupported` for calls control, recording, live handling and STT.

### §5 Fixture/manual runtime — 0%

Missing:

- deterministic fixture account;
- manual companion-state fixture;
- fixture message/status/media ingestion;
- fixture realtime event tests;
- local smoke validation.

### §6 WebView companion runtime — 0%

Missing:

- desktop-owned WebView runtime;
- owner-visible link/session flow;
- runtime start/stop/status;
- storage path policy;
- session expiry detection;
- provider event bridge;
- smoke validation.

Live runtime must remain `blocked` until these are present.

### §7 Meta Business future provider separation — 0%

Missing:

- separate provider kind;
- separate auth/account model;
- separate capability matrix;
- separate command model;
- separate ADR/update.

Meta Business API must not replace the personal WhatsApp Web architecture.

### §8 Dialog/chat list — 0%

Missing projections for:

- private chat;
- group;
- community;
- broadcast;
- status.

Missing UI:

- dialog list;
- filters;
- selected dialog detail;
- provider state labels;
- runtime/degraded state display.

### §9 Private chats — 0%

Missing:

- 1:1 chat projection;
- phone/display-name identity traces;
- message timeline;
- provider read state when observed;
- source evidence references.

### §10 Groups/communities/broadcasts — 0%

Missing:

- group projection;
- community projection;
- broadcast projection;
- participant/member/admin evidence;
- join/leave command outbox;
- provider-observed reconciliation.

### §11 WhatsApp Statuses — 0%

WhatsApp Status target model:

```text
source evidence
timeline evidence
identity signal
```

Missing:

- status raw records;
- status projection;
- media attachment linkage;
- read capability;
- publish command outbox;
- Timeline integration.

Statuses are not a separate domain.

### §12 Message ingestion/projection — 0%

Missing:

- raw `whatsapp_web_message` records;
- canonical Communication projection;
- idempotent import keys;
- provider message identity;
- sanitized raw evidence read;
- shared engine candidate refresh boundary.

### §13 Message lifecycle commands — 0%

Missing provider-write commands for:

- send;
- reply;
- forward;
- delete;
- edit if supported;
- status publish;
- voice send.

All commands must use the outbox and capability contract.

### §14 Replies/forwards — 0%

Missing:

- reply target projection;
- reply context UI;
- forward attribution projection;
- provider command routes;
- provider-observed reconciliation.

### §15 Reactions — 0%

Missing:

- reaction evidence projection;
- add/change/remove commands;
- `whatsapp.reaction.changed` event emission;
- frontend cache patching;
- reconciliation state.

### §16 Message history/versioning — 0%

Missing:

- observed update versions;
- tombstone records;
- deletion reason classes;
- local visibility state;
- diff views;
- source-backed lifecycle evidence.

Hermes must not reconstruct unobserved provider history.

### §17 Media metadata/download/upload — 0%

Missing:

- media projection;
- media download lifecycle;
- media upload command;
- gallery/search;
- scanner state;
- preview artifacts.

Target media classes:

- photo;
- video;
- document;
- audio;
- voice note;
- contact;
- location;
- sticker;
- gif.

### §18 Attachments preview/search/dedup — 0%

Missing:

- local blob persistence;
- attachment rows;
- scanner-backed verdicts;
- safe preview routes;
- dedup UX;
- local media search.

### §19 Voice notes — 0%

Supported architecture target:

- voice metadata;
- voice attachment;
- voice playback;
- future transcript integration.

Missing all runtime/UI/storage support. STT is out of first-version scope.

### §20 Calls metadata — 0%

Supported first-version target:

- call metadata;
- call evidence;
- call timeline entries.

Missing all call evidence support.

Out of scope:

- audio capture;
- video capture;
- call control;
- recording;
- live call handling;
- STT.

### §21 Phone-centric identity traces — 0%

Missing:

- phone trace storage;
- `wa_id` trace storage;
- display-name history;
- contact-card source evidence;
- group/admin/community role evidence;
- Persona candidate handoff.

WhatsApp does not implement Persona lifecycle.

### §22 Search — 0%

Missing:

- local message search;
- dialog search;
- media search;
- status search;
- participant/phone trace search;
- provider search capability.

### §23 Realtime — 0%

Missing event contracts:

```text
whatsapp.message.created
whatsapp.message.updated
whatsapp.message.deleted
whatsapp.chat.updated
whatsapp.reaction.changed
whatsapp.media.download.started
whatsapp.media.download.progress
whatsapp.media.download.completed
whatsapp.media.download.failed
whatsapp.command.status_changed
whatsapp.command.reconciled
```

Missing frontend cache patching and replay semantics.

### §24 Frontend workbench — 0%

Missing:

- desktop three-pane WhatsApp workbench;
- account/session setup;
- dialog list;
- thread view;
- composer;
- media/status/call panels;
- command audit panel;
- capability matrix.

### §25 Timeline/shared engine integration points — 0%

Missing:

- Timeline evidence emission;
- Search integration;
- Risk/enrichment candidate handoff;
- Obligation/Decision candidate handoff;
- Polygraph evidence wiring.

WhatsApp may emit candidates only.

### §26 Offline/outbox/export/session bundles — 0%

Missing:

- durable outbox;
- retry/dead-letter handling;
- offline command replay;
- export policy;
- encrypted session bundle design;
- session import/export ADR.

### §27 Dialog management — 0%

Missing:

- read/unread;
- pin/unpin;
- archive/unarchive;
- mute/unmute;
- folder/list overlays;
- provider-observed reconciliation.

These features require capability states before UI exposure.

### §28 Provider-write command model — 0%

Missing command states:

```text
queued
executing
retrying
completed
failed
dead_letter
```

Missing:

- idempotency keys;
- per-command target metadata;
- execution locking;
- retry policy;
- provider-observed reconciliation;
- redacted audit;
- command realtime events.

### §29 Privacy/security/capability UX — 0%

Missing:

- owner-visible session controls;
- capability explanations;
- confirmation gates;
- no-hidden-recording UI boundary;
- redacted audit review;
- media safety labels;
- blocked/degraded runtime diagnostics.

### §30 Documentation and audit set — planned

This package defines the target documentation set. It does not count as runtime,
API, storage or UI implementation.

## Recommended next implementation order

### P0 — Foundation before provider writes

1. Confirm ADR posture for WebView companion runtime and hidden-vs-visible owner
   controls.
2. Define operation-level capability contract and account/session model.
3. Add fixture/manual source-record validation before live WebView runtime.
4. Add append-only raw records and canonical Communication projections.

### P1 — Read-only evidence first

1. Dialog/message/status projections.
2. Media metadata and safe local blob download.
3. Phone-centric identity traces.
4. Timeline evidence entries.
5. Realtime created/updated/deleted/chat/media events.

### P2 — Provider-write command model

1. Durable command outbox.
2. Send/reply/reaction/delete/media/status/voice commands.
3. Retry/dead-letter/audit.
4. Provider-observed reconciliation before `completed`.

### P3 — Native/ADR blocked

1. Call runtime and controls.
2. Audio/video capture.
3. Recording.
4. STT/transcripts.
5. Meta Business Cloud provider.
