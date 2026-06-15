# Telegram Implementation Status

Статус на 2026-06-15.

Проценты ниже описывают coverage Telegram Channel only. Они не являются оценкой
готовности Communications, Memory, Knowledge, Obligations, Decisions, Personas,
Organizations или Timeline.

## Summary Table

| § | Раздел | Статус | % |
|---|---|---|---:|
| 1 | Channel framing and ADR alignment | ✓ | 100 |
| 2 | Provider/account model | ◐ | 70 |
| 3 | Secret references and host-vault boundary | ✓ | 90 |
| 4 | Capability contract | ◐ | 35 |
| 5 | Fixture runtime | ✓ | 100 |
| 6 | TDLib QR user runtime | ◐ | 60 |
| 7 | Bot runtime | ✗ | 10 |
| 8 | Dialog/chat list | ◐ | 55 |
| 9 | Private chats | ◐ | 60 |
| 10 | Groups/supergroups/channels | ◐ | 35 |
| 11 | Topics/forums | ✗ | 0 |
| 12 | Message ingestion/projection | ✓ | 85 |
| 13 | Message lifecycle commands | ◐ | 20 |
| 14 | Replies/forwards/pins | ✗ | 5 |
| 15 | Reactions | ✗ | 0 |
| 16 | Message history/versioning | ◐ | 25 |
| 17 | Media metadata/download | ◐ | 45 |
| 18 | Attachments preview/search/dedup | ◐ | 25 |
| 19 | Voice/video messages | ✗ | 10 |
| 20 | Calls/transcripts | ◐ | 35 |
| 21 | Search | ◐ | 30 |
| 22 | Realtime | ◐ | 15 |
| 23 | AI assistance | ◐ | 20 |
| 24 | Frontend workbench | ◐ | 55 |
| 25 | Timeline/shared engine integration points | ◐ | 35 |
| 26 | Offline/outbox/export/proxy/session bundles | ✗ | 0 |
| 27 | Dialog management: unread/pinned/archive/mute/folders | ◐ | 15 |
| 28 | Provider-write command model | ✗ | 10 |
| 29 | Privacy/security/capability UX | ◐ | 30 |
| 30 | Documentation and audit set | ✓ | 100 |

## Legend

- ✓ — реализовано для текущего scope.
- ◐ — есть foundation или частичная реализация, но production scope не закрыт.
- ✗ — durable implementation не найден в текущем репозитории.

## Details

### §2 Provider/account model — 70%

`telegram_user` and `telegram_bot` provider kinds exist. Multiple accounts are
supported. Account lifecycle can list, logout and remove while preserving
evidence.

Missing:

- session import/export;
- proxy profiles;
- live bot runtime;
- richer account capability breakdown;
- provider folder/session UX.

### §3 Secret references — 90%

Telegram API hash, bot token and session key purposes are account-scoped and
resolved by secret reference. Host-vault writes exist for new live credentials.

Remaining gap:

- session/proxy secret bundle UX;
- explicit secret rotation/reconnect UI.

### §4 Capability contract — 35%

`/api/v1/telegram/capabilities` exists but is coarse.

Missing detailed states for:

- edit;
- delete;
- reactions;
- topics;
- exports;
- proxies;
- session bundles;
- calls;
- destructive/admin actions;
- media upload/download;
- offline command queue.

### §5 Fixture runtime — 100%

Fixture runtime is deterministic and should remain the validation foundation for
CI/local testing, especially when TDLib is unavailable. Human civilization has
apparently decided native dependencies should be everyone’s problem forever.

### §6 TDLib QR runtime — 60%

QR login start/status/password/cancel exists. `tdlib_qr_authorized` runtime can
start actor, sync chats/history, send manual text and download media when native
TDLib and app credentials are configured.

Missing:

- production parity;
- provider command coverage;
- robust live sync lifecycle;
- provider search;
- provider folder state;
- realtime event contracts.

### §7 Bot runtime — 10%

Bot account and token references exist. Live Bot API runtime is missing and must
remain blocked until a separate runtime slice exists.

### §8 Dialog/chat list — 55%

Chat projection and chat list endpoint exist. Frontend has virtualized chat list
with filters and metadata badges.

Missing:

- unread counters;
- pinned chats provider parity;
- archived chats;
- folders/chat lists;
- mute state;
- provider-side search;
- members/participants inspector.

### §9 Private chats — 60%

Private chat kind exists and selected history sync works.

Missing:

- full provider parity;
- read/unread sync;
- typing indicators;
- reply/reaction/edit/delete lifecycle.

### §10 Groups/supergroups/channels — 35%

`group` and `channel` chat kinds exist. TDLib supergroups are collapsed into
`group` unless channel-specific.

Missing:

- first-class supergroup metadata;
- participants;
- permissions;
- admin actions;
- join/leave lifecycle;
- channel post/admin state;
- topic-enabled forum support.

### §11 Topics/forums — 0%

No topic table/API/projection found.

Required:

- topic identity;
- topic-scoped timeline;
- topic reply model;
- topic unread/pinned state;
- provider sync.

### §12 Message ingestion/projection — 85%

Fixture and TDLib messages become append-only raw records and
`communication_messages`. TDLib media messages may have empty text while
preserving raw payload.

