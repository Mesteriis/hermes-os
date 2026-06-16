# Telegram Implementation Status

Статус на 2026-06-16.

Проценты ниже описывают coverage Telegram Channel only. Они не являются оценкой
готовности Communications, Memory, Knowledge, Obligations, Decisions, Personas,
Organizations или Timeline.

## Summary Table

| § | Раздел | Статус | % |
|---|---|---|---:|
| 1 | Channel framing and ADR alignment | ✓ | 100 |
| 2 | Provider/account model | ◐ | 74 |
| 3 | Secret references and host-vault boundary | ✓ | 90 |
| 4 | Capability contract | ◐ | 80 |
| 5 | Fixture runtime | ✓ | 100 |
| 6 | TDLib QR user runtime | ◐ | 65 |
| 7 | Bot runtime | ✗ | 10 |
| 8 | Dialog/chat list | ◐ | 64 |
| 9 | Private chats | ◐ | 60 |
| 10 | Groups/supergroups/channels | ◐ | 35 |
| 11 | Topics/forums | ⚠ | 60 |
| 12 | Message ingestion/projection | ✓ | 85 |
| 13 | Message lifecycle commands | ◐ | 48 |
| 14 | Replies/forwards/pins | ◐ | 38 |
| 15 | Reactions | ◐ | 43 |
| 16 | Message history/versioning | ◐ | 55 |
| 17 | Media metadata/download | ◐ | 60 |
| 18 | Attachments preview/search/dedup | ◐ | 38 |
| 19 | Voice/video messages | ◐ | 18 |
| 20 | Calls/transcripts | ◐ | 43 |
| 21 | Search | ◐ | 68 |
| 22 | Realtime | ◐ | 74 |
| 23 | AI assistance | ◐ | 20 |
| 24 | Frontend workbench | ◐ | 70 |
| 25 | Timeline/shared engine integration points | ◐ | 35 |
| 26 | Offline/outbox/export/proxy/session bundles | ✗ | 0 |
| 27 | Dialog management: unread/pinned/archive/mute/folders | ◐ | 58 |
| 28 | Provider-write command model | ◐ | 35 |
| 29 | Privacy/security/capability UX | ◐ | 50 |
| 30 | Documentation and audit set | ✓ | 100 |

## Legend

- ✓ — реализовано для текущего scope.
- ◐ — есть foundation или частичная реализация, но production scope не закрыт.
- ✗ — durable implementation не найден в текущем репозитории.

## Details

### §2 Provider/account model — 74%

`telegram_user` and `telegram_bot` provider kinds exist. Multiple accounts are
supported. Account lifecycle can list, logout and remove while preserving
evidence.

Frontend now also exposes a local account manager in the Telegram inspector
`About` tab, covering account setup plus logout/remove lifecycle actions for
existing account records. That same surface now also exposes the real TDLib QR
login flow (`start/status/password/cancel`) and can apply suggested QR-authorized
account metadata back into the local setup form before save. The same inspector
surface now renders the selected account capability payload as a read-only
matrix with operation status, action class, reason, confirmation and closure
gate flags.

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

### §4 Capability contract — 78%

`/api/v1/telegram/capabilities` and
`/api/v1/telegram/accounts/{account_id}/capabilities` now return a detailed
per-operation matrix with:

- `operation`;
- `category`;
- `status`;
- `action_class`;
- `reason`;
- `confirmation_required`;
- `closure_gate`.

The account-scoped route now also returns `account_scope` metadata with:

- `account_id`;
- `provider_kind`;
- `runtime_kind`;
- `lifecycle_state`.

Current account-aware shaping covers:

- user vs bot runtime differences;
- bot QR/TDLib operations marked unsupported;
- account lifecycle (`active` / `logged_out` / `removed`) blocking sync/write/runtime operations where applicable;
- selected-account capability consumption in the Telegram workbench with global fallback;
- explicit inspector rendering of the selected-account capability matrix for operator review.

Covered groups:

- account lifecycle;
- runtime and QR auth;
- sync;
- dialogs;
- messages read/write;
- replies/forwards;
- reactions;
- topics;
- media;
- voice/calls;
- search;
- realtime;
- automation;
- AI.

