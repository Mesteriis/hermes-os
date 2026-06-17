# Telegram Implementation Status

Статус на 2026-06-17.

Проценты ниже описывают coverage Telegram Channel only. Они не являются оценкой
готовности Communications, Memory, Knowledge, Obligations, Decisions, Personas,
Organizations или Timeline.

## Summary Table

| § | Раздел | Статус | % |
|---|---|---|---:|
| 1 | Channel framing and ADR alignment | ✓ | 100 |
| 2 | Provider/account model | ◐ | 74 |
| 3 | Secret references and host-vault boundary | ✓ | 90 |
| 4 | Capability contract | ◐ | 83 |
| 5 | Fixture runtime | ✓ | 100 |
| 6 | TDLib QR user runtime | ◐ | 65 |
| 7 | Bot runtime | ✗ | 10 |
| 8 | Dialog/chat list | ◐ | 68 |
| 9 | Private chats | ◐ | 60 |
| 10 | Groups/supergroups/channels | ◐ | 45 |
| 11 | Topics/forums | ⚠ | 63 |
| 12 | Message ingestion/projection | ✓ | 88 |
| 13 | Message lifecycle commands | ◐ | 62 |
| 14 | Replies/forwards/pins | ◐ | 58 |
| 15 | Reactions | ◐ | 62 |
| 16 | Message history/versioning | ◐ | 55 |
| 17 | Media metadata/download/upload | ◐ | 68 |
| 18 | Attachments preview/search/dedup | ◐ | 43 |
| 19 | Voice/video messages | ◐ | 20 |
| 20 | Calls/transcripts | ◐ | 43 |
| 21 | Search | ◐ | 72 |
| 22 | Realtime | ◐ | 89 |
| 23 | AI assistance | ◐ | 20 |
| 24 | Frontend workbench | ◐ | 77 |
| 25 | Timeline/shared engine integration points | ◐ | 35 |
| 26 | Offline/outbox/export/proxy/session bundles | ◐ | 30 |
| 27 | Dialog management: unread/pinned/archive/mute/folders | ◐ | 68 |
| 28 | Provider-write command model | ◐ | 68 |
| 29 | Privacy/security/capability UX | ◐ | 50 |
| 30 | Documentation and audit set | ✓ | 100 |

## Pass Log

- 2026-06-16: capability contract aligned with the existing TDLib command
  executor for edit/delete/pin/reaction provider writes; Telegram and WhatsApp
  capability support were split out of the former messaging integration helper
  so touched production modules remain below the 700-line architecture limit.
- 2026-06-16: dialog read/unread commands moved from local-only command rows to
  provider-write rows; active TDLib actors now execute queued manual unread
  toggles through `toggleChatIsMarkedAsUnread`, while message-level read receipt
  parity and provider-observed reconciliation remain open.
- 2026-06-16: dialog archive/unarchive and mute/unmute commands were wired into
  the active TDLib command executor through `addChatToList` and
  `setChatNotificationSettings`; capability action classes now report
  provider-write semantics, while provider-observed folder/notification
  reconciliation remains open.
- 2026-06-16: dialog pin/archive/mute routes now emit typed
  `telegram.chat.pinned`, `telegram.chat.archived` and `telegram.chat.muted`
  events with sanitized projected chat snapshots, matching existing frontend
  cache patch handlers.
- 2026-06-16: dialog read/unread routes now emit `telegram.chat.updated`
  events with sanitized projected chat snapshots after unread counter
  recomputation, so existing frontend cache patch handlers can refresh chat
  list/detail projections without full reloads.
- 2026-06-16: media download requests now emit
  `telegram.media.download.started`, `telegram.media.download.progress` and
  `telegram.media.download.failed` around the existing completed-download
  `telegram.media.downloaded` event, preserving replayable status evidence for
  failed fixture/live-blocked attempts and in-flight TDLib snapshots.
- 2026-06-16: `GET /api/v1/telegram/messages/{message_id}/raw` now exposes the
  append-only raw provider evidence row for Telegram-projected messages with
  recursive secret-key redaction, preserving TDLib/source payload metadata for
  local inspection.
