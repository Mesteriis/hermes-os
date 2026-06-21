# Telegram API Reference — Conversations

См. также: [API Index](../api.md), [Foundation](foundation.md) и [Media and Search](media-search.md).

## Chats / Dialogs

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/provider-conversations?account_id=&limit=` | Projected Telegram chats ordered by last activity |
| POST | `/api/v1/communications/provider-sync/chats` | Sync account chats through fixture or TDLib runtime |

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
| GET | `/api/v1/communications/provider-conversations/{chat_id}` | Projection-backed selected-chat detail |
| GET | `/api/v1/communications/provider-conversations/{chat_id}/members?query=&role=&limit=&cursor=` | Provider roster rows when synced; labeled message-sender fallback otherwise |
| POST | `/api/v1/communications/provider-conversations/{chat_id}/members/sync` | TDLib-backed provider roster sync for supergroups/channels and metadata-backed private/saved-message roster hydration |
| POST | `/api/v1/communications/provider-conversations/join` | Queue provider chat join through durable outbox (`command_kind=join`) |
| POST | `/api/v1/communications/provider-conversations/{chat_id}/leave` | Queue provider chat leave through durable outbox (`command_kind=leave`) |
| GET | `/api/v1/communications/provider-conversations/{chat_id}/pinned-messages?limit=` | Projection-backed pinned-message list for the selected chat |
| GET | `/api/v1/communications/provider-conversations/search?q=&account_id=` | Projection-backed dialog search |
| GET | `/api/v1/communications/provider-conversation-folders?account_id=` | Projection-backed Telegram folder/chat-list filters derived from `telegram_chats.metadata.folder_name` / `folder_labels`, including optional `provider_folder_id` when TDLib folder metadata is known |
| POST | `/api/v1/communications/provider-conversations/{chat_id}/pin` | Provider/local pin command |
| POST | `/api/v1/communications/provider-conversations/{chat_id}/archive` | Provider/local archive command |
| POST | `/api/v1/communications/provider-conversations/{chat_id}/mute` | Mute/unmute command |
| POST | `/api/v1/communications/provider-conversations/{chat_id}/read` | Projected mark-read command; when `last_read_inbox_provider_message_id` is provided, queued TDLib `viewMessages(force_read=true)` targets that provider message for provider-side read reconciliation |
| POST | `/api/v1/communications/provider-conversations/{chat_id}/unread` | Projected mark-unread command plus queued TDLib manual-unread set |

Current implementation of pin/archive/mute/read/unread updates projected local
chat metadata and records durable command rows with realtime command-status
events. Mark-read commands can now use TDLib `viewMessages` when a target
`last_read_inbox_provider_message_id` is supplied, while mark-unread continues
to use `toggleChatIsMarkedAsUnread`; archive/unarchive uses `addChatToList`, and
mute/unmute uses `setChatNotificationSettings`. TDLib unsolicited
`updateChatIsMarkedAsUnread`, `updateChatNotificationSettings` and
`updateChatPosition` updates now project provider-observed unread-flag, mute
state, archive state and dialog pin state back into `telegram_chats.metadata`,
emit `telegram.chat.updated` / `telegram.chat.muted` / `telegram.chat.archived`
/ `telegram.chat.pinned`, and reconcile matching `mark_read` / `mark_unread`,
`pin` / `unpin`, exact-shape `mute` / `unmute`, and `archive` / `unarchive`
commands into provider-observed `completed`, while contradictory provider state
is recorded as `failed` with `reconciliation_status=mismatch` and
`telegram.command.reconciled`.
Dialog write routes also append `api_audit_log` records with explicit
provider-write capability metadata; targeted mark-read requests include the
provider message id in audit metadata without storing message body content.
Extended provider semantics such as true message-level receipt rows, custom
mute shapes outside the current exact TDLib request contract and folder
semantics beyond current add/remove/reassign paths are outside the base
closure.
`POST /api/v1/communications/provider-sync/chats` now also
hydrates TDLib folder labels through `getChatFolder`, persists
`folder_labels` / `provider_folder_id` into chat metadata, and runtime
`updateChatFolders` events emit `telegram.folders.updated` so the folders query
can patch without polling.
Member sync currently uses TDLib `getSupergroupMembers` for chats with
`tdlib_supergroup_id`, TDLib `getBasicGroup` + `getBasicGroupFullInfo` for
basic groups, and private/Saved Messages chat metadata via
`tdlib_private_user_id` for TDLib private dialogs. All three paths store
provider roster rows in `telegram_chat_participants`, emit
`telegram.participant.updated`, and leave admin commands open. Exhaustive roster
sync now also marks previously projected TDLib members that disappeared from the
provider snapshot as `absent_exhaustive`, so stale rows stop appearing in
`GET /members` and frontend member caches can remove them from the active
roster view; provider rows already marked `left` / `banned` are likewise
excluded from that active roster surface. Join/leave commands
are queued with command status events and active TDLib actors dispatch
`joinChat`/`leaveChat`; dispatch ACK is not completion, so these commands
remain awaiting provider observation until a later roster/chat reconciliation
pass confirms the membership state. Self `join` commands can now complete when
TDLib member sync observes the selected account's own active
`user:<telegram_id>` roster row and emits `telegram.command.reconciled`. Self
`leave` can now also complete when TDLib roster sync explicitly returns the
selected account in inactive `left` / `banned` state, or when an exhaustive
provider roster snapshot no longer contains that self member id. TDLib history sync can
now also reconcile self `join`/`leave` when explicit
`messageChatAddMembers` or `messageChatDeleteMember` service messages name the
selected account. `leave` is still not inferred from absence in a non-exhaustive
recent roster page. Supergroup member sync now also merges TDLib
`supergroupMembersFilterAdministrators`, so provider roster updates can observe
admin role/permission changes even when admins are absent from recent pages;
silent membership state changes still need stronger provider evidence. The Telegram workbench action rail now consumes
this dedicated folders route instead of recomputing folder groups only from
the currently loaded chat list, and the Members inspector exposes
capability-gated Join/Leave controls.

## Topics / Forums

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/provider-conversations/{telegram_chat_id}/topics?limit=` | Fetch forum topics (DB-backed), with optional live TDLib sync refresh |
| POST | `/api/v1/communications/provider-conversations/{telegram_chat_id}/topics` | Queue forum-topic create command |
| GET | `/api/v1/communications/provider-topics/{topic_id}` | Read topic details |
| POST | `/api/v1/communications/provider-topics/{topic_id}/close` | Queue forum-topic close/reopen command |
| GET | `/api/v1/communications/provider-topics/{topic_id}/messages` | Read topic-scoped timeline by `forum_topic_id` metadata |
| GET | `/api/v1/communications/provider-topics/search?q=&telegram_chat_id=&limit=` | Search forum topics within a chat by title |

