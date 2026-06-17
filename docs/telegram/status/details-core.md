# Telegram Implementation Status — Details Core (§2-§16)

См. также: [Status Index](../status.md) и [Details Extended](details-extended.md).

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
`telegram_chats.metadata.folder_name` / `folder_labels`, including optional
`provider_folder_id` when TDLib folder metadata is known, and the Telegram
action rail now consumes
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
Provider-backed member projection now exists for TDLib supergroups/channels and
private/Saved Messages chats: `POST /api/v1/telegram/chats/{chat_id}/members/sync`
fetches `getSupergroupMembers` for supergroups/channels and can hydrate
private-dialog roster rows from TDLib chat metadata (`tdlib_private_user_id`),
stores `telegram_chat_participants` rows with role/status/admin/owner/
permissions evidence, marks previously projected TDLib members as
`absent_exhaustive` when an exhaustive provider roster snapshot no longer
contains them, emits `telegram.participant.updated`, and the frontend
members panel patches cached member queries before invalidation; active
member views now exclude inactive provider rows marked `left`, `banned`,
or `absent_exhaustive`. When no
provider roster is available, the members endpoint still exposes
message-sender aggregation only as
`source=message_heuristic` fallback.
TDLib Saved Messages can now be identified as a special projected dialog when
the account owner private chat is present in TDLib chat sync; the UI renders a
Saved Messages marker without changing the public chat kind contract.

Missing:

- pinned chats provider parity;
- archived/provider sync parity;
- provider-backed folder mutations;
- mute provider sync;
- true message-level read receipts and richer read-position history;
- provider-identity mention targeting and provider mention sync parity;
- provider-observed Saved Messages reconciliation and non-TDLib fallback;
- provider-side search;
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
`messageChatDeleteMember` names the selected account. Self `leave` can also
reconcile when an exhaustive provider roster snapshot no longer contains the
selected account. Absence in a non-exhaustive recent roster page is still not
treated as proof. TDLib supergroup member sync now also merges
`supergroupMembersFilterAdministrators`, so provider roster updates can observe
admin role/permission changes even when those admins are not present in recent
member pages, but silent state
changes still require stronger provider observation.

Missing:

- admin actions;
- authoritative provider-observed lifecycle reconciliation for silent
  membership changes without explicit inactive roster evidence;
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

- richer edit/delete provider diff/history UX and broader retry/dead-letter controls;
- provider-side restore parity, where applicable;
- richer message-level read/unread receipt history beyond current dialog unread reconciliation;
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
- richer message-level pin ordering/state parity beyond current provider-observed reconciliation and mismatch evidence.

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