- 2026-06-16: the Telegram thread reference panel now consumes the raw evidence
  route through a dedicated TanStack Query hook and source-evidence component,
  showing sanitized payload and provenance without component-level fetch or
  manual cache logic.
- 2026-06-16: message ingestion now preserves synced public chat usernames
  during metadata-only chat upserts and projects `https://t.me/<username>/<id>`
  message links for public username-backed messages; the thread reference panel
  renders that provider permalink when present.
- 2026-06-16: TDLib message ingestion now derives remote attachment metadata for
  `messageSticker`, `messageAnimation` and `messageVideoNote` payloads from raw
  source evidence; existing files/media viewer surfaces can list, download and
  preview those attachments after local download.
- 2026-06-16: TDLib `messagePoll`, `messageLocation`, `messageVenue` and
  `messageContact` payloads now project read-only structured evidence into
  Telegram message metadata; the thread reference panel renders poll,
  location/venue and contact-card evidence without creating Contact/Persona
  lifecycle.
- 2026-06-16: TDLib chat sync now preserves distinct supergroup identity in
  `telegram_chats.metadata` (`tdlib_chat_type`, `tdlib_supergroup_id`,
  `is_supergroup`, `is_channel_supergroup`, `is_forum`) while keeping the
  public chat kind compatible as `group`/`channel`; the Telegram inspector
  `About` tab renders those provider-specific fields as read-only projection
  evidence.
- 2026-06-16: typing indicators now have a reserved realtime contract
  (`telegram.typing.changed`), a TDLib `updateUserChatAction` parser and
  frontend realtime invalidation handling.
- 2026-06-17: runtime-started TDLib actors now drain unsolicited
  `updateUserChatAction` updates while idle and forward sanitized typing
  snapshots through a runtime event bridge that appends/broadcasts
  `telegram.typing.changed`.
- 2026-06-17: TDLib actors created implicitly by sync, search, send, media
  download and topic routes now receive the same runtime event bridge context,
  so typing events are not limited to explicit runtime start/restart flows.
  Richer provider clear semantics remain open.
- 2026-06-17: frontend realtime cache patching now applies
  `telegram.typing.changed` payloads to `telegram_chats.metadata.active_typing`,
  adds a short `expires_at` boundary, and the chat list/thread header render
  only non-expired projected typing state without component-level fetches or a
  Telegram-specific realtime transport.
- 2026-06-16: TDLib message ingestion now projects `media_album_id` and
  `media_album_key` from raw source evidence, and the Telegram Files tab groups
  loaded selected-chat messages into read-only media album cards without adding
  upload/send behavior.
- 2026-06-16: TDLib chat sync now preserves boolean `chat.permissions` fields
  in `telegram_chats.metadata.tdlib_permissions`, and the Telegram inspector
  `About` tab renders a read-only permissions summary while member roles/admin
  lifecycle remain out of scope for this slice.
- 2026-06-16: TDLib join/leave service messages now project read-only
  `telegram_join_leave` structured evidence for add-members, join-by-link,
  join-by-request and delete-member events; the thread reference panel renders
  that evidence without adding provider join/leave commands.
- 2026-06-16: Frontend Telegram SRP hygiene pass reduced two existing
  700-line Telegram files below the configured line-count failure threshold
  without changing runtime behavior.
- 2026-06-16: TDLib owner private chats now project Saved Messages metadata
  (`is_saved_messages`, `saved_messages_source`, `tdlib_private_user_id`) when
  the private `user_id` matches the selected account Telegram user id; the chat
  list and thread header render this as read-only dialog evidence.
- 2026-06-16: TDLib message ingestion now projects provider-origin aggregate
  emoji reaction summaries from `interaction_info.reactions` into
  `message.metadata.reaction_summary`, allowing the existing thread reaction
  chips to render source-backed provider counts without creating synthetic
  sender rows.
- 2026-06-16: TDLib message ingestion now also upserts sender-level emoji
  reaction rows from `interaction_info.reactions.recent_reactions` when TDLib
  supplies concrete `messageSender*` evidence, preserving provider provenance
  without creating command/audit rows.
