# Telegram API Reference

Статус: verified route audit + целевой API scope на 2026-06-17.

Все текущие маршруты защищены локальным API guard из ADR-0056, если явно не
указано иначе. Browser WebSocket clients передают local secret через
`hermes_secret`, потому что native WebSocket requests не могут выставить
`X-Hermes-Secret`.

## Base

```text
/api/v1/telegram
```

## Capability Contract

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/capabilities` | Detailed per-operation capability matrix with `operation`, `category`, `status`, `action_class`, `reason`, `confirmation_required`, `closure_gate` |
| GET | `/api/v1/telegram/accounts/{account_id}/capabilities` | Account-scoped capability matrix with the same operation contract plus selected-account scope metadata and runtime/provider overrides |

### Целевые capability states

```text
available
degraded
blocked
unsupported
```

### Целевые operation groups

- account lifecycle;
- runtime;
- sync;
- read/search;
- send;
- edit;
- delete;
- react;
- pin;
- media download/upload;
- export;
- session/proxy;
- calls/recording;
- admin actions.

Account-scoped responses now also include:

```text
account_scope.account_id
account_scope.provider_kind
account_scope.runtime_kind
account_scope.lifecycle_state
```

Current account-aware overrides cover:

- bot accounts not using TDLib QR operations;
- user accounts not using Bot API runtime operations;
- `logged_out` / `removed` lifecycle blocking selected runtime/sync/write actions.

Telegram inspector `About` tab now also surfaces the account-scoped capability
route as a read-only matrix for the selected account, exposing operation status,
action class, reason, confirmation requirement and closure-gate metadata
alongside the existing capability-gated controls.

## Accounts

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/accounts/fixture` | Создать fixture `telegram_user` или `telegram_bot` account metadata |
| GET | `/api/v1/telegram/accounts?include_removed=` | Список Telegram provider accounts |
| POST | `/api/v1/telegram/accounts` | Создать live/live-blocked/QR-authorized Telegram account metadata и secret bindings |
| DELETE | `/api/v1/telegram/accounts/{account_id}` | Mark account `removed`, stop runtime actor, preserve local evidence |
| POST | `/api/v1/telegram/accounts/{account_id}/logout` | Mark account `logged_out`, stop runtime actor |

Account config хранит non-secret metadata. Credential payloads resolved через
host-vault/secret references. Telegram workbench `About` inspector tab now
contains a local account manager UI for `setup`, `logout` and `remove`, and the
header `Add Account` action opens that account-management surface.

### Дополнительно реализовано

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/accounts/{account_id}/capabilities` | Account-scoped detailed capability matrix |

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/accounts/{account_id}/export-session` | Sanitized session bundle export без secrets unless explicitly encrypted |
| POST | `/api/v1/telegram/accounts/import-session` | Import encrypted session bundle |
| GET/PUT | `/api/v1/telegram/accounts/{account_id}/proxy` | Proxy / MTProxy / SOCKS5 profile binding |

## Runtime

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/runtime/status?account_id=` | Account-scoped runtime status, runtime kind, TDLib readiness, split Telegram app-credential flags, TDLib probe error and derived runtime blockers |
| POST | `/api/v1/telegram/runtime/start` | Start fixture или TDLib QR-authorized runtime actor |
| POST | `/api/v1/telegram/runtime/stop` | Stop the account-scoped runtime actor idempotently and return the current runtime status with local audit evidence |
| POST | `/api/v1/telegram/runtime/restart` | Stop then start the account-scoped runtime actor and return the current runtime status with local audit evidence |

Runtime kinds observed:

```text
fixture
tdlib_qr_authorized
live_blocked
```

Current runtime status payload now also exposes:

```text
tdjson_path
tdjson_probe_error
telegram_api_id_configured
telegram_api_hash_configured
runtime_blockers[]
```

No runtime restart route is currently missing. Remaining runtime gaps are native
dependency remediation, Bot API runtime and portable session/proxy controls.

## QR Login

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/login/qr/start` | Start TDLib QR login setup |
| GET | `/api/v1/telegram/login/qr/{setup_id}` | Poll QR login status |
| DELETE | `/api/v1/telegram/login/qr/{setup_id}` | Cancel pending QR login session |
| POST | `/api/v1/telegram/login/qr/{setup_id}/password` | Submit 2-step verification password |

QR statuses:

```text
waiting_qr_scan
waiting_password
ready
expired
failed
runtime_unavailable
```

