# Telegram Implementation Status — Details Extended (§17-§30)

См. также: [Status Index](../status.md) и [Details Core](details-core.md).

### §17 Media metadata/download/upload — 68%

TDLib raw media metadata can be retained. Completed downloads persist local blobs
and attachment rows. The Telegram files tab now also loads a chat-scoped media
gallery from `GET /api/v1/telegram/search/media`, and downloaded media can be
opened in a dedicated read-only viewer. The files tab now merges query-backed
media-search rows with loaded attachment hints so older media outside the
current message window can still retain TDLib/local download metadata when that
projection data already exists. Media download requests now emit replayable
started/progress/failed events, and completed downloads refresh the projected
Telegram message attachment metadata (`download_state`, `local_path`,
`attachment_id`, `tdlib_file_id`) before emitting `telegram.media.downloaded`
with sanitized projected message/chat snapshots for realtime cache patching.
TDLib sticker, animation/GIF and video-note messages now also project remote
attachment metadata from raw message payloads, so the existing files tab and
media viewer can list/download/preview them without a separate media model.
TDLib messages with `media_album_id` now also project `media_album_id` and
`media_album_key` into Communication message metadata. The Files tab groups
loaded selected-chat messages by this key and can reopen the first message in a
read-only album group.
The Files and Voice tabs now also render a current-chat download queue derived
from projected attachment readiness, so in-flight downloads and failed retries
stay visible in the thread surface and can be retried through the existing
download action without polling.
Provider-side media upload now has a backend command path: generic local
attachment import writes shared blob/import metadata, Telegram upload accepts
only `attachment_id` or `blob_id`, records redacted audit, queues `send_media`,
and the outbox executor resolves local blobs into TDLib `sendMessage` media
requests for photo, video, document, audio, voice note, sticker and
animation/GIF. Command completion is still reserved for provider-observed TDLib
message snapshots.

Missing:

- Bot API media send;
- byte-level/provider-native upload progress;
- richer gallery grouping and non-local preview parity;
- provider media search;
- persisted preview artifacts;
- voice/video recording;
- provider album sync beyond loaded selected-chat history;
- album-aware media send/upload.

### §18 Attachments preview/search/dedup — 43%

Downloaded media uses existing scanner/blob boundary. Shared Communication
attachment APIs may expose rows once downloaded. Telegram workbench now has a
Telegram-scoped media gallery for the selected chat, and query-backed gallery
rows can preserve local blob/TDLib metadata so downloaded files outside the
current message window can still open in the dedicated preview/viewer surface
or trigger a new download.
Projected TDLib sticker, animation and video-note metadata now flows through the
same attachment hint path as files/photos/videos.
Workspace search media cards now also count media hits in the overall search
result total and surface projected preview/download readiness from
`local_path`, `tdlib_file_id`, `provider_attachment_id` and `download_state`.
Downloaded Telegram attachments with a projected Communication `attachment_id`
can now use the shared safe attachment preview route from the Telegram media
viewer for text/image previews; that route keeps the existing local-blob,
scan-status and size-limit checks.
Local composer/import attachments now have a provider-neutral import table
(`communication_attachment_imports`) referencing shared local blobs before a
provider message exists. Telegram upload commands reuse those imported blobs
instead of uploading bytes directly from UI.

Missing:

- preview parity for remote/not-yet-downloaded files;
- richer Telegram media search/filter UX;
- Telegram duplicate UX;
- scanner-backed clean verdicts;
- persisted preview artifacts.