- 2026-06-17: TDLib unsolicited runtime events now flow through the Telegram
  actor event bridge for `updateNewMessage`, `updateDeleteMessages` and
  `updateMessageInteractionInfo`. Provider-origin `telegram.message.created`,
  `telegram.message.deleted` and `telegram.reaction.changed` now append to the
  canonical event log/realtime bus when the runtime is active, provider delete
  events record an idempotent `deleted_by_provider` tombstone and can
  reconcile matching delete commands, and live interaction updates now refresh
  reaction summary metadata while reconciling matching self `react` /
  `unreact` commands from chosen emoji state.
- 2026-06-17: TDLib unsolicited `updateMessageContent` and
  `updateMessageEdited` now also flow through the actor event bridge. Hermes
  refreshes the projected message body/metadata, records append-only observed
  edit versions from provider content updates, emits provider-origin
  `telegram.message.updated`, and can now reconcile matching edit commands
  from provider-observed body text.
- 2026-06-17: TDLib unsolicited `updateMessageIsPinned` now also flows through
  the actor event bridge. Hermes refreshes projected message `is_pinned`
  metadata, emits provider-origin `telegram.message.updated`, and reconciles
  message-targeted `pin` commands from provider-observed pinned state without
  conflating them with dialog pin/unpin reconciliation.
- 2026-06-16: TDLib custom reaction aggregate counts are now preserved as
  `message.metadata.reaction_summary.custom_reactions`, keeping source evidence
  without faking them into the emoji-only `reaction_emoji` contract.
- 2026-06-16: the Telegram thread reference panel now renders TDLib custom
  reaction aggregate metadata as read-only provider evidence, without adding
  fake custom reaction write controls or rows.
- 2026-06-16: the Telegram thread reference panel source-evidence rendering was
  split into `TelegramMessageSourceEvidencePanel` plus a pure metadata evidence
  helper, reducing the main lifecycle/reference panel from 697 to 506 lines and
  preserving the no-inline-fetch/TanStack Query boundary.
- 2026-06-16: TDLib message metadata derivation for mentions, public links,
  albums, attachment hints and structured source evidence was extracted from
  `messages/ingestion.rs` into `messages/message_metadata.rs`, reducing the
  ingestion orchestration module from 693 to 160 lines without changing the
  Communication projection contract.
- 2026-06-16: frontend Telegram route calls and UI-friendly workspace wrappers
  were split so `api/telegram.ts` stays route-focused and dropped from 699 to
  419 lines, while `api/telegramWorkspace.ts` owns workspace helper
  orchestration without component-level fetches or manual cache behavior.
- 2026-06-16: Telegram thread reaction rendering, capability gating and emoji
  picker behavior were extracted from `TelegramMessageList.vue` into
  `TelegramMessageReactions.vue`, keeping the message list below the SRP line
  threshold without changing the existing reaction contract.
- 2026-06-16: Telegram message reaction API handlers were extracted from
  `api/messages.rs` into `api/messages/reactions.rs`, keeping the message route
  module below the 700-line architecture threshold without changing routes,
  audit, provider-write command rows or realtime event payloads.
- 2026-06-16: `POST /api/v1/telegram/runtime/stop` now stops the selected
  account runtime actor idempotently, returns the refreshed runtime status,
  records local API audit evidence and appears in the detailed capability
  contract as `runtime.stop`.
- 2026-06-16: frontend runtime controls now expose `Stop Runtime` in the
  Telegram action rail, call `POST /api/v1/telegram/runtime/stop` through
  TanStack Query and invalidate runtime/account caches without component-level
  fetches or manual cache state.
- 2026-06-16: `POST /api/v1/telegram/runtime/restart` now performs a controlled
  account-scoped stop/start, records `telegram.runtime.restart` audit metadata,
  appears in the capability contract as `runtime.restart`, and is wired to a
  `Restart Runtime` action rail control. Runtime TanStack Query hooks were
  split into `useTelegramRuntimeQuery.ts` to keep `useTelegramQuery.ts` below
  the SRP line threshold.
- 2026-06-17: Telegram workbench status messages now surface the shared
  realtime transport state from `useRealtimeStatusStore`, so WS/SSE/long-poll
  fallback and replay/offline recovery state is visible in the channel UI
  without adding a Telegram-specific socket, fetch path or manual cache layer.