Telegram account management UI now exposes a QR login panel inside the
inspector `About` tab. It can call `start`, `status`, `password` and `cancel`
routes, render the current QR/2FA state, and apply suggested account metadata
back into the local account setup form before the user saves the QR-authorized
account record.

## Chats / Dialogs

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/chats?account_id=&limit=` | Projected Telegram chats ordered by last activity |
| POST | `/api/v1/telegram/sync/chats` | Sync account chats through fixture or TDLib runtime |

`telegram_chats.chat_kind` currently supports:

```text
private
group
channel
bot
```

TDLib `chatTypeSupergroup` remains API-compatible as `group`, unless it is a
channel, but selected chat/list payloads now preserve provider-specific
supergroup metadata when TDLib supplied it:

```text
metadata.tdlib_chat_type = chatTypeSupergroup
metadata.tdlib_supergroup_id
metadata.is_supergroup
metadata.is_channel_supergroup
metadata.is_forum
```

TDLib owner private chats are also preserved as Saved Messages projection
evidence when `chatTypePrivate.user_id` matches the selected account external
Telegram user id:

```text
metadata.is_saved_messages = true
metadata.saved_messages_source = tdlib_private_self_chat
metadata.tdlib_private_user_id
```

This is read-only dialog metadata. It does not implement provider reconciliation,
session export or Saved Messages-specific write behavior.

When TDLib supplies `chat.permissions`, selected chat/list payloads also
preserve boolean provider permission evidence under:

```text
metadata.tdlib_permissions
```

These fields are read-only projection metadata. They do not implement member
role lifecycle, admin actions or permission mutation commands. Join/leave uses
durable provider-write commands and does not mutate projection state until
provider-observed reconciliation.

### Текущие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/chats/{chat_id}` | Projection-backed selected-chat detail |
| GET | `/api/v1/telegram/chats/{chat_id}/members?query=&role=&limit=&cursor=` | Provider roster rows when synced; labeled message-sender fallback otherwise |
| POST | `/api/v1/telegram/chats/{chat_id}/members/sync` | TDLib-backed provider roster sync for supergroups/channels |
| POST | `/api/v1/telegram/chats/join` | Queue provider chat join through durable outbox (`command_kind=join`) |
| POST | `/api/v1/telegram/chats/{chat_id}/leave` | Queue provider chat leave through durable outbox (`command_kind=leave`) |
| GET | `/api/v1/telegram/chats/{chat_id}/pinned-messages?limit=` | Projection-backed pinned-message list for the selected chat |
| GET | `/api/v1/telegram/chats/search?q=&account_id=` | Projection-backed dialog search |
| GET | `/api/v1/telegram/folders?account_id=` | Projection-backed Telegram folder/chat-list filters derived from `telegram_chats.metadata.folder_name` |
| POST | `/api/v1/telegram/chats/{chat_id}/pin` | Provider/local pin command |
| POST | `/api/v1/telegram/chats/{chat_id}/archive` | Provider/local archive command |
| POST | `/api/v1/telegram/chats/{chat_id}/mute` | Mute/unmute command |
| POST | `/api/v1/telegram/chats/{chat_id}/read` | Projected mark-read command plus queued TDLib manual-unread clear |
| POST | `/api/v1/telegram/chats/{chat_id}/unread` | Projected mark-unread command plus queued TDLib manual-unread set |