2026-06-17 pass note: media search UI gained read-only projection readiness
labels for already downloaded files, TDLib-downloadable remote files and
metadata-only hits. This does not add a Telegram-specific preview bytes route,
Bot media send, provider-side search or persisted preview artifacts.
2026-06-17 pass note: the Telegram media viewer now reuses the shared
Communication attachment preview endpoint for downloaded projected attachments
with `attachment_id`. This improves local text/image preview without bypassing
scanner/local blob boundaries or adding a Telegram-specific preview route.
2026-06-17 pass note: the Telegram media viewer now exposes a direct Download
action for remote-only attachments that carry `tdlib_file_id`, reusing the
existing TDLib download mutation and scanner/blob persistence path from the
Files tab. This improves viewer navigation for not-yet-downloaded files but
does not add remote preview bytes or persisted preview artifacts.
2026-06-17 pass note: the Telegram Files tab now embeds a Telegram-scoped
wrapper over the shared Communication attachment search panel. Searches are
limited by the selected Telegram account id and cover downloaded/persisted
attachment metadata only; this does not add provider-side remote attachment
search or durable preview artifact generation.
2026-06-17 pass note: Telegram search now reuses the shared Communication
saved-search strip from the thread surface with selected-account scope and
`channel_kind=telegram`. This adds create/select/delete UI for Telegram saved
searches without introducing a Telegram-specific saved-search route, scheduler
or provider-side saved-search execution model.
2026-06-17 pass note: `GET /api/v1/telegram/search/media` now attempts TDLib
provider message search when both `q` and `account_id` are present, then filters
the refreshed Communication projection for media rows. The response reports
`source`, `provider_search_attempted` and `provider_search_error` so clients can
distinguish provider refresh from projection-only fallback. This does not add
remote preview bytes or richer provider media-type filters.
2026-06-17 pass note: the Telegram workspace media results panel now renders
that media-search provider/projection source state when media hits are present.
`TelegramPage.vue` stayed under the 700-line hard limit by moving static chat
filter tab construction into a Telegram store helper. This is UI visibility for
existing search metadata only; it does not add new provider media commands.
2026-06-17 pass note: the composer attachment affordance now reads the selected
account capability contract and renders the `messages.send_media` status/reason
instead of a hardcoded disabled message. This is capability visibility only; it
does not add media upload/send or an attachment outbox.

### §19 Voice/video messages — 20%

Some voice metadata may be retained from TDLib payloads. The Telegram thread now
has a dedicated `Voice` tab that derives local voice/audio attachments from the
selected message history and renders inline playback cards for downloaded local
files with a download fallback for remote-only items.
Video-note TDLib payloads now project remote `video_note` attachment metadata
for download and viewer handoff, but recording/send/transcript flows remain
out of scope for this slice.

Missing:

- voice recording;
- voice send;
- transcription pipeline;
- video note support;
- audio/video permissions;
- richer waveform/progress UX and provider-backed voice parity.

2026-06-17 pass note: the composer microphone affordance now reads the selected
account capability contract and renders the `voice.record_send` status/reason
instead of a hardcoded disabled message. This does not add native audio capture,
voice send, transcription or provider-backed voice parity.

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

### §21 Search — 72%

Local UI chat/message filtering exists. Shared Communication search can include
Telegram-projected messages by channel kind. Backend now also exposes:

- `GET /api/v1/telegram/chats/search`;
- `GET /api/v1/telegram/search/messages`;
- `POST /api/v1/telegram/search/provider`.
- `GET /api/v1/telegram/search/media`.

Frontend now uses those routes for:

- header-driven dialog search results that can reopen projected chats;
- header-driven workspace Telegram message search;
- header-driven workspace media search within the selected chat, where media
  cards can now reopen the owning message context in the current dialog;
- selected-chat media gallery in the files tab, with query-backed attachment
  metadata merged into the local file cards for preview/download parity beyond
  the currently loaded message window.
The workspace search panel now also displays whether the current result set is
local projection search, fixture projection search, provider search with
projection refresh, provider fallback, or projection-only because the runtime is
unavailable.

Provider-side TDLib search is now wired: `SearchMessages` and `SearchChatMessages`
runtime command variants call TDLib `searchMessages`/`searchChatMessages`; results
are ingested via `search_provider_messages` on `TelegramRuntimeManager` before the
projection query. `GET /api/v1/telegram/search/messages` now automatically
enriches the DB projection with live TDLib results when `account_id` is present.
A dedicated `POST /api/v1/telegram/search/provider` route now exposes the same
search command path explicitly for capability/UX-aligned provider search calls.

2026-06-17 pass note: provider/local search source is now visible in the
workspace search results panel using existing runtime status and query behavior.
At that pass no saved-search persistence or new provider command was added;
Telegram saved-search UI now exists through the shared Communication
saved-search strip with `channel_kind=telegram`.

Remaining gaps:

- richer gallery filters;
- topic/member search;
- provider-side scheduled saved-search execution.

### §22 Realtime — 85%

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
- `telegram.chat.pinned`;
- `telegram.chat.archived`;
- `telegram.chat.muted`;
- `telegram.chat.updated`;
- `telegram.typing.changed`;
- `telegram.media.download.started`;
- `telegram.media.download.progress`;
- `telegram.media.download.failed`;
- `telegram.media.downloaded`;
- `telegram.topic.updated`;
- `telegram.command.status_changed`;
- `telegram.command.reconciled`.

When PostgreSQL/event log is configured, these events are also appended to the
canonical event store, which gives Telegram parity with:

- WebSocket replay through `/api/events/ws`;
- SSE through `/api/events/stream`;
- long-poll/replay through `/api/v1/events`;
- offline recovery from stored event positions.

The Telegram page now relies on the shared frontend realtime bootstrap instead
of opening a second channel-scoped WebSocket, so replay cursor persistence and
offline recovery use the same WS/SSE/long-poll path as the rest of Hermes.
The Telegram workbench now renders that shared transport state in its status
area, including degraded fallback/offline labels from `useRealtimeStatusStore`.
WebSocket `lagged` frames now surface replay-gap diagnostics with skipped-event
counts and trigger broad cache invalidation before manual reconnect/replay.
Runtime-started TDLib actors now also bridge unsolicited provider message
events: `updateNewMessage` ingests the observed snapshot and emits
`telegram.message.created`, `updateDeleteMessages` records an idempotent
`deleted_by_provider` tombstone and emits `telegram.message.deleted`, and
`updateMessageInteractionInfo` refreshes reaction summary metadata/sender rows
before emitting `telegram.reaction.changed`.
`updateMessageContent` and `updateMessageEdited` now also refresh projected
message text/metadata, record observed edit versions from provider content
updates and emit provider-origin `telegram.message.updated`.
`telegram.message.created/updated/deleted/visibility_restored` payloads now
also carry a sanitized projected `message` snapshot plus deterministic
`telegram_chat_id` resolution, and when a projected dialog row exists they now
also carry a sanitized projected `chat` snapshot. Frontend cache patching can
therefore upsert message lists, pinned-message lists and message-search
results, and it can refresh matching chat list/detail caches for current
message events without waiting for a full query refetch.
Account-scoped `telegram.sync.*`, `telegram.command.status_changed` and
`telegram.command.reconciled` events now also patch cached runtime status
queries, so selected-chat inspector/runtime surfaces can show the latest sync
scope/status/count and the last observed Telegram command outcome without a
full reload. Current local dialog command events now also carry a sanitized
projected `chat` snapshot plus `action`, which lets the frontend patch
account-scoped chat lists and active chat-detail caches immediately after
pin/archive/mute/read/unread actions.
Read/unread dialog routes also emit `telegram.chat.updated` after projected
unread counters are recomputed, carrying the same sanitized chat snapshot for
existing list/detail cache patch handlers.
Provider-observed TDLib `updateChatReadInbox` and
`updateChatUnreadMentionCount` updates now also feed `telegram.chat.updated`
through the runtime event bridge. Hermes resolves the projected chat, updates
`unread_count` / `mention_count` metadata plus provider-source counter evidence
and broadcasts a sanitized projected chat snapshot for the same frontend cache
patch path used by local dialog read/unread actions.
Typing indicators now have a reserved `telegram.typing.changed` event type and
TDLib `updateUserChatAction` parsing for provider evidence normalization.
Runtime-started TDLib actors now drain idle unsolicited `updateUserChatAction`
updates and forward sanitized typing snapshots through a runtime event bridge
that appends to `event_log` when available and broadcasts through the shared
realtime bus. Frontend realtime invalidation handles `telegram.typing.*`
without broad Telegram cache invalidation, and cache patching can now project
expiry-bounded active typing state into chat-list and selected-chat detail
caches for visible UI chips.
TDLib `updateForumTopicInfo` updates now follow the same actor event bridge:
Hermes resolves the projected chat, upserts the existing `telegram_topics`
row, appends/broadcasts `telegram.topic.updated` and patches cached topic
list/search queries before normal replay invalidation.
Selected-chat surfaces now also consume that cached runtime state directly: the
thread header renders realtime sync/command chips when the cached target matches
the current dialog, and inspector context now shows explicit sync/command
targets in addition to the existing summaries.
Dialog pin/archive/mute routes now also emit their dedicated
`telegram.chat.pinned`, `telegram.chat.archived` and `telegram.chat.muted`
events with `telegram_chat_id`, provider chat id, the changed flag and a
sanitized projected `chat` snapshot. These events align backend emission with
the existing surgical frontend chat list/detail patch handlers.
Provider-observed TDLib `updateChatNotificationSettings` and
`updateChatPosition` updates now also emit `telegram.chat.updated` with the
same projected chat snapshot in addition to their dedicated
`telegram.chat.muted` / `telegram.chat.pinned` / `telegram.chat.archived`
events, so generic chat-list/detail snapshot patching and command-reconciled
chat refresh stay aligned for provider-origin flag changes.
Media download requests now emit a replayable request lifecycle:
`telegram.media.download.started` before runtime dispatch,
`telegram.media.download.progress` when TDLib returns a non-completed file
snapshot, `telegram.media.download.failed` when the runtime rejects/fails the
request, and the existing `telegram.media.downloaded` event after completed
blob/attachment persistence and projected message metadata refresh.