- 2026-06-17: TDLib `updateForumTopicInfo` updates now parse into sanitized
  runtime topic snapshots, upsert the existing `telegram_topics` projection and
  emit `telegram.topic.updated`; frontend realtime patching updates cached
  topic list/search queries and replay invalidates topic caches without adding
  topic write commands.
- 2026-06-17: TDLib `updateChatReadInbox` and
  `updateChatUnreadMentionCount` updates now parse into sanitized runtime
  unread snapshots, update projected chat metadata counters and emit
  `telegram.chat.updated` with a projected chat snapshot; true message-level
  read receipts remain open.
- 2026-06-17: Telegram provider-write command rows were upgraded into a
  durable outbox foundation with `dead_letter`, due timestamps, lock/claim
  fields, provider-observed reconciliation fields, stale locked execution
  recovery, exponential backoff scheduling, manual retry API/UI, projection
  refresh for TDLib returned send/reply/forward snapshots, and
  `telegram.command.reconciled` events for provider-observed completion.
  Admin commands, Bot API runtime and full provider reconciliation remain open.
- 2026-06-17: Communication attachment import and Telegram provider-side media
  upload now use the existing provider command path: local files are imported
  into shared blob storage through `POST /api/v1/communications/attachments/import`,
  `POST /api/v1/telegram/media/upload` queues a `send_media` outbox command from
  `attachment_id`/`blob_id`, TDLib builders cover photo/video/document/audio/
  voice/sticker/animation, upload started/status/completed/failed events are
  emitted, malicious local imports are rejected before command insertion, and
  the Vue composer now routes selected local files through the import/upload
  command path. Bot API media send, album-aware send and richer progress/cache
  UX remain open.
- 2026-06-17: Telegram participants gained a provider-backed projection slice
  for TDLib supergroups/channels. Migration 0089 adds
  `telegram_chat_participants`; `POST /api/v1/telegram/chats/{chat_id}/members/sync`
  fetches TDLib `getSupergroupMembers`; `GET /members` returns provider roster
  rows first with role/status/admin/owner/permissions fields and falls back to
  `source=message_heuristic` only when no provider roster exists. The inspector
  members panel now exposes provider sync, roster source labels, role/admin
  badges and realtime `telegram.participant.updated` cache patching. Basic
  groups/private roster sync, admin commands and authoritative participant
  lifecycle reconciliation beyond explicit TDLib roster/service-message
  evidence remain open.
- 2026-06-17: Telegram participant lifecycle commands gained provider-write
  join/leave coverage. `POST /api/v1/telegram/chats/join` and
  `POST /api/v1/telegram/chats/{chat_id}/leave` enqueue durable
  `telegram_provider_write_commands` rows; active TDLib actors dispatch
  `joinChat`/`leaveChat`; the Members inspector exposes capability-gated
  Join/Leave controls through TanStack Query. Commands intentionally remain
  `awaiting_provider` after TDLib ACK until provider-observed roster/chat
  reconciliation confirms the actual membership state.
- 2026-06-17: TDLib member sync now performs provider-observed reconciliation
  for self `join` commands when `getSupergroupMembers` returns the selected
  account's own `user:<telegram_id>` as an active roster member. Matching
  commands move to `completed`, persist `provider_state`, and emit
  `telegram.command.status_changed` plus `telegram.command.reconciled`. Leave
  commands are not completed from absence in the recent roster because the
  current TDLib member query is not an authoritative absence proof.
- 2026-06-17: TDLib history sync now also performs provider-observed
  participant lifecycle reconciliation from explicit service-message evidence.
  When `messageChatDeleteMember` or `messageChatAddMembers` names the selected
  account's own Telegram user id, matching `leave` or `join` commands can move
  to `completed`, persist provider message evidence in `provider_state`, and
  emit command status/reconciled events. This still does not treat roster
  absence as proof and does not cover silent/admin membership state changes.
- 2026-06-17: TDLib unsolicited `updateChatIsMarkedAsUnread` and
  `updateChatNotificationSettings` updates now parse into sanitized runtime
  snapshots, project provider-observed unread-flag and mute state back into
  `telegram_chats.metadata`, emit `telegram.chat.updated` /
  `telegram.chat.muted`, and reconcile matching `mark_read`/`mark_unread` plus
  exact-shape `mute`/`unmute` commands into `completed` only after provider
  observation. Archive/folder parity, true message-level read receipts and
  custom mute-shape reconciliation remain open.