Current implementation of pin/archive/mute/read/unread updates projected local
chat metadata and records durable command rows with realtime command-status
events. Read/unread commands are eligible for active TDLib execution through
`toggleChatIsMarkedAsUnread`; archive/unarchive uses `addChatToList`, and
mute/unmute uses `setChatNotificationSettings`. TDLib unsolicited
`updateChatIsMarkedAsUnread`, `updateChatNotificationSettings` and
`updateChatPosition` updates now project provider-observed unread-flag, mute
state, archive state and dialog pin state back into `telegram_chats.metadata`,
emit `telegram.chat.updated` / `telegram.chat.muted` / `telegram.chat.archived`
/ `telegram.chat.pinned`, and reconcile matching `mark_read` / `mark_unread`,
`pin` / `unpin`, exact-shape `mute` / `unmute`, and `archive` / `unarchive`
commands into `completed` with `telegram.command.reconciled`.
Provider-observed reconciliation remains missing for folder labels/mutations,
true message-level read receipt state and custom mute shapes outside the
current exact TDLib request contract. Folder filters
currently surface only projected/local folder names already persisted into chat
metadata; there is no provider folder sync route in the current backend.
Member sync currently uses TDLib `getSupergroupMembers` for chats with
`tdlib_supergroup_id`, stores provider roster rows in
`telegram_chat_participants`, emits `telegram.participant.updated`, and leaves
basic-group/private roster parity and admin commands open. Join/leave commands
are queued with command status events and active TDLib actors dispatch
`joinChat`/`leaveChat`; dispatch ACK is not completion, so these commands
remain awaiting provider observation until a later roster/chat reconciliation
pass confirms the membership state. Self `join` commands can now complete when
TDLib member sync observes the selected account's own active
`user:<telegram_id>` roster row and emits `telegram.command.reconciled`. TDLib
history sync can now also reconcile self `join`/`leave` when explicit
`messageChatAddMembers` or `messageChatDeleteMember` service messages name the
selected account. `leave` is still not inferred from absence in the current
recent roster page, and silent/admin membership state changes still need
stronger provider evidence. The Telegram workbench action rail now consumes
this dedicated folders route instead of recomputing folder groups only from
the currently loaded chat list, and the Members inspector exposes
capability-gated Join/Leave controls.

## Topics / Forums

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/chats/{telegram_chat_id}/topics?limit=` | Fetch forum topics (DB-backed), with optional live TDLib sync refresh |
| GET | `/api/v1/telegram/topics/{topic_id}` | Read topic details |
| GET | `/api/v1/telegram/topics/{topic_id}/messages` | Read topic-scoped timeline by `forum_topic_id` metadata |
| GET | `/api/v1/telegram/topics/search?q=&telegram_chat_id=&limit=` | Search forum topics within a chat by title |

The frontend Topics tab consumes these routes through TanStack Query and now
renders projected `is_pinned`, `is_closed`, `unread_count`, `provider_topic_id`
and `last_message_at` fields as read-only topic state/provider labels. This is
projection evidence only; it does not create or mutate provider topics.
TDLib `updateForumTopicInfo` runtime updates now refresh the same
`telegram_topics` projection fields for title, icon, pinned and closed state,
then emit `telegram.topic.updated` with a sanitized projected topic snapshot.
Frontend realtime patching updates cached topic list/search queries and replay
invalidates topic caches; no topic write route is introduced by this slice.

### Текущие lifecycle/history маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/messages/{message_id}/edit` | Record append-only local edit version and command metadata |
| POST | `/api/v1/telegram/messages/{message_id}/delete` | Record tombstone evidence and command metadata |
| POST | `/api/v1/telegram/messages/{message_id}/restore-visibility` | Record local visibility restore event, command metadata and redacted audit |
| POST | `/api/v1/telegram/messages/{message_id}/pin` | Record local pin/unpin projection state, command metadata and redacted audit; active TDLib actors execute queued provider pin/unpin |
| POST | `/api/v1/telegram/messages/{message_id}/reply` | Send TDLib reply message through the active QR-authorized runtime |
| POST | `/api/v1/telegram/messages/{message_id}/forward` | Forward a source Telegram message through the active QR-authorized runtime |
| GET | `/api/v1/telegram/messages/{message_id}/versions` | List observed/local message versions |
| GET | `/api/v1/telegram/messages/{message_id}/tombstones` | List message tombstones |
| GET | `/api/v1/telegram/messages/{message_id}/raw` | Sanitized raw provider evidence record for the projected Telegram message |
| GET | `/api/v1/telegram/messages/{message_id}/reply-chain` | Read current reply-chain projection |
| GET | `/api/v1/telegram/messages/{message_id}/forward-chain` | Read current forward-chain projection |
| GET | `/api/v1/telegram/commands?account_id=&limit=` | List durable Telegram command rows; inspector context now surfaces them as a local audit panel with current-chat filter and text search |
| POST | `/api/v1/telegram/commands/{command_id}/retry` | Manually requeue an eligible failed/dead-letter/retrying provider command row through the durable outbox runtime |