Remaining gaps:

- provider-runtime-aware degradation for more write commands;
- provider/local parity signaling for more dialog and message actions;
- explicit export/session/proxy diagnostics beyond the current TDLib/runtime health payload.

### §5 Fixture runtime — 100%

Fixture runtime is deterministic and should remain the validation foundation for
CI/local testing, especially when TDLib is unavailable. Human civilization has
apparently decided native dependencies should be everyone’s problem forever.

### §6 TDLib QR runtime — 65%

QR login start/status/password/cancel exists. `tdlib_qr_authorized` runtime can
start actor, sync chats/history, send manual text and download media when native
TDLib and app credentials are configured. The existing
`GET /api/v1/telegram/runtime/status?account_id=` payload now also exposes
runtime diagnostics for inspector/capability UX, including:

- configured TDLib library path;
- TDLib probe availability and error text;
- split `telegram_api_id` / `telegram_api_hash` readiness flags;
- derived runtime blockers for TDLib and live-runtime prerequisites.

Missing:

- production parity;
- provider command coverage;
- robust live sync lifecycle;
- provider search;
- provider folder state.

### §7 Bot runtime — 10%

Bot account and token references exist. Live Bot API runtime is missing and must
remain blocked until a separate runtime slice exists.

### §8 Dialog/chat list — 64%

Chat projection and chat list endpoint exist. Frontend has virtualized chat list
with filters and metadata badges. Selected chat workbench now exposes local
pin/archive/mute/read toggles through the verified dialog-action routes.
Unread counters are now recomputed from projected received messages and stored
in chat metadata. Inspector rail now reads projected chat detail and top
message senders for the selected chat. `GET /api/v1/telegram/folders?account_id=`
now exposes projection-backed folder/chat-list filters from
`telegram_chats.metadata.folder_name`, and the Telegram action rail now consumes
that dedicated query instead of deriving folder groups only from the currently
loaded chat list, scoped to the selected account when account context exists.
Unread mention state is now also visible in the primary workbench: chat rows can
render mention badges from projected `mention_count`, and the selected thread
header surfaces unread and mention summary chips from the same projection state.
The inspector `Members` tab now also supports local search over the projected
member roster returned by `GET /api/v1/telegram/chats/{chat_id}/members`.

Missing:

- pinned chats provider parity;
- archived/provider sync parity;
- provider-backed folder sync and folder mutations;
- mute provider sync;
- provider unread/read sync parity;
- provider-identity mention targeting and provider mention sync parity;
- provider-side search;
- member roles/permissions and provider-synced participant state.

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

### §11 Topics/forums — 60%

Foundation in place: `telegram_topics` table (migration 0086), `TelegramTopic` model,
`topics.rs` client (upsert/list/get), three API routes
(`GET /chats/{id}/topics`, `GET /topics/{id}`, `GET /topics/{id}/messages`),
frontend types, TanStack Query hooks (`useTelegramTopicsQuery`,
`useTelegramTopicMessagesQuery`), and a Topics tab in `TelegramThreadSideSections`.

Live TDLib sync is now wired: `GetForumTopics` runtime command, `actor_get_forum_topics`
actor handler, `request_actor_get_forum_topics` async wrapper, and `sync_forum_topics`
method on `TelegramRuntimeManager`. The `GET /chats/{id}/topics` API route calls
`sync_forum_topics` before serving the DB projection, so topics are auto-populated for
live TDLib accounts on first request. Falls back silently to DB rows for fixture accounts.

Missing:

- push-event listener for TDLib `updateForumTopicInfo` (live topic state changes);
- topic unread/pinned state driven by live events;
- provider write (create/close/reopen topic).

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

### §13 Message lifecycle commands — 45%

Manual text send exists. Edit/delete/restore lifecycle endpoints now persist:

- append-only message versions;
- tombstones;
- provider-write command rows;
- redacted audit for send/edit/delete.