The frontend Topics tab consumes these routes through TanStack Query and now
renders projected `is_pinned`, `is_closed`, `unread_count`, `provider_topic_id`
and `last_message_at` fields as read-only topic state/provider labels. This is
projection evidence only; provider mutations now flow through durable commands rather than direct projection writes.
TDLib `updateForumTopicInfo` runtime updates now refresh the same
`telegram_topics` projection fields for title, icon, pinned and closed state,
then emit `telegram.topic.updated` with a sanitized projected topic snapshot.
Frontend Topics tab now also exposes capability-gated topic create and
close/reopen controls. `topic_create` commands reconcile immediately from the
TDLib `createForumTopic` response, while `topic_close` / `topic_reopen`
commands stay `awaiting_provider` until `updateForumTopicInfo` observes the
closed-state transition and emits `telegram.command.reconciled`.

### Текущие lifecycle/history маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/communications/provider-messages/{message_id}/edit` | Record append-only local edit version and command metadata |
| POST | `/api/v1/communications/provider-messages/{message_id}/delete` | Record tombstone evidence and command metadata |
| POST | `/api/v1/communications/provider-messages/{message_id}/restore-visibility` | Record local visibility restore event, command metadata and redacted audit |
| POST | `/api/v1/communications/provider-messages/{message_id}/pin` | Record local pin/unpin projection state, command metadata and redacted audit; active TDLib actors execute queued provider pin/unpin |
| POST | `/api/v1/communications/provider-messages/{message_id}/reply` | Send TDLib reply message through the active QR-authorized runtime |
| POST | `/api/v1/communications/provider-messages/{message_id}/forward` | Forward a source Telegram message through the active QR-authorized runtime |
| GET | `/api/v1/communications/provider-messages/{message_id}/versions` | List observed/local message versions |
| GET | `/api/v1/communications/provider-messages/{message_id}/tombstones` | List message tombstones |
| GET | `/api/v1/communications/provider-messages/{message_id}/raw` | Sanitized raw provider evidence record for the projected Telegram message |
| GET | `/api/v1/communications/provider-messages/{message_id}/reply-chain` | Read current reply-chain projection |
| GET | `/api/v1/communications/provider-messages/{message_id}/forward-chain` | Read current forward-chain projection |
| GET | `/api/v1/integrations/telegram/commands?account_id=&limit=` | List durable Telegram command rows; inspector context now surfaces them as a local audit panel with current-chat filter and text search |
| POST | `/api/v1/integrations/telegram/commands/{command_id}/retry` | Manually requeue an eligible failed/dead-letter/retrying provider command row through the durable outbox runtime |

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
| POST | `/api/v1/communications/provider-conversations/{telegram_chat_id}/topics` | Topic write command (create forum topic) |
| POST | `/api/v1/communications/provider-topics/{topic_id}/close` | Topic write command (close/reopen forum topic) |

## Messages

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/provider-messages?account_id=&provider_chat_id=&limit=` | Recent projected Telegram messages |
| POST | `/api/v1/communications/provider-messages` | Ingest fixture Telegram message |
| POST | `/api/v1/communications/provider-sync/history` | Sync selected chat history |
| POST | `/api/v1/communications/provider-messages/send` | Manual text send through fixture or TDLib QR-authorized runtime |
| POST | `/api/v1/communications/provider-messages/{message_id}/mark-read` | Dedicated provider mark-read command for the selected message; resolves projected chat/message identifiers and queues targeted TDLib `viewMessages(force_read=true)` through the dialog outbox path |

`POST /api/v1/communications/provider-sync/history` supports:

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
| POST | `/api/v1/communications/provider-messages/{message_id}/reactions` | Add reaction |
| DELETE | `/api/v1/communications/provider-messages/{message_id}/reactions` | Remove reaction |
| GET | `/api/v1/communications/provider-messages/{message_id}/reactions` | Current reaction projection |

Current implementation persists local reaction projection rows, durable command
metadata and redacted audit records for add/remove, exposes degraded local-write
capability states, and emits both `telegram.reaction.changed` and
`telegram.command.status_changed`; active TDLib actors execute queued
provider reaction commands, while provider-origin reaction reconciliation is
selected-account scoped. TDLib message ingestion now projects aggregate emoji reaction counts
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
actor absence is preserved as provider evidence rather than selected-account
command success. Runtime-started TDLib actors now also forward unsolicited
`updateMessageInteractionInfo` through the actor event bridge, refresh the
projected reaction summary plus sender rows and emit provider-origin
`telegram.reaction.changed`.