These endpoints persist lifecycle evidence and command metadata. Provider-side
edit/delete/reaction/pin/forward command dispatch is handled by the TDLib
command executor for active runtime actors, while direct reply and direct
forward user actions use TDLib runtime send paths that immediately project the
returned message snapshot.
Provider command rows now carry retry/dead-letter, due timestamp, execution
lock and provider-observed reconciliation fields. Dispatch success alone is
not completion: commands complete only after provider-observed state is
persisted; ACK-only writes remain `executing/awaiting_provider` until a later
reconciliation pass observes the provider state. Self `join` is currently
reconciled by TDLib member sync when the active account appears in the provider
roster, and self `join`/`leave` can also reconcile from explicit TDLib
participant service-message evidence during history sync. Silent/admin
membership state changes still remain awaiting stronger provider evidence.
Frontend now uses the reply/forward read endpoints for a read-only per-message
reference panel inside the Telegram thread, and the payload now includes local
message summaries for reply targets/replies plus source summaries for forward rows.
That same panel now also consumes `GET /versions`, `GET /tombstones`,
`GET /reactions` and `GET /commands?account_id=` as read-only lifecycle evidence.
Message pin/unpin now follows the same local-write contract and writes both
`pinned` and `is_pinned` metadata keys for projection/UI compatibility.
The raw evidence route returns the append-only `communication_raw_records` row
for a Telegram-projected message and recursively redacts secret-like keys while
preserving source payload fields such as TDLib metadata. The frontend thread
reference panel consumes this route through `useTelegramRawMessageEvidenceQuery`
and renders sanitized payload/provenance in a dedicated source-evidence section;
components do not call `fetch` directly.
Projected message metadata may also include `message_link` and
`message_link_kind=public_t_me` for messages in public username-backed Telegram
chats. The reference panel renders that permalink as read-only provider
evidence. Hermes does not invent private/group deep links when provider evidence
does not contain a public username.
Projected message metadata may also include read-only structured evidence from
TDLib raw payloads:

- `telegram_poll` for `messagePoll`;
- `telegram_location` for `messageLocation` and `messageVenue`;
- `telegram_contact_card` for `messageContact`.
- `telegram_join_leave` for TDLib join/leave service messages.
- `reaction_summary` aggregate emoji counts from TDLib
  `interaction_info.reactions`.

These fields are Communication projection evidence only. They do not create
Contact/Persona lifecycle records or any cross-domain extraction workflow.

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/topics/{topic_id}/create` | Topic write command (create forum topic) |
| POST | `/api/v1/telegram/topics/{topic_id}/close` | Topic write command (close/reopen forum topic) |
| POST | `/api/v1/telegram/messages/{message_id}/mark-read` | Provider mark read |

## Messages

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/telegram/messages?account_id=&provider_chat_id=&limit=` | Recent projected Telegram messages |
| POST | `/api/v1/telegram/messages` | Ingest fixture Telegram message |
| POST | `/api/v1/telegram/sync/history` | Sync selected chat history |
| POST | `/api/v1/telegram/messages/send` | Manual text send through fixture or TDLib QR-authorized runtime |

`POST /api/v1/telegram/sync/history` supports:

```text
latest
older
full
```

`older` requires `from_message_id`.

Projected Telegram message payload metadata can include mention evidence:

```text
metadata.mention_count
metadata.mentions
metadata.mentions_detected_by
```

The thread UI renders this metadata as read-only mention chips and chat
metadata uses it to derive aggregate unread mention counts. These fields are
source-evidence projection only; they do not perform provider identity targeting
or Persona/identity resolution.

## Reactions

### Текущие маршруты

| Method | Path | Назначение |
|---|---|---|
| POST | `/api/v1/telegram/messages/{message_id}/reactions` | Add reaction |
| DELETE | `/api/v1/telegram/messages/{message_id}/reactions` | Remove reaction |
| GET | `/api/v1/telegram/messages/{message_id}/reactions` | Current reaction projection |

Current implementation persists local reaction projection rows, durable command
metadata and redacted audit records for add/remove, exposes degraded local-write
capability states, and emits both `telegram.reaction.changed` and
`telegram.command.status_changed`; active TDLib actors execute queued
provider reaction commands, while provider-origin reaction reconciliation is
partial. TDLib message ingestion now projects aggregate emoji reaction counts
from `interaction_info.reactions` into message metadata for existing thread
summary UI and upserts sender-level emoji rows from
`interaction_info.reactions.recent_reactions` when TDLib supplies concrete
`messageSender*` evidence. Custom reaction counts are preserved separately in
`reaction_summary.custom_reactions` without faking `reaction_emoji` rows, and
the message reference panel renders those custom aggregate counts as read-only
provider evidence. TDLib history sync now also reconciles matching self
`react` / `unreact` commands from provider-observed chosen emoji state in
`interaction_info.reactions`, and current-actor emoji rows can deactivate when
TDLib observes that a previously local reaction is no longer chosen. Non-self
actor removal/absence reconciliation and custom reaction row mapping are still
missing. Runtime-started TDLib actors now also forward unsolicited
`updateMessageInteractionInfo` through the actor event bridge, refresh the
projected reaction summary plus sender rows and emit provider-origin
`telegram.reaction.changed`.