Frontend now also reads lifecycle evidence in the per-message reference panel,
which consumes projected versions, tombstones and recent account command rows
for the current message/chat context. Inspector context now also exposes a
first-class recent-command audit panel for the selected account with local
current-chat filtering and search over durable command rows. The reference
panel itself now also supports local filtering across reply/forward/lifecycle
evidence rows, plus diff-style edit history summaries, tombstone visibility
state summaries and command capability/action metadata for the current message.

Remaining gaps:

- provider-side edit/delete/restore/reaction execution against TDLib/Bot runtime;
- provider mark read/unread parity beyond local projected chat state;
- retries/executing/failed transitions backed by a real outbox worker;
- per-target result rows beyond the current command record.

### §14 Replies/forwards/pins — 52%

Pinned-message metadata can already surface in UI, and the selected-chat pinned
tab is now backed by a dedicated projection query instead of only the currently
loaded message window. Reopening a pinned/search result can now focus a target
message in the thread even when that row is not part of the current 100-message
window, by carrying the projected message into the selected-chat view. Reply-
chain and forward-chain projection endpoints now also have thread UI through
per-message reference panels, and those panels render projected sender/text
summaries, can reopen local reply targets, reply sources and forward sources
into the selected thread, and now support local filtering across reply/forward/
history/command evidence plus richer forward source summaries. `POST
/api/v1/telegram/messages/{message_id}/pin` now also records local pin/unpin
state with durable command rows, redacted audit and thread-level pin/unpin
controls.

Reply command flow is now complete: `tdlib_send_reply_request` builder,
`ReplyMessage` runtime command variant, actor handler, async request function,
`POST /api/v1/telegram/messages/{message_id}/reply` API route, and full
frontend wiring — composer reply-preview banner, Reply action in message list,
and `useTelegramSendActions` composable routing send vs reply in TelegramPage.

Missing:

- deeper reply graph/thread traversal beyond single-hop projected reopen;
- provider-grade forward attribution beyond current origin metadata;
- provider-side forward command flow;
- pin sync.

### §15 Reactions — 43%

Local reaction add/remove/list endpoints exist. Reaction rows persist in
`telegram_message_reactions`, message list responses include reaction summary,
`telegram.reaction.changed` realtime events are emitted, and the Vue thread UI
can render summary chips plus capability-gated add/remove controls. The
per-message reference panel now also reads
`GET /api/v1/telegram/messages/{message_id}/reactions` for evidence-backed
reaction details.

The `telegram.reaction.changed` event now also patches the
`['telegram', 'message-reactions', messageId]` reaction detail cache surgically,
updating aggregate summary counts without requiring a full refetch.

Remaining gaps:

- provider-side reaction execution/sync;
- background/provider reconciliation;
- individual sender rows in reaction detail list not updated via event (requires full fetch).

### §16 Message history/versioning — 55%

Selected history sync and older pagination exist. Dedicated lifecycle history is
now present through:

- `telegram_message_versions`;
- `telegram_message_tombstones`;
- `GET /api/v1/telegram/messages/{message_id}/versions`;
- `GET /api/v1/telegram/messages/{message_id}/tombstones`.

The per-message reference panel now also renders that lifecycle evidence as a
more useful local history surface instead of a raw list: edit versions show
relative text-length deltas between captured bodies, tombstones show provider
delete vs local visibility state, and related commands show action class,
capability state and confirmation decision.

Remaining gaps:

- provider-observed diff metadata beyond the current local edit payload;
- timeline/history UI surfaces;
- provider delete reconciliation.

### §17 Media metadata/download — 60%

TDLib raw media metadata can be retained. Completed downloads persist local blobs
and attachment rows. The Telegram files tab now also loads a chat-scoped media
gallery from `GET /api/v1/telegram/search/media`, and downloaded media can be
opened in a dedicated read-only viewer. The files tab now merges query-backed
media-search rows with loaded attachment hints so older media outside the
current message window can still retain TDLib/local download metadata when that
projection data already exists. Completed downloads now also refresh
the projected Telegram message attachment metadata (`download_state`,
`local_path`, `attachment_id`, `tdlib_file_id`) and emit a
`telegram.media.downloaded` event with sanitized projected message/chat
snapshots for realtime cache patching.

Missing:

- upload/send media;
- richer gallery grouping and non-local preview parity;
- provider media search;
- persisted preview artifacts;
- voice/video recording;
- stickers/GIF/video notes.