- 2026-06-17: TDLib `updateChatPosition` updates now parse into sanitized
  runtime chat-position snapshots, preserve `tdlib_chat_positions` plus
  derived `is_archived` / `folder_ids` projection metadata, emit
  `telegram.chat.archived`, and reconcile matching `archive`/`unarchive`
  commands into `completed` when TDLib observes archive or main-list presence.
  Provider folder labels/mutation and true message-level read receipts remain
  open.
- 2026-06-17: dialog pin reconciliation now also completes from TDLib
  `updateChatPosition` main/archive list snapshots. Provider-observed
  `is_pinned` is projected into `telegram_chats.metadata`, matching
  `pin`/`unpin` commands reconcile into `completed`, `telegram.chat.pinned`
  is emitted from the runtime bridge, and the chat event payload builders were
  split into a dedicated module so Telegram runtime files stay below the
  700-line architecture limit.
- 2026-06-17: TDLib history sync now also performs provider-observed self
  reaction reconciliation from `interaction_info.reactions`. Chosen emoji
  state completes matching `react` / `unreact` commands with
  `telegram.command.reconciled`, and current-actor reaction rows now deactivate
  when TDLib observes that a previously local emoji is no longer chosen.
- 2026-06-17: Telegram runtime manager methods were refactored to use shared
  operation/start context structs instead of repeated store/config/secret
  argument lists. This keeps runtime modules under the 700-line limit and
  removes the `clippy::too_many_arguments` validation blocker without changing
  provider behavior.
- 2026-06-17: Telegram runtime realtime handling and provider chat-state logic
  were split out of the previously oversized runtime/client files into focused
  modules (`manager/chat_events.rs`, `manager/topic_events.rs`,
  `client/chat_state.rs`), bringing touched production files back under the
  700-line architecture limit without changing scope beyond Telegram.

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

### §4 Capability contract — 82%

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

`POST /api/v1/telegram/runtime/stop` now provides an explicit account-scoped
runtime stop path. It removes the in-memory actor handle, returns the current
runtime status projection and records `telegram.runtime.stop` audit metadata
without changing account lifecycle or deleting local evidence. The Telegram
action rail exposes this as a selected-account stop control backed by the same
typed frontend API and TanStack Query mutation layer as runtime start.
`POST /api/v1/telegram/runtime/restart` now provides the matching controlled
restart path with local audit evidence and frontend action-rail control.

Missing:

- production parity;
- provider command coverage;
- robust live sync lifecycle;
- provider search;
- provider folder state.

### §7 Bot runtime — 10%

Bot account and token references exist. Live Bot API runtime is missing and must
remain blocked until a separate runtime slice exists.

### §8 Dialog/chat list — 67%

Chat projection and chat list endpoint exist. Frontend has virtualized chat list
with filters and metadata badges. Selected chat workbench now exposes local
pin/archive/mute/read toggles through the verified dialog-action routes.
Unread counters are now recomputed from projected received messages and stored
in chat metadata. Read/unread dialog commands now also enqueue provider-write
rows that active TDLib actors execute as manual unread flag toggles. Inspector
rail now reads projected chat detail and provider/fallback member roster rows
for the selected chat. `GET /api/v1/telegram/folders?account_id=`
now exposes projection-backed folder/chat-list filters from
`telegram_chats.metadata.folder_name`, and the Telegram action rail now consumes
that dedicated query instead of deriving folder groups only from the currently
loaded chat list, scoped to the selected account when account context exists.
Unread mention state is now also visible in the primary workbench: chat rows can
render mention badges from projected `mention_count`, and the selected thread
header surfaces unread and mention summary chips from the same projection state.
Individual message bubbles now also render read-only mention chips from
projected per-message `mentions`, `mention_count` and `mentions_detected_by`
metadata, so the aggregate unread mention badge can be traced back to source
message evidence in the thread.
The inspector `Members` tab now also supports local search over the projected
member roster returned by `GET /api/v1/telegram/chats/{chat_id}/members`.
Provider-backed member projection now exists for TDLib supergroups/channels:
`POST /api/v1/telegram/chats/{chat_id}/members/sync` fetches
`getSupergroupMembers`, stores `telegram_chat_participants` rows with
role/status/admin/owner/permissions evidence, emits
`telegram.participant.updated`, and the frontend members panel patches cached
member queries before invalidation. When no provider roster is available, the
members endpoint still exposes message-sender aggregation only as
`source=message_heuristic` fallback.
TDLib Saved Messages can now be identified as a special projected dialog when
the account owner private chat is present in TDLib chat sync; the UI renders a
Saved Messages marker without changing the public chat kind contract.