## Media

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/media/download` | Download TDLib file and persist completed local blob + Communication attachment row |
| POST | `/api/v1/telegram/media/upload` | Queue provider-side media send from a local `attachment_id` or `blob_id` through the durable Telegram command outbox |
| GET | `/api/v1/telegram/search/media?account_id=&provider_chat_id=&kind=&limit=` | Projected Telegram media gallery/filter with attachment download metadata for files-tab parity |

Fixture runtime intentionally fails closed for media downloads.
Media download requests emit `telegram.media.download.started` before runtime
dispatch, `telegram.media.download.failed` when the runtime rejects or fails the
request, and `telegram.media.download.progress` when TDLib returns a
non-completed file snapshot. Completed TDLib downloads also update projected
Telegram message attachment metadata (`download_state`, `local_path`,
`attachment_id`, `tdlib_file_id`) and emit `telegram.media.downloaded` with
sanitized projected `message` and `chat` snapshots for realtime cache patching.
Message ingestion now also derives remote attachment metadata for TDLib
`messageSticker`, `messageAnimation` and `messageVideoNote` payloads. Those rows
reuse the existing projected `attachments` metadata contract and can be listed
by the files tab, downloaded through the existing media route, and previewed
after local download.

Media upload uses the provider command model:

```text
POST /api/v1/communications/attachments/import
  -> communication_mail_blobs + communication_attachment_imports
POST /api/v1/telegram/media/upload
  -> telegram_provider_write_commands(command_kind=send_media)
  -> TDLib sendMessage media builder
```

`/api/v1/telegram/media/upload` does not accept raw file bytes and does not call
TDLib directly from UI. It accepts a local `attachment_id` or `blob_id`, rejects
malicious imported attachments, records provider-write audit metadata and emits
`telegram.media.upload.started` plus command status events. Provider completion
is only recorded after the outbox executor receives a provider message snapshot.

Supported backend media send types:

```text
photo
video
document
audio
voice
sticker
animation / gif
```

The workspace search UI consumes the same media search response, includes media
hits in the result count and projects local-preview/download readiness from
`local_path`, `tdlib_file_id`, `provider_attachment_id` and `download_state`.
When a downloaded Telegram attachment has a projected Communication
`attachment_id`, the Telegram media viewer can call the shared
`GET /api/v1/communications/attachments/{attachment_id}/preview` route for safe
text/image previews. That shared route enforces local blob, scan-status and
size-limit checks; Hermes still does not expose a Telegram-specific preview
bytes route.
TDLib messages with `media_album_id` now also project album grouping metadata
into the canonical message payload:

```text
metadata.media_album_id
metadata.media_album_key = provider_chat_id + media_album_id
```

The Telegram Files tab can group already-loaded selected-chat messages by this
key. Provider-wide album sync and album-aware media upload/send are still
missing.

Media download required fields:

```text
account_id
provider_chat_id
provider_message_id
tdlib_file_id
```

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/media` | Media gallery/search |
| GET | `/api/v1/telegram/media/{media_id}/preview` | Telegram workbench preview |
| GET | `/api/v1/telegram/media/search?q=&type=` | Provider/local media search |
| POST | `/api/v1/telegram/voice/send` | Voice message send |
| POST | `/api/v1/telegram/video-note/send` | Video note send |

## Attachments

Telegram currently reuses shared Communication attachment APIs after media download.
The Telegram Files tab now exposes the shared
`/api/v1/communications/attachments/search` index through a Telegram-scoped
panel limited by the selected chat account id, so downloaded Telegram
attachments can be found by filename/content type/scan status without creating
a Telegram-specific attachment store.

Target: Telegram workbench should expose Telegram-specific attachment views while
storage remains provider-neutral.

## Search

### Текущие возможности