Frontend cache patching now also handles:

- `telegram.chat.pinned/archived/muted`: surgical metadata toggle on matching chat rows in list and detail caches;
- `telegram.chat.updated`: full chat snapshot upsert in list and detail,
  currently emitted for local read/unread projection changes;
- `telegram.typing.changed`: expiry-bounded active typing metadata patch on
  matching chat list and chat detail caches;
- `telegram.media.*`: invalidation of Telegram message and media-search query families;
- `telegram.command.status_changed` / `telegram.command.reconciled`: status and reconciliation field updates on the matching row in the commands list cache;
- `telegram.reaction.changed`: aggregate summary count patch on the reaction detail cache.

Remaining gaps:

- richer explicit provider typing clear/absence semantics are still missing;
- richer Telegram media retry/dead-letter UX beyond replayable status events;
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

### §24 Frontend workbench — 76%

Vue Telegram page has chat list, selected timeline, local filters/tabs, composer,
sync controls, media download action, workspace search, and selected-chat
pin/archive/mute actions. Inspector rail can now also surface account-scoped
runtime state together with the last sync progress and last command status
patched from realtime events. The action rail now also loads folder filters
through `GET /api/v1/telegram/folders` with selected-account scope and local
chat-derived fallback when that query is unavailable. Per-message reference
panels now also surface lifecycle evidence from versions, tombstones, reactions
and recent command rows alongside reply/forward references. The same per-message
panel now also exposes sanitized raw source evidence and provenance through
`GET /api/v1/telegram/messages/{message_id}/raw`, and renders projected public
provider permalinks plus poll/location/venue/contact-card structured evidence
when those metadata keys are present. Source-metadata rendering now lives in a
dedicated `TelegramMessageSourceEvidencePanel`, keeping the main reference
panel focused on lifecycle, reply and forward orchestration. The inspector
`Context` tab now also contains a first-class recent-command audit panel backed
by `GET /api/v1/telegram/commands` plus a first-class calls panel backed by
`GET /api/v1/calls`, including local projected-call search before loading
read-only transcripts. The `About` tab contains a local account manager for
setup/logout/remove flows wired to the existing account routes. The composer
send dropdown now also surfaces Telegram automation dry-run through the
existing policy/template routes, including enabled-policy selection for the
selected chat, required-variable inputs and rendered preview/hash output before
any live send.
The frontend Telegram API layer is split between route-level calls in
`api/telegram.ts` and UI-friendly workspace helper orchestration in
`api/telegramWorkspace.ts`, preserving the rule that components use Query/
helpers instead of inline `fetch`.

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

### §26 Offline/outbox/export/proxy/session bundles — 30%

Telegram provider-write commands now have a durable outbox foundation:

- queued/retrying/executing/completed/failed/cancelled/dead-letter status model;
- due timestamps and atomic claim/lock fields;
- retry backoff scheduling;
- stale locked execution recovery;
- manual retry endpoint and UI control for failed/dead-letter rows;
- provider-observed reconciliation fields.
- media upload commands via local blob/import reuse.

