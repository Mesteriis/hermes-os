# Telegram Implementation Status — Pass Log

См. также: [Status Index](../status.md).

## Pass Log

- 2026-06-17: restored topic provider-write command kinds in the migration
  chain. Migration `0090_restore_topic_telegram_command_kinds.sql` restores
  `topic_create`, `topic_close` and `topic_reopen` in the
  `telegram_provider_write_commands_command_kind` constraint after the
  narrower `0085` allowlist regressed topic command inserts. Runtime topic
  reconciliation now has a direct test that seeds a queued `topic_close`
  command, observes TDLib `updateForumTopicInfo`, and verifies
  `telegram.command.status_changed`, `telegram.command.reconciled` and
  `telegram.topic.updated` append to `event_log` in order.
- 2026-06-17: realtime command reconciliation now has direct backend coverage
  in `runtime/manager/realtime_events.rs`: a targeted async test verifies that
  provider-observed reconciliation appends both
  `telegram.command.status_changed` and `telegram.command.reconciled` to
  `event_log`, matching the existing frontend command-cache patch contract.
- 2026-06-16: capability contract aligned with the existing TDLib command
  executor for edit/delete/pin/reaction provider writes; Telegram and WhatsApp
  capability support were split out of the former messaging integration helper
  so touched production modules remain below the 700-line architecture limit.
- 2026-06-17: capability contract now reports forum-topic read support
  truthfully. `topics.list` is no longer marked unsupported when projection/API
  routes and realtime cache patching already exist; account-scoped responses
  degrade topic reads when only local projection is available, mark bot account
  topic operations unsupported, and keep `topics.create` / `topics.close`
  blocked until provider-write runtime commands and reconciliation exist.
- 2026-06-17: forum-topic provider writes are now wired through the durable
  Telegram outbox. `POST /api/v1/telegram/chats/{telegram_chat_id}/topics`
  queues `topic_create`, `POST /api/v1/telegram/topics/{topic_id}/close`
  queues `topic_close` / `topic_reopen`, the command executor dispatches TDLib
  `createForumTopic` / `toggleForumTopicIsClosed`, immediate create responses
  reconcile into the topic projection, and `updateForumTopicInfo` now
  reconciles provider-observed close/reopen state into `telegram.command.reconciled`.
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
  for TDLib supergroups/channels and metadata-backed parity for private/Saved
  Messages chats. Migration 0089 adds
  `telegram_chat_participants`; `POST /api/v1/telegram/chats/{chat_id}/members/sync`
  fetches TDLib `getSupergroupMembers`, while private/saved-message dialogs can
  hydrate roster rows from TDLib chat metadata (`tdlib_private_user_id`);
  `GET /members` returns provider roster rows first with
  role/status/admin/owner/permissions fields and falls back to
  `source=message_heuristic` only when no provider roster exists. The inspector
  members panel now exposes provider sync, roster source labels, role/admin
  badges, query-backed search, exact role filtering, cursor-based `Load more`
  pagination, and realtime `telegram.participant.updated` cache patching. Admin
  commands and authoritative participant lifecycle reconciliation
  beyond explicit TDLib roster/service-message
  evidence remain open.
- 2026-06-17: Telegram participant lifecycle commands gained provider-write
  join/leave coverage. `POST /api/v1/telegram/chats/join` and
  `POST /api/v1/telegram/chats/{chat_id}/leave` enqueue durable
  `telegram_provider_write_commands` rows; active TDLib actors dispatch
  `joinChat`/`leaveChat`; the Members inspector exposes capability-gated
  Join/Leave controls through TanStack Query. Commands intentionally remain
  `awaiting_provider` after TDLib ACK until provider-observed roster/chat
  reconciliation confirms the actual membership state.
- 2026-06-17: Telegram files and voice tabs now surface a derived current-chat
  media download queue from projected attachment readiness. Realtime
  `telegram.media.download.started/progress/failed/downloaded` events patch
  message/media-search attachment state in place, and the queue exposes
  in-flight/failure state plus retry actions without polling or component-level
  fetches.