- local chat title filter in UI;
- local loaded thread text filter;
- shared Communication search can include Telegram-projected messages by `channel_kind`.
- workspace dialog search UI backed by `GET /api/v1/telegram/chats/search`;
- workspace message search UI backed by `GET /api/v1/telegram/search/messages`;
- explicit provider search command `POST /api/v1/telegram/search/provider`;
- workspace media search UI plus files tab gallery backed by `GET /api/v1/telegram/search/media`.
- downloaded local Telegram media can open in a read-only viewer using projected
  attachment metadata and local blob paths.
- saved-search create/select/delete UI reuses the shared Communication
  `/api/v1/communications/saved-searches` API with `channel_kind=telegram` and
  selected Telegram account scope. No Telegram-specific saved-search route is
  introduced.

### Недостающие маршруты
| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/search/provider` | Не требуется: command endpoint реализован как POST для явного provider search. |

Current search contract notes:

- `GET /api/v1/telegram/chats/search` searches projected Telegram chats by
  title and optional `account_id` scope.
- `GET /api/v1/telegram/search/messages` searches projected Telegram messages by
  text and optional `account_id` / `provider_chat_id` scope. Если `account_id` указан, в запрос
  также инициируется TDLib provider search для актуализации projection.
- `POST /api/v1/telegram/search/provider` now exposes provider search as explicit
  capability-aligned command endpoint and returns full projection response after
  provider attempt.
- The workspace search results panel renders a provider/local/fallback search
  source label derived from selected-account runtime status, so provider-backed
  searches are visible without requiring a separate provider-only UI.
- `GET /api/v1/telegram/search/media` is a projection-backed gallery/filter
  endpoint. It currently supports optional free-text `q`, scope and `kind`,
  attempts TDLib provider message search first when both `q` and `account_id`
  are present, and then returns projected attachment download metadata
  (`tdlib_file_id`, `provider_attachment_id`, `local_path`) so files-tab gallery
  rows can keep preview/download parity outside the currently loaded message
  window. The response includes `source`, `provider_search_attempted` and
  `provider_search_error` to distinguish provider-refresh results from
  projection-only fallback.

## Automation / Policies

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/policies/templates` | List automation templates |
| POST | `/api/v1/policies/templates` | Create/update automation template |
| GET | `/api/v1/policies` | List automation policies |
| POST | `/api/v1/policies` | Create/update automation policy |
| POST | `/api/v1/policies/telegram-send/dry-run` | Telegram send policy/template validation and sanitized audit |

Automation live send is blocked. Dry-run stores preview hashes and redacted metadata.
Telegram composer now also surfaces this route as a read-only preflight panel in
the send dropdown: it loads policies/templates, filters enabled policies for the
selected account/chat, accepts template variables and renders the returned
`rendered_text` plus `rendered_preview_hash` before any live send.

## Audit

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/audit/events?target_id=&actor_id=&after_audit_id=&limit=` | Shared protected audit-event reader |

Telegram audit records must never include:

- message bodies;
- rendered variables;
- media bytes;
- passwords;
- tokens;
- app secrets;
- raw TDLib payloads.

## Calls

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/calls?account_id=&limit=` | List Telegram call metadata rows |
| POST | `/api/v1/calls` | Create/update fixture Telegram call metadata |
| GET | `/api/v1/calls/{call_id}/transcript` | Get latest stored transcript; inspector context now surfaces this through a first-class calls panel with local metadata search |
| POST | `/api/v1/calls/{call_id}/transcript` | Create fixture transcript through fixture STT provider |

Telegram workbench now consumes `GET /api/v1/calls?account_id=&limit=` through
a first-class inspector calls panel for the selected account, and that same
panel can locally search projected call metadata before loading
`GET /api/v1/calls/{call_id}/transcript` as read-only transcript evidence for
the selected projected call. Live call controls and recording flows remain
outside the current UI slice.

### Заблокированные live возможности

- live call control;
- audio capture;
- device selection;
- real STT;
- hidden recording.

## Realtime / Events

### Текущие generic routes

| Method | Path | Описание |
|---|---|---|
| GET | `/api/events/ws?after_position=&hermes_secret=` | Protected WebSocket event stream with replay/heartbeat |
| GET | `/api/events/stream?after_position=` | Protected SSE stream |
| GET | `/api/v1/events?after_position=&limit=&wait_seconds=` | Protected JSON replay/long-poll fallback |
| POST | `/api/v1/events` | Local event API command boundary |
| GET | `/api/v1/events/{event_id}` | Read single event |

### Текущие Telegram realtime contracts