### §18 Attachments preview/search/dedup — 38%

Downloaded media uses existing scanner/blob boundary. Shared Communication
attachment APIs may expose rows once downloaded. Telegram workbench now has a
Telegram-scoped media gallery for the selected chat, and query-backed gallery
rows can preserve local blob/TDLib metadata so downloaded files outside the
current message window can still open in the dedicated preview/viewer surface
or trigger a new download.

Missing:

- preview parity for remote/not-yet-downloaded files;
- richer Telegram media search/filter UX;
- Telegram duplicate UX;
- scanner-backed clean verdicts;
- persisted preview artifacts.

### §19 Voice/video messages — 18%

Some voice metadata may be retained from TDLib payloads. The Telegram thread now
has a dedicated `Voice` tab that derives local voice/audio attachments from the
selected message history and renders inline playback cards for downloaded local
files with a download fallback for remote-only items.

Missing:

- voice recording;
- voice send;
- transcription pipeline;
- video note support;
- audio/video permissions;
- richer waveform/progress UX and provider-backed voice parity.

### §20 Calls/transcripts — 43%

Call metadata and fixture transcripts exist. Telegram inspector context now also
renders a selected-account recent-calls card backed by `GET /api/v1/calls`, and
that same card can load a stored transcript for the selected projected call
through `GET /api/v1/calls/{call_id}/transcript`.

Missing:

- live call control;
- audio capture;
- device selection;
- real STT;
- call actions;
- visible permission flow.

Hidden recording remains unsupported.

### §21 Search — 68%

Local UI chat/message filtering exists. Shared Communication search can include
Telegram-projected messages by channel kind. Backend now also exposes:

- `GET /api/v1/telegram/chats/search`;
- `GET /api/v1/telegram/search/messages`;
- `GET /api/v1/telegram/search/media`.

Frontend now uses those routes for:

- header-driven dialog search results that can reopen projected chats;
- header-driven workspace Telegram message search;
- header-driven workspace media search within the selected chat, where media
  cards can now reopen the owning message context in the current dialog;
- selected-chat media gallery in the files tab, with query-backed attachment
  metadata merged into the local file cards for preview/download parity beyond
  the currently loaded message window.

Provider-side TDLib search is now wired: `SearchMessages` and `SearchChatMessages`
runtime command variants call TDLib `searchMessages`/`searchChatMessages`; results
are ingested via `search_provider_messages` on `TelegramRuntimeManager` before the
projection query — `GET /api/v1/telegram/search/messages` now automatically enriches
the DB projection with live TDLib results when `account_id` is present.

Remaining gaps:

- richer gallery filters;
- topic/member search;
- saved Telegram-specific searches.

### §22 Realtime — 80%

Generic WebSocket/SSE/long-poll transports exist. Telegram now emits typed
events into the canonical event flow and the dedicated realtime bus for:

- `telegram.sync.started`;
- `telegram.sync.progress`;
- `telegram.sync.completed`;
- `telegram.message.created`;
- `telegram.message.updated`;
- `telegram.message.deleted`;
- `telegram.message.visibility_restored`;
- `telegram.reaction.changed`;
- `telegram.command.status_changed`.

When PostgreSQL/event log is configured, these events are also appended to the
canonical event store, which gives Telegram parity with:

- WebSocket replay through `/api/events/ws`;
- SSE through `/api/events/stream`;
- long-poll/replay through `/api/v1/events`;
- offline recovery from stored event positions.

The Telegram page now relies on the shared frontend realtime bootstrap instead
of opening a second channel-scoped WebSocket, so replay cursor persistence and
offline recovery use the same WS/SSE/long-poll path as the rest of Hermes.
`telegram.message.created/updated/deleted/visibility_restored` payloads now
also carry a sanitized projected `message` snapshot plus deterministic
`telegram_chat_id` resolution, and when a projected dialog row exists they now
also carry a sanitized projected `chat` snapshot. Frontend cache patching can
therefore upsert message lists, pinned-message lists and message-search
results, and it can refresh matching chat list/detail caches for current
message events without waiting for a full query refetch.
Account-scoped `telegram.sync.*` and `telegram.command.status_changed` events
now also patch cached runtime status queries, so selected-chat inspector/runtime
surfaces can show the latest sync scope/status/count and the last observed
Telegram command outcome without a full reload. Current local dialog command
events now also carry a sanitized projected `chat` snapshot plus `action`,
which lets the frontend patch account-scoped chat lists and active chat-detail
caches immediately after pin/archive/mute/read/unread actions.