Missing:

- dedicated reply fields;
- reaction projection;
- edit version projection;
- delete/tombstone projection;
- forward attribution;
- topic identity.

### §13 Message lifecycle commands — 20%

Manual text send exists. Redacted provider-write audit exists.

Missing:

- edit;
- delete;
- restore visibility;
- mark read/unread;
- provider state sync;
- durable command queue;
- retries;
- per-target result rows.

### §14 Replies/forwards/pins — 5%

Current UI may derive pinned metadata opportunistically. No provider parity or
projection schema is durable.

Missing:

- reply target;
- reply graph;
- forward attribution;
- forward chains;
- pin/unpin command;
- pin sync.

### §15 Reactions — 0%

No add/remove/sync reaction API, projection or realtime contract found.

Telegram reactions should be modeled as communication events, not decorative UI
sprinkles, because apparently tiny emoji are now enterprise state. Naturally.

### §16 Message history/versioning — 25%

Selected history sync and older pagination exist.

Missing:

- observed edit versions;
- diffs;
- delete history;
- tombstones;
- restore visibility;
- timeline history view.

### §17 Media metadata/download — 45%

TDLib raw media metadata can be retained. Completed downloads persist local blobs
and attachment rows.

Missing:

- upload/send media;
- media gallery;
- provider media search;
- rich preview;
- persisted preview artifacts;
- voice/video recording;
- stickers/GIF/video notes.

### §18 Attachments preview/search/dedup — 25%

Downloaded media uses existing scanner/blob boundary. Shared Communication
attachment APIs may expose rows once downloaded.

Missing:

- Telegram-specific preview surface;
- Telegram media search;
- Telegram duplicate UX;
- scanner-backed clean verdicts;
- dedicated media viewer.

### §19 Voice/video messages — 10%

Some voice metadata may be retained from TDLib payloads.

Missing:

- voice playback UX;
- voice recording;
- voice send;
- transcription pipeline;
- video note support;
- audio/video permissions.

### §20 Calls/transcripts — 35%

Call metadata and fixture transcripts exist.

Missing:

- live call control;
- audio capture;
- device selection;
- real STT;
- call actions;
- visible permission flow.

Hidden recording remains unsupported.

### §21 Search — 30%

Local UI chat/message filtering exists. Shared Communication search can include
Telegram-projected messages by channel kind.

Missing:

- provider-side Telegram search;
- dedicated dialog search API;
- media search;
- topic/member search;
- saved Telegram-specific searches.

### §22 Realtime — 15%

Generic WebSocket/SSE/long-poll transports exist.

Missing:

- `telegram.message.created`;
- `telegram.message.edited`;
- `telegram.message.deleted`;
- `telegram.reaction.changed`;
- `telegram.sync.*`;
- `telegram.media.downloaded`;
- frontend cache patch handlers.

### §23 AI assistance — 20%

Projected Telegram messages can feed shared candidates and engines.

Missing Telegram-specific UI/API:

- summary;
- translation;
- bilingual reply;
- task extraction review;
- note extraction;
- event extraction;
- persona extraction review;
- voice transcript summary.

### §24 Frontend workbench — 55%

Vue Telegram page has chat list, selected timeline, local filters/tabs, composer,
sync controls and media download action.

Missing:

- account setup drawer completion;
- rich inspector;
- media viewer;
- provider commands;
- reply/thread/topic UX;
- reactions;
- voice player;
- call panel;
- search UI parity.

### §25 Timeline/shared engine integration points — 35%

Projected messages and selected audit events exist. Candidate refresh paths are
present.

Missing:

- Telegram-specific timeline event contracts;
- first-class timeline feed;
- durable AI result lifecycle;
- source-evidence review surfaces.

### §26 Offline/outbox/export/proxy/session bundles — 0%

No durable Telegram offline command queue, retry state, export flows, proxy
profile persistence or session bundle import/export found.

### §27 Dialog management — 15%

Basic dialog list exists.

Missing:

- unread counters;
- typing indicators;
- pinned chats;
- archived chats;
- folders/chat lists;
- mute/unmute;
- saved messages special handling;
- participants/admin state.

### §28 Provider-write command model — 10%

Manual send is implemented. No general durable command model exists.

Required:

- idempotency;
- retries;
- audit;
- capability decisions;
- per-message command results;
- sync/realtime command state;
- destructive action confirmation.

### §29 Privacy/security/capability UX — 30%

Secret refs and redacted audit exist.

Missing:

- detailed capability UI;
- TDLib health diagnostics;
- proxy/session safety UX;
- scanner-backed clean verdicts;
- call/recording permission model;
- provider-write risk explanation.

### §30 Documentation and audit set — 100%

Current documentation set exists and is aligned to the Mail-style audit pattern.

## Recommended next implementation order

1. Detailed capability matrix.
2. Telegram realtime event contracts.
3. Message lifecycle schema: versions/tombstones/replies/forwards/reactions.
4. Provider command/outbox model beyond send.
5. Dialog management: unread, pinned, archive, mute, folders.
6. Media gallery/search/preview.
7. Telegram-specific AI surfaces.
8. Voice/calls with explicit native permission boundary.