```text
telegram.sync.started
telegram.sync.progress
telegram.sync.completed

telegram.message.created
telegram.message.updated
telegram.message.deleted
telegram.message.visibility_restored

telegram.chat.pinned
telegram.chat.archived
telegram.chat.muted
telegram.chat.updated
telegram.typing.changed
telegram.topic.updated
telegram.media.download.started
telegram.media.download.progress
telegram.media.download.failed
telegram.media.downloaded

telegram.reaction.changed
telegram.command.status_changed
telegram.command.reconciled
```

Frontend cache patching still accepts the legacy `telegram.message.edited`
event name for compatibility with older event-log rows or stale clients, but
new backend emissions use `telegram.message.updated`.

Telegram UI consumers should use the shared frontend realtime bootstrap backed
by `/api/events/ws`, `/api/events/stream` and `/api/v1/events`; the Telegram
workbench no longer opens a dedicated page-local realtime socket.

Current emission scope:

- fixture ingest and manual send -> `telegram.message.created`;
- lifecycle edit/delete/restore -> corresponding `telegram.message.*`;
- TDLib `updateNewMessage` now feeds provider-origin
  `telegram.message.created` through the actor event bridge after ingesting
  the observed message snapshot into the Communication projection;
- TDLib `updateMessageContent` now feeds provider-origin
  `telegram.message.updated` after refreshing projected message text/metadata,
  recording an observed edit version and reconciling matching edit commands;
- TDLib `updateMessageEdited` now also feeds provider-origin
  `telegram.message.updated` after refreshing projected edit metadata such as
  `provider_edit_timestamp` and reply-markup evidence;
- TDLib `updateMessageIsPinned` now also feeds provider-origin
  `telegram.message.updated` after refreshing projected `is_pinned` metadata
  and reconciling matching message-targeted `pin` commands;
- chat/history sync -> `telegram.sync.started/progress/completed/failed`;
- local dialog pin/archive/mute toggles -> `telegram.chat.pinned/archived/muted`;
- local dialog read/unread toggles -> `telegram.chat.updated`;
- TDLib `updateChatReadInbox` and `updateChatUnreadMentionCount` parsing now
  feed provider-observed unread/mention counter updates into
  `telegram.chat.updated` after refreshing projected chat metadata;
- TDLib `updateChatIsMarkedAsUnread` parsing now feeds provider-observed
  unread-flag updates into `telegram.chat.updated` after refreshing projected
  chat metadata and reconciling matching read/unread commands;
- TDLib `updateChatNotificationSettings` parsing now feeds provider-observed
  mute-state updates into `telegram.chat.muted` after refreshing projected chat
  metadata and reconciling matching exact-shape mute/unmute commands;
- TDLib `updateChatPosition` parsing now feeds provider-observed archive/main
  list-state updates into `telegram.chat.archived` and `telegram.chat.pinned`
  after refreshing projected chat metadata and reconciling matching
  archive/unarchive plus pin/unpin commands;
- TDLib `updateDeleteMessages` now feeds provider-origin
  `telegram.message.deleted` through the actor event bridge after recording an
  idempotent `deleted_by_provider` tombstone and reconciling matching delete
  commands;
- TDLib `updateUserChatAction` parsing now feeds `telegram.typing.changed`
  from runtime-started TDLib actors through an actor event bridge into the
  canonical event log and realtime bus;
- TDLib `updateForumTopicInfo` parsing feeds `telegram.topic.updated` through
  the same actor event bridge after resolving the projected chat and upserting
  the topic projection;
- TDLib `updateNewMessage` now feeds provider-origin
  `telegram.message.created` through the actor event bridge after ingesting
  the observed message snapshot into the Communication projection;
- TDLib `updateDeleteMessages` now feeds provider-origin
  `telegram.message.deleted` through the actor event bridge after recording an
  idempotent `deleted_by_provider` tombstone and reconciling matching delete
  commands;
- TDLib `updateMessageInteractionInfo` now feeds provider-origin
  `telegram.reaction.changed` through the actor event bridge after refreshing
  reaction summary metadata, syncing sender-level provider rows and
  reconciling matching self reaction commands;
- local reaction add/remove -> `telegram.reaction.changed`;
- media download request/failure/non-completed TDLib snapshot -> `telegram.media.download.started/progress/failed`;
- completed TDLib media download persistence -> `telegram.media.downloaded`;
- manual send, lifecycle edit/delete/restore, reaction add/remove and current dialog actions (`pin/unpin/archive/unarchive/mute/unmute/read/unread`) -> `telegram.command.status_changed`.
- provider-observed command completion -> `telegram.command.reconciled`.