Still missing: participant/admin command coverage, Bot API outbox runtime,
complete provider reconciliation workers for ACK-only actions, export flows,
proxy profile persistence and session bundle import/export.

### §27 Dialog management — 66%

Basic dialog list exists. Local metadata-backed pin/archive/mute routes now
exist for projected chats, capability contract now marks archive/mute/read
actions as provider writes when active TDLib execution is possible, and the
selected-chat workbench can toggle them through UI. Read/unread routes update
local projection immediately and enqueue provider-write commands that active
TDLib actors execute via `toggleChatIsMarkedAsUnread` for manual unread toggles
and `viewMessages(force_read=true)` for targeted mark-read requests;
archive/unarchive and mute/unmute commands now dispatch through
`addChatToList` and `setChatNotificationSettings`. Provider-observed TDLib chat
state now reconciles dialog `pin/unpin`, `archive/unarchive`, exact-shape
`mute/unmute`, and `mark_unread` commands into either `completed` or
`mismatch`, and realtime command-status payloads for those dialog actions now
include projected chat snapshots, so chat list/detail state can update across
active clients without waiting for a manual reload.

Missing:

- message-level read receipts and richer read-position history;
- richer typing clear/expiry semantics beyond the current TDLib typing
  projection;
- richer folder state parity beyond the current add/remove/reassign projection model;
- saved messages special handling;
- participant admin command state.

### §28 Provider-write command model — 68%

Manual send is implemented. Telegram now also persists durable command rows for
message lifecycle actions, current local message pin/unpin and current dialog
actions (`pin/unpin/archive/unarchive/mute/unmute/read/unread`), with:

- command kind;
- idempotency key;
- capability state;
- confirmation decision;
- retry counters;
- manual retry/dead-letter UI state in the command audit panel;
- due timestamps and execution locks;
- provider-observed reconciliation fields;
- result payload slot;
- actor/audit metadata.

The active TDLib command executor currently covers edit/delete/reaction,
message pin/unpin, forward, manual unread toggles, archive/unarchive,
mute/unmute, participant join/leave and media upload/send from local imported
blobs.
TDLib member sync now also reconciles matching self `join` commands from
provider roster presence and emits command status/reconciled events. TDLib
history sync can now reconcile matching self `join`/`leave` commands from
explicit provider service-message evidence.

Required:

- participant admin command execution;
- Bot API command execution;
- provider execution for admin-style commands;
- provider-observed reconciliation for remaining ACK-only write actions,
  especially silent/admin participant lifecycle changes;
- per-message/provider result detail beyond command-level state;
- richer command status UI.

2026-06-17 pass note: the command audit inspector now projects existing
`status`, `retry_count`, `max_retries`, `last_error`, `reconciliation_status`
and dead-letter fields into retry/dead-letter state, and can call
`POST /api/v1/telegram/commands/{command_id}/retry` for eligible failed or
dead-letter rows. Completion is now reserved for provider-observed state; ACK
dispatch-only commands remain `executing/awaiting_provider` until a later
reconciliation pass observes the provider state. Self `join` commands are now
an exception to the generic ACK-only backlog when TDLib member sync observes the
account's own active provider roster row. Self `leave` can now also reconcile
when TDLib history sync ingests an explicit `messageChatDeleteMember` event for
the selected account.

### §29 Privacy/security/capability UX — 50%

Secret refs and redacted audit exist.

Capability-gated message actions are now visible in UI with blocked/degraded
reasons instead of silently disappearing. The inspector `About` tab now also
surfaces the selected account capability contract as a read-only matrix so
blocked/degraded states, confirmation requirements and closure gates are
inspectable without developer tools. The composer now also consumes that
selected-account contract for media and voice disabled affordances, showing
`messages.send_media` and `voice.record_send` reasons in-place.

Missing:

- TDLib health diagnostics;
- proxy/session safety UX;
- scanner-backed clean verdicts;
- call/recording permission model;
- fuller provider-write risk explanation across the entire workbench.

### §30 Documentation and audit set — 100%

Current documentation set exists and is aligned to the Mail-style audit pattern.