Missing:

- pinned chats provider parity;
- archived/provider sync parity;
- provider-backed folder sync and folder mutations;
- mute provider sync;
- true message-level read receipts and richer read-position history;
- provider-identity mention targeting and provider mention sync parity;
- provider-observed Saved Messages reconciliation and non-TDLib fallback;
- provider-side search;
- basic-group/private-chat provider roster parity and member pagination beyond
  the first TDLib recent page;
- participant admin command execution;
- provider-observed participant lifecycle reconciliation.

2026-06-17 pass note: per-message mention projection is now visible in the
thread UI. This uses existing Communication projection metadata only; it does
not add provider identity targeting or cross-domain identity/persona matching.

### §9 Private chats — 60%

Private chat kind exists and selected history sync works.
Saved Messages is now detected as a read-only metadata specialization of a
private TDLib self-chat when the account owner id matches the private chat user
id.

Missing:

- full provider parity;
- Saved Messages provider reconciliation outside TDLib owner-id evidence;
- read/unread sync;
- typing indicators;
- reply/reaction/edit/delete lifecycle.

### §10 Groups/supergroups/channels — 45%

`group` and `channel` chat kinds exist. TDLib supergroups remain API-compatible
as `group`/`channel`, but TDLib sync now preserves distinct supergroup metadata
(`tdlib_chat_type`, `tdlib_supergroup_id`, `is_supergroup`,
`is_channel_supergroup`, `is_forum`) and the inspector `About` tab renders that
projection evidence.
TDLib `chat.permissions` boolean fields are also preserved in
`telegram_chats.metadata.tdlib_permissions`, and the inspector renders them as
read-only provider capability evidence for the selected dialog.
TDLib join/leave service messages now project read-only `telegram_join_leave`
structured evidence into message metadata and the thread reference panel can
render those events.
Provider join/leave commands are now queued through the durable outbox and
dispatched by active TDLib actors via `joinChat`/`leaveChat`; projection state
is not mutated by dispatch. TDLib member sync can now mark self `join` commands
completed only after the provider roster returns the account's own active
member id. TDLib history sync can now also reconcile self `join`/`leave` from
explicit provider service-message evidence when `messageChatAddMembers` or
`messageChatDeleteMember` names the selected account. Absence in the current
recent roster page is still not treated as proof, and silent/admin state
changes still require stronger provider observation.

Missing:

- basic-group/private-chat provider roster parity;
- member pagination beyond the first TDLib recent page;
- admin actions;
- authoritative provider-observed lifecycle reconciliation for silent/admin
  membership changes;
- channel post/admin state;
- topic-enabled forum support.

### §11 Topics/forums — 60%

Foundation in place: `telegram_topics` table (migration 0086), `TelegramTopic` model,
`topics.rs` client (upsert/list/get), three API routes
(`GET /chats/{id}/topics`, `GET /topics/{id}`, `GET /topics/{id}/messages`),
frontend types, TanStack Query hooks (`useTelegramTopicsQuery`,
`useTelegramTopicMessagesQuery`), and a Topics tab in `TelegramThreadSideSections`.
The Topics tab now also renders projected topic state and provider identity:
pinned/closed/unread flags, provider topic id and last projected activity are
visible as read-only Communication projection evidence.

Live TDLib sync is now wired: `GetForumTopics` runtime command, `actor_get_forum_topics`
actor handler, `request_actor_get_forum_topics` async wrapper, and `sync_forum_topics`
method on `TelegramRuntimeManager`. The `GET /chats/{id}/topics` API route calls
`sync_forum_topics` before serving the DB projection, so topics are auto-populated for
live TDLib accounts on first request. Falls back silently to DB rows for fixture accounts.
TDLib `updateForumTopicInfo` runtime updates now upsert the existing topic
projection and emit `telegram.topic.updated` with a sanitized projected topic
snapshot for frontend cache patching/replay.