Current message-event payload shape is richer than the initial coarse contract:

- top-level lifecycle fields such as `provider_chat_id`, `provider_message_id`,
  `version_number`, `reason_class`, `tombstone_id`, `is_pinned`;
- sanitized projected `message` snapshot for created/updated/deleted/visibility
  restore events;
- sanitized projected `chat` snapshot for message events when a corresponding
  `telegram_chats` projection row exists;
- deterministic `telegram_chat_id` derived from projection row or
  `account_id + provider_chat_id`, so frontend pinned-message caches can patch
  even when a dedicated `telegram_chats` row has not been materialized yet.

Current frontend realtime runtime-state usage also relies on:

- `event.metadata.account_id` for account-scoped runtime query matching;
- `telegram.sync.*` payload fields such as `scope`, `status`, `synced_count`,
  `has_more`, `provider_chat_id`;
- `telegram.command.status_changed` / `telegram.command.reconciled` payload
  fields such as `command_id`, `status`, `provider_chat_id`,
  `telegram_chat_id`, `message_id`, `reconciliation_status`,
  `provider_observed_at`, `reconciled_at` and `next_attempt_at`.

Current local dialog command payloads are also richer than the original coarse
command-status contract:

- `action` for the local dialog operation kind;
- sanitized projected `chat` snapshot for current local dialog actions, so
  frontend chat list/detail caches can patch immediately after pin/archive/
  mute/read/unread changes.

Current local dialog flag events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- one changed flag: `is_pinned`, `is_archived` or `is_muted`;
- sanitized projected `chat` snapshot.

Current local read/unread chat update events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `action`: `mark_read` or `mark_unread`;
- sanitized projected `chat` snapshot after unread counter recomputation.

Current provider-observed unread chat update events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `action`: `provider_unread_update`;
- optional `unread_count`;
- optional `unread_mention_count`;
- optional `last_read_inbox_provider_message_id`;
- sanitized projected `chat` snapshot after provider counter projection.

Current provider-observed unread-flag chat update events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `action`: `provider_marked_as_unread_update`;
- `is_marked_as_unread`;
- sanitized projected `chat` snapshot after provider flag projection.

Current provider-observed mute chat events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `is_muted`;
- `use_default_mute_for`;
- `mute_for`;
- sanitized projected `chat` snapshot after provider notification projection.

Current provider-observed archive chat events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `is_archived`;
- `list_kind`;
- optional `provider_folder_id`;
- `order`;
- `is_pinned`;
- sanitized projected `chat` snapshot after provider position projection.

Current media download lifecycle events carry:

- `provider_chat_id`;
- `provider_message_id`;
- `tdlib_file_id`;
- `provider_attachment_id`;
- `download_state`;
- progress fields (`expected_size_bytes`, `downloaded_size_bytes`,
  `is_downloading_active`, `is_downloading_completed`) when TDLib returns a
  non-completed file snapshot;
- sanitized projected `message` and `chat` snapshots only on completed
  `telegram.media.downloaded` events after local blob/attachment persistence.

When the database-backed event log is configured, these same typed events are
also appended into canonical `event_log`, so they are available through:

- `GET /api/events/ws?after_position=&hermes_secret=`;
- `GET /api/events/stream?after_position=`;
- `GET /api/v1/events?after_position=&limit=&wait_seconds=`.

Missing parity:

- provider-observed folder labels/mutations beyond current main/archive pin-state observation;
- true message-level read receipt records beyond projected chat counters;
- custom mute-shape reconciliation beyond the current exact TDLib request contract;
- richer Telegram media retry/dead-letter UX beyond request/progress/failed/completed events;
- fully patched provider-dialog cache surfaces without fallback invalidation.

## Frontend API Client

Current frontend client:

```text
frontend/src/domains/telegram/api/telegram.ts
frontend/src/domains/telegram/queries/useTelegramQuery.ts
```

Covered:

- capabilities;
- accounts;
- chats;
- messages;
- runtime;
- QR login;
- send;
- media download;
- dry-run;
- calls.

Not covered because backend routes do not exist:

- topic/provider-command parity beyond current lifecycle, reaction, pin, archive, mute, read/unread and search execution paths;
- media gallery;
- provider-observed folder/archive and notification/mute reconciliation;
- Telegram-specific realtime cache patching.