Frontend cache patching now also handles:

- `telegram.chat.pinned/archived/muted`: surgical metadata toggle on matching chat rows in list and detail caches;
- `telegram.chat.updated`: full chat snapshot upsert in list and detail;
- `telegram.command.status_changed`: status field update on the matching row in the commands list cache;
- `telegram.reaction.changed`: aggregate summary count patch on the reaction detail cache.

Remaining gaps:

- `telegram.chat.pinned/archived/muted/updated` events not yet emitted by backend routes (handlers defined, constants exist);
- Telegram media progress/retry event families beyond the completed-download event;
- individual sender rows in reaction detail cache not patched (requires full fetch);
- richer optimistic patching without follow-up query refresh.

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

### §24 Frontend workbench — 72%

Vue Telegram page has chat list, selected timeline, local filters/tabs, composer,
sync controls, media download action, workspace search, and selected-chat
pin/archive/mute actions. Inspector rail can now also surface account-scoped
runtime state together with the last sync progress and last command status
patched from realtime events. The action rail now also loads folder filters
through `GET /api/v1/telegram/folders` with selected-account scope and local
chat-derived fallback when that query is unavailable. Per-message reference
panels now also surface lifecycle evidence from versions, tombstones, reactions
and recent command rows alongside reply/forward references. The inspector
`Context` tab now also contains a first-class recent-command audit panel backed
by `GET /api/v1/telegram/commands` plus a first-class calls panel backed by
`GET /api/v1/calls`, including local projected-call search before loading
read-only transcripts. The `About` tab contains a local account manager for
setup/logout/remove flows wired to the existing account routes. The composer
send dropdown now also surfaces Telegram automation dry-run through the
existing policy/template routes, including enabled-policy selection for the
selected chat, required-variable inputs and rendered preview/hash output before
any live send.

Missing:

- account setup drawer completion;
- rich inspector;
- media viewer;
- provider-parity command execution;
- reply/thread/topic UX;
- reactions;
- voice player;
- dialog/provider search parity.

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

### §27 Dialog management — 58%

Basic dialog list exists. Local metadata-backed pin/archive/mute routes now
exist for projected chats, capability contract now marks them as degraded local
writes rather than blocked provider writes, and the selected-chat workbench can
toggle them through UI. Realtime command-status payloads for those local dialog
actions now include projected chat snapshots, so chat list/detail state can
update across active clients without waiting for a manual reload.

Missing:

- provider-synced pin/archive/mute execution;
- provider unread/read parity;
- typing indicators;
- provider-backed folder sync and folder mutations;
- saved messages special handling;
- participants/admin state.

### §28 Provider-write command model — 35%

Manual send is implemented. Telegram now also persists durable command rows for
message lifecycle actions, current local message pin/unpin and current local
dialog actions (`pin/unpin/archive/unarchive/mute/unmute`), with:

- command kind;
- idempotency key;
- capability state;
- confirmation decision;
- retry counters;
- result payload slot;
- actor/audit metadata.

Required:

- command executor/outbox worker;
- retries backed by runtime failures;
- per-message/provider result detail;
- richer command status UI.

### §29 Privacy/security/capability UX — 50%

Secret refs and redacted audit exist.

Capability-gated message actions are now visible in UI with blocked/degraded
reasons instead of silently disappearing. The inspector `About` tab now also
surfaces the selected account capability contract as a read-only matrix so
blocked/degraded states, confirmation requirements and closure gates are
inspectable without developer tools.

Missing:

- TDLib health diagnostics;
- proxy/session safety UX;
- scanner-backed clean verdicts;
- call/recording permission model;
- fuller provider-write risk explanation across the entire workbench.

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