Missing:

- topic create/close/reopen provider-write commands;
- richer topic unread/live state reconciliation beyond `updateForumTopicInfo`
  title/pin/closed projection updates.

2026-06-17 pass note: topic projection UI gained read-only state/provider
labels for existing `TelegramTopic` fields. This does not implement topic write
commands or provider-observed `updateForumTopicInfo` reconciliation.
2026-06-17 pass note: `updateForumTopicInfo` provider-observed topic info
updates now reconcile existing projection fields and realtime caches. This does
not implement topic write commands or provider-side topic creation/closure.

### §12 Message ingestion/projection — 88%

Fixture and TDLib messages become append-only raw records and
`communication_messages`. TDLib media messages may have empty text while
preserving raw payload.
TDLib metadata derivation now lives in `messages/message_metadata.rs`, while
`messages/ingestion.rs` stays focused on orchestration: validate message,
resolve account, upsert chat, write raw source record, project Communication
message and run post-projection refreshes.
`GET /api/v1/telegram/messages/{message_id}/raw` now exposes the underlying raw
provider evidence row for a projected Telegram message with recursive
secret-like key redaction.
For public username-backed chats, ingestion now projects stable provider
permalinks into message metadata as `message_link` / `message_link_kind` without
inventing links for private or non-public chats.
TDLib poll, location, venue and contact-card payloads now also project read-only
structured evidence (`telegram_poll`, `telegram_location`,
`telegram_contact_card`) from raw source payloads.

Missing:

- dedicated reply fields;
- private/group/internal Telegram deep-link projection;
- reaction projection;
- edit version projection;
- delete/tombstone projection;
- forward attribution;
- topic identity.

### §13 Message lifecycle commands — 58%

Manual text send exists. Edit/delete/restore lifecycle endpoints now persist:

- append-only message versions;
- tombstones;
- provider-write command rows;
- redacted audit for send/edit/delete.

Frontend now also reads lifecycle evidence in the per-message reference panel,
which consumes projected versions, tombstones and recent account command rows
for the current message/chat context. Inspector context now also exposes a
first-class recent-command audit panel for the selected account with local
current-chat filtering, search over durable command rows, retry-budget summaries
and read-only dead-letter surfacing for exhausted failed commands. The reference
panel itself now also supports local filtering across reply/forward/lifecycle
evidence rows, plus diff-style edit history summaries, tombstone visibility
state summaries and command capability/action metadata for the current message.
Active TDLib actors execute queued edit/delete commands through the command
executor; without QR runtime the same operations remain degraded local evidence
writes.

Remaining gaps:

- provider-observed edit/delete reconciliation and retry controls;
- provider-side restore parity, where applicable;
- provider-observed read/unread reconciliation beyond manual unread toggles;
- richer backoff/dead-letter controls beyond the current read-only retry/fail/dead-letter command rows;
- per-target result rows beyond the current command record.

### §14 Replies/forwards/pins — 58%

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
state with durable command rows, redacted audit, thread-level pin/unpin
controls and TDLib provider execution through active runtime actors.

Reply command flow is now complete: `tdlib_send_reply_request` builder,
`ReplyMessage` runtime command variant, actor handler, async request function,
`POST /api/v1/telegram/messages/{message_id}/reply` API route, and full
frontend wiring — composer reply-preview banner, Reply action in message list,
and `useTelegramSendActions` composable routing send vs reply in TelegramPage.

Forward command flow now has the same live TDLib path for direct user actions:
`tdlib_send_forward_request` builds `forwardMessages`, `ForwardMessage` runtime
commands dispatch through the actor, `send_forward_message` ingests the returned
TDLib snapshot into the Communication projection, `POST
/api/v1/telegram/messages/{message_id}/forward` publishes message/command
events, and the Vue thread action calls the TanStack Query mutation without
component-local fetches.

Missing:

- deeper reply graph/thread traversal beyond single-hop projected reopen;
- provider-grade forward attribution beyond current origin metadata;
- queued outbox forward result projection after executor dispatch;
- provider-observed pin sync/reconciliation.

### §15 Reactions — 62%

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
Active TDLib actors execute queued provider reaction add/remove commands through
the command executor; without QR runtime the capability remains degraded local
projection.
TDLib provider-origin aggregate emoji reactions are now projected from
`interaction_info.reactions` into `message.metadata.reaction_summary`, so the
existing thread summary chips can render source-backed provider counts even when
sender-level reaction rows have not been reconciled.
When TDLib includes `recent_reactions`, ingestion now also upserts sender-level
emoji rows into `telegram_message_reactions` with provider provenance, allowing
the existing reactions endpoint/reference panel to expose provider-observed
reaction actors for recent reactions. TDLib aggregate `is_chosen` state now
also reconciles the current actor's own emoji rows during message ingest,
deactivating absent self reactions and keeping chosen self reactions active
even when the provider snapshot does not list them in `recent_reactions`.
Custom TDLib reaction aggregate counts are preserved under
`reaction_summary.custom_reactions` as read-only source evidence, and the
message reference panel renders those custom reaction counts as provider
evidence without inserting them into `telegram_message_reactions` or exposing
fake custom reaction write controls.
The thread reaction chips and capability-gated emoji picker are now isolated in
`TelegramMessageReactions.vue`; this is a frontend SRP refactor only and does
not change provider-origin realtime coverage.

Remaining gaps:

- provider removal/absence reconciliation for non-self reaction actors not present in current TDLib evidence;
- custom reaction row mapping beyond aggregate metadata;
- provider-origin `telegram.reaction.changed` realtime updates now flow from
  TDLib `updateMessageInteractionInfo`, but non-self sender-row removal parity
  and richer cache patching still require additional work.

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
Provider-side media upload now has a backend command path: generic local
attachment import writes shared blob/import metadata, Telegram upload accepts
only `attachment_id` or `blob_id`, records redacted audit, queues `send_media`,
and the outbox executor resolves local blobs into TDLib `sendMessage` media
requests for photo, video, document, audio, voice note, sticker and
animation/GIF. Command completion is still reserved for provider-observed TDLib
message snapshots.

Missing:

- Bot API media send;
- richer upload progress/cache patch UX;
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
The Telegram workbench now renders that shared transport state in its status
area, including degraded fallback/offline labels from `useRealtimeStatusStore`.
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
Account-scoped `telegram.sync.*` and `telegram.command.status_changed` events
now also patch cached runtime status queries, so selected-chat inspector/runtime
surfaces can show the latest sync scope/status/count and the last observed
Telegram command outcome without a full reload. Current local dialog command
events now also carry a sanitized projected `chat` snapshot plus `action`,
which lets the frontend patch account-scoped chat lists and active chat-detail
caches immediately after pin/archive/mute/read/unread actions.
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
- `telegram.command.status_changed`: status field update on the matching row in the commands list cache;
- `telegram.reaction.changed`: aggregate summary count patch on the reaction detail cache.

Remaining gaps:

- provider-observed `telegram.chat.updated` events are not yet emitted by backend runtime reconciliation;
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
TDLib actors execute via `toggleChatIsMarkedAsUnread`; archive/unarchive and
mute/unmute commands now dispatch through `addChatToList` and
`setChatNotificationSettings`. Realtime command-status payloads for those
dialog actions now include projected chat snapshots, so chat list/detail state
can update across active clients without waiting for a manual reload.

Missing:

- provider-observed archive/folder and mute/notification reconciliation;
- provider-observed read/unread reconciliation and message-level read receipts;
- richer typing clear/expiry semantics beyond the current TDLib typing
  projection;
- provider-backed folder sync and folder mutations;
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

## Recommended next implementation order

1. Detailed capability matrix.
2. Telegram realtime event contracts.
3. Message lifecycle schema: versions/tombstones/replies/forwards/reactions.
4. Provider command/outbox model beyond send.
5. Dialog management: unread, pinned, archive, mute, folders.
6. Media gallery/search/preview.
7. Telegram-specific AI surfaces.
8. Voice/calls with explicit native permission boundary.