- 2026-06-17: TDLib member sync now performs provider-observed reconciliation
  for self `join` commands when `getSupergroupMembers` returns the selected
  account's own `user:<telegram_id>` as an active roster member. Matching
  commands move to `completed`, persist `provider_state`, and emit
  `telegram.command.status_changed` plus `telegram.command.reconciled`. Leave
  commands are now also completed when the same roster explicitly returns the
  selected account in inactive `left` / `banned` state. Absence in the recent
  roster is still not treated as proof because the current TDLib member query
  is not an authoritative absence proof.
- 2026-06-17: TDLib history sync now also performs provider-observed
  participant lifecycle reconciliation from explicit service-message evidence.
  When `messageChatDeleteMember` or `messageChatAddMembers` names the selected
  account's own Telegram user id, matching `leave` or `join` commands can move
  to `completed`, persist provider message evidence in `provider_state`, and
  emit command status/reconciled events. This still does not treat roster
  absence as proof and does not cover silent/admin membership state changes
  without explicit inactive roster or service-message evidence.
- 2026-06-17: TDLib unsolicited `updateChatIsMarkedAsUnread` and
  `updateChatNotificationSettings` updates now parse into sanitized runtime
  snapshots, project provider-observed unread-flag and mute state back into
  `telegram_chats.metadata`, emit `telegram.chat.updated` /
  `telegram.chat.muted`, and reconcile matching `mark_read`/`mark_unread` plus
  exact-shape `mute`/`unmute` commands into `completed` only after provider
  observation. Archive/folder parity, true message-level read receipts and
  custom mute-shape reconciliation remain open.
- 2026-06-17: selected thread header now surfaces projected
  `last_read_inbox_provider_message_id` as provider-observed read-progress
  evidence, and realtime cache patch tests now cover that metadata flowing
  through `telegram.chat.updated` into both chat list and chat detail caches.
- 2026-06-17: thread timeline now derives message-level read progress from the
  same projected `last_read_inbox_provider_message_id`. A shared frontend
  helper parses TDLib-style provider message ids, computes the last visible
  read boundary for the selected chat, and `TelegramMessageList.vue` renders a
  `Read through here` divider without adding any component-level fetch or
  polling path.
- 2026-06-17: dialog mark-read requests can now carry a targeted
  `last_read_inbox_provider_message_id` from the visible thread window. The
  durable `mark_read` command stores that provider message id, active TDLib
  actors dispatch `viewMessages(force_read=true)` instead of only toggling the
  unread flag, and `updateChatReadInbox` can now reconcile queued targeted
  read commands into `telegram.command.reconciled`.
- 2026-06-17: `POST /api/v1/telegram/messages/{message_id}/mark-read` now
  resolves the projected chat/message target, records a provider-write
  `mark_read` outbox row plus API audit evidence, updates the selected chat's
  local read-progress metadata, and lets the thread expose a dedicated
  message-level `Mark read` action on the latest visible incoming unread
  message while provider reconciliation still comes from `updateChatReadInbox`.
- 2026-06-17: dialog lifecycle routes (`pin/unpin/archive/unarchive/mute/
  unmute/read/unread/join/leave`) now also write explicit `api_audit_log`
  records via `NewApiAuditRecord::telegram_chat_action`, keeping chat-level
  provider writes aligned with the existing message/topic/media/runtime audit
  boundary.
- 2026-06-17: Telegram command audit UI now formats targeted `mark_read`
  commands as readable progress (`Read through <provider_message_id>`) and
  prefers provider-observed `last_read_inbox_message_id` when reconciliation
  has completed, instead of showing raw provider ids without context.
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
- 2026-06-17: runtime member-roster sync now has direct `event_log` coverage
  for self join/leave reconciliation ordering. Dedicated runtime tests verify
  that roster-based self presence and exhaustive self absence first append
  `telegram.participant.updated`, then `telegram.command.status_changed`, and
  finally `telegram.command.reconciled`, while the matching `join` / `leave`
  command row ends in `completed` + `observed`.
- 2026-06-17: Telegram capability catalog now reflects the current dialog
  provider-write contract more accurately. `dialogs.pin` is classified as
  `provider_write`, and QR-ready dialog `pin/archive/mute` plus projected
  unread counters now report `available` in the catalog because the durable
  outbox plus provider-observed TDLib reconciliation path is already present.
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
