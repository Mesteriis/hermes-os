# Telegram Gap Analysis

Status date: 2026-06-17.

Labels:

- `IMPLEMENTED` — реализовано и подтверждено текущими files/tests/docs audit.
- `PARTIAL` — есть meaningful foundation, но production scope не закрыт.
- `BROKEN` — реализация есть, но current evidence показывает, что она не работает.
- `MISSING` — durable implementation в текущем репозитории не найден.
- `REGRESSION` — current behavior хуже ранее задокументированного поведения.

Evidence sources: `backend/src/integrations/telegram/`, `backend/tests/telegram.rs`,
`backend/migrations/0020_create_v4_telegram_policy_calls.sql`,
`backend/migrations/0058_allow_empty_telegram_tdlib_message_bodies.sql`,
`backend/src/platform/audit/telegram.rs`, `backend/src/platform/calls/`,
`frontend/src/domains/telegram/`, ADR-0050, ADR-0083, ADR-0091.

No confirmed `BROKEN` or `REGRESSION` capability was found during this audit.
Most gaps are `PARTIAL` or `MISSING`.

## Accounts / Runtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| Fixture accounts | IMPLEMENTED | `POST /api/v1/telegram/accounts/fixture`, fixture tests and provider kind constraints exist. |
| Multiple accounts | IMPLEMENTED | Account list and account-scoped runtime/status paths exist. |
| Live user account metadata | PARTIAL | Live setup stores metadata/secret refs; usable runtime depends on QR-authorized TDLib and local native resources. |
| QR authorization | PARTIAL | QR start/status/password/cancel exists, and the account manager now surfaces that TDLib QR flow with polling, 2FA password submission and suggested-account application back into the local setup form; live readiness still depends on TDLib runtime and app credentials. |
| Bot account setup | PARTIAL | Bot token can be stored via host-vault reference; live Bot API runtime is missing. |
| Account-scoped capabilities | IMPLEMENTED | `GET /api/v1/telegram/accounts/{account_id}/capabilities` now returns the same detailed capability matrix with `account_scope` metadata plus bot/QR/runtime/lifecycle-aware overrides for the selected account, and the inspector `About` tab renders that payload as a read-only matrix. |
| Logout | IMPLEMENTED | Account logout marks lifecycle state and stops actor with audit. |
| Remove account | IMPLEMENTED | Removal preserves raw records/messages and records audit. |
| Runtime health | PARTIAL | `GET /api/v1/telegram/runtime/status?account_id=` now returns TDLib path/probe diagnostics, split API credential readiness and derived runtime blockers for the selected account. `POST /api/v1/telegram/runtime/stop` and `/restart` are wired through frontend action rail controls; deeper native dependency remediation remains incomplete. |
| Session import/export | MISSING | No bundle schema/API/UI found. |
| Proxy / MTProxy / SOCKS5 | MISSING | No proxy profile schema/API/runtime command found. |

## Dialogs / Chat Management

| Capability | Status | Evidence / Gap |
|---|---|---|
| Dialogs | PARTIAL | `telegram_chats` projection, chat list/detail endpoints, local projected pin/archive/mute/read actions, realtime chat snapshot patching and active TDLib execution for read/unread, archive/unarchive and mute/unmute exist. TDLib `updateChatIsMarkedAsUnread`, `updateChatNotificationSettings` and `updateChatPosition` now reconcile provider-observed unread-flag, exact mute/unmute state, archive/unarchive state and dialog pin/unpin state into chat metadata plus command completion. Provider folder labels/mutations and true message-level read receipt reconciliation are still missing. |
| Private chats | PARTIAL | `private` chat kind exists and selected history sync works; no full provider parity. |
| Groups | PARTIAL | `group` chat kind exists; provider member roster and join/leave command foundation exists for TDLib supergroups/channels, but admin/group lifecycle parity remains incomplete. |
| Supergroups | PARTIAL | TDLib supergroups remain API-compatible as `group`/`channel`, but chat sync now preserves `tdlib_chat_type`, `tdlib_supergroup_id`, `is_supergroup`, `is_channel_supergroup` and `is_forum` metadata and the inspector renders it. Provider permissions/admin lifecycle remain missing. |
| Channels | PARTIAL | `channel` chat kind exists; subscribe/unsubscribe/post admin actions missing. |
| Saved Messages | PARTIAL | TDLib private chats whose `chatTypePrivate.user_id` matches the selected account external Telegram user id now project `is_saved_messages` metadata and the UI renders a Saved Messages marker. Provider-observed reconciliation and non-TDLib fallback remain missing. |
| Pinned chats | PARTIAL | Local metadata-backed pin/unpin routes exist for projected chats, and TDLib `updateChatPosition` now projects provider-observed `is_pinned`, emits `telegram.chat.pinned`, and reconciles matching dialog `pin` / `unpin` commands when main/archive list state is observed. Folder-label parity, folder mutations and message-level pinned-state reconciliation remain missing. |
| Archived chats | PARTIAL | Local metadata-backed archive/unarchive routes exist and queued commands dispatch to TDLib `addChatToList`. TDLib `updateChatPosition` now projects `tdlib_chat_positions` / `is_archived`, emits `telegram.chat.archived`, and reconciles matching `archive` / `unarchive` commands when main/archive list presence is observed. Folder-label parity and archive behavior beyond observed TDLib list state remain missing. |
| Chat folders | PARTIAL | `GET /api/v1/telegram/folders?account_id=` now returns projection-backed local/Telegram folder filters derived from `telegram_chats.metadata.folder_name`. TDLib chat sync and `updateChatPosition` now preserve provider `folder_ids` as technical projection metadata, but provider folder labels, folder sync routes and folder mutation routes are still missing. |
| Mute/unmute | PARTIAL | Local metadata-backed mute/unmute routes exist and queued commands dispatch to TDLib `setChatNotificationSettings`. TDLib `updateChatNotificationSettings` now projects `tdlib_notification_settings` / `is_muted`, emits `telegram.chat.muted`, and reconciles matching exact-shape `mute` / `unmute` commands into `completed`. Provider archive/folder parity, custom mute-shape handling and Bot API coverage remain missing. |
| Unread counters | PARTIAL | Chat metadata now stores projection-backed `unread_count`, `POST /api/v1/telegram/chats/{chat_id}/read` and `/unread` mutate local read state, unread badges/filters use that value, active TDLib actors execute queued manual-unread toggles, projected unread mention totals are derived from per-message mention metadata, and TDLib `updateChatReadInbox` / `updateChatUnreadMentionCount` updates now reconcile provider-observed unread and mention counters through `telegram.chat.updated`. True message-level read receipts and richer read-position history are still missing. |
| Typing indicators | PARTIAL | `telegram.typing.changed` is now reserved in the realtime contract, TDLib `updateUserChatAction` parsing exists, runtime-started TDLib actors emit sanitized typing events, frontend invalidation handles the event family, cache patching projects expiry-bounded `active_typing` onto chat metadata, and chat list/thread header render that state; provider-wide reconciliation and richer expiry/clear semantics remain missing. |
| Participants / Members | PARTIAL | Migration 0089 adds `telegram_chat_participants`; `POST /api/v1/telegram/chats/{chat_id}/members/sync` fetches TDLib `getSupergroupMembers` for supergroups/channels, and `GET /members` returns provider roster rows first with `source=tdlib`. Message-sender aggregation remains only as explicit `source=message_heuristic` fallback when no provider roster exists. `POST /api/v1/telegram/chats/join` and `POST /api/v1/telegram/chats/{chat_id}/leave` now enqueue durable provider-write commands and active TDLib actors dispatch `joinChat`/`leaveChat`; member sync can reconcile self `join` commands when TDLib returns the account's own active roster member id, and history sync can reconcile self `join`/`leave` when TDLib service messages explicitly name the selected account. Basic-group/private roster parity, pagination beyond the first TDLib recent page, silent/admin lifecycle reconciliation and admin mutations remain missing. |
| Roles / Permissions | PARTIAL | TDLib chat sync preserves boolean `chat.permissions` fields in `telegram_chats.metadata.tdlib_permissions`, and participant sync now stores member `role`, `status`, `is_admin`, `is_owner` and status-derived permission payloads in `telegram_chat_participants`. Admin mutation commands and full provider lifecycle reconciliation remain missing. |
| Join / Leave | PARTIAL | TDLib service messages for add members, join-by-link, join-by-request and delete-member project read-only `telegram_join_leave` structured evidence, the reference panel renders it, and provider join/leave commands now use the durable outbox plus TDLib `joinChat`/`leaveChat` dispatch path. Self `join` commands can now complete from provider-observed roster presence during TDLib member sync, and self `join`/`leave` can reconcile from TDLib `messageChatAddMembers` / `messageChatDeleteMember` service-message evidence during history sync; both paths emit `telegram.command.reconciled`. Absence in the recent member roster is still not treated as proof, so silent/admin lifecycle remains missing. |

## Messaging

| Capability | Status | Evidence / Gap |
|---|---|---|
| Topics | PARTIAL | Migration 0086 adds `telegram_topics` table; `upsert_topic`/`list_topics`/`get_topic` queries exist; three API routes (`GET /chats/{id}/topics`, `GET /topics/{id}`, `GET /topics/{id}/messages`) are wired; frontend `TelegramTopic` type, `useTelegramTopicsQuery`/`useTelegramTopicMessagesQuery` hooks, and Topics tab in `TelegramThreadSideSections` are in place; `GetForumTopics` runtime command calls TDLib `getForumTopics` on demand and upserts results via `sync_forum_topics` on the manager — topics projection is now populated for live TDLib accounts on first request. TDLib `updateForumTopicInfo` updates now upsert title/pin/closed projection fields and emit sanitized `telegram.topic.updated` events that patch topic list/search caches. Topic write commands (create/close/reopen) and richer topic-scoped unread/live state reconciliation remain missing. |
| Replies | PARTIAL | Reply command flow is now complete end-to-end: `POST /api/v1/telegram/messages/{message_id}/reply` records the command, sends `sendMessage` with `reply_to` via TDLib through `ReplyMessage` runtime variant and `send_reply_message` on the manager, and the frontend wires a composer reply-preview banner, Reply action in message action bars, and `useTelegramSendActions` composable routing send vs reply in TelegramPage; multi-hop reply-thread model and deeper graph navigation are still missing. |
| Reply chains | PARTIAL | `GET /api/v1/telegram/messages/{message_id}/reply-chain` now returns projected source/target message summaries and the reference panel can reopen/filter those summaries inside the thread; provider-side reply creation is now supported via the reply command flow above, but deeper graph navigation beyond single-hop is still missing. |
| Forwards | PARTIAL | Forward-chain projection API exists, thread UI can read origin attribution, and `POST /api/v1/telegram/messages/{message_id}/forward` now sends provider-side forwards through the TDLib QR-authorized runtime with frontend action wiring. Queued outbox forward result projection and deeper forward-target selection UX are still missing. |
| Forward chains | PARTIAL | `GET /api/v1/telegram/messages/{message_id}/forward-chain` now returns local source-message summaries plus origin metadata, and the reference panel can filter these rows while showing richer origin/source text summaries; deeper chain UX beyond single-hop is still missing. |
| Mentions | PARTIAL | Ingest now derives per-message `mention_count` from TDLib formatted-text entities or fallback `@handle` parsing, preserves `mentions` / `mentions_detected_by`, chat metadata recomputes unread mention totals, chat/thread header badges surface aggregate state, and message bubbles render read-only mention chips from projected source evidence. Provider-identity targeting and richer mention semantics are still missing. |
| Pinned messages | PARTIAL | `GET /api/v1/telegram/chats/{chat_id}/pinned-messages` now backs a chat-scoped pinned-message tab from projection metadata, pinned/search reopen can focus a projected target row in the selected thread even if it is outside the currently loaded window, and `POST /api/v1/telegram/messages/{message_id}/pin` now records local pin/unpin projection state, durable command rows, redacted audit, thread-level pin UI and TDLib provider execution through the active command executor. TDLib `updateMessageIsPinned` now refreshes projected `is_pinned` state, emits `telegram.message.updated`, and reconciles message-targeted `pin` commands from provider-observed state; richer chat-level pinned ordering parity remains partial. |
| Raw provider evidence | PARTIAL | `GET /api/v1/telegram/messages/{message_id}/raw` returns the append-only `communication_raw_records` row for projected Telegram messages with recursive secret-key redaction; the thread reference panel now reads it through a dedicated TanStack Query hook and shows sanitized payload/provenance, while richer diff/search UX over raw evidence is still missing. |
| Message links | PARTIAL | Public username-backed messages now preserve synced chat usernames during message ingest and project `message_link` / `message_link_kind` metadata as `https://t.me/<username>/<id>`; the thread reference panel renders that provider permalink when present. Private/group/internal deep links and provider-observed permalink reconciliation are still missing. |
| Polls | PARTIAL | TDLib `messagePoll` raw payloads now project read-only `telegram_poll` metadata with question, options, vote counts and closed state; the thread reference panel renders that structured evidence. Provider poll voting and live reconciliation are still missing. |
| Locations | PARTIAL | TDLib `messageLocation` and `messageVenue` raw payloads now project read-only `telegram_location` metadata with coordinates plus venue title/address when present; the thread reference panel renders that structured evidence. Map preview/provider reconciliation are still missing. |
| Contacts | PARTIAL | TDLib `messageContact` raw payloads now project read-only `telegram_contact_card` metadata and the thread reference panel renders it as source evidence. This does not create Contacts, Persona lifecycle or identity resolution; richer contact-card UX remains missing. |

## Message Lifecycle

| Capability | Status | Evidence / Gap |
|---|---|---|
| Send text | PARTIAL | Manual text send works through fixture/TDLib QR runtime and redacted audit; queued outbox missing. |
| Send media | PARTIAL | `POST /api/v1/communications/attachments/import` stores local files as shared Communication blobs/import rows, `POST /api/v1/telegram/media/upload` queues `send_media` provider-write commands from `attachment_id`/`blob_id`, TDLib builders cover photo/video/document/audio/voice/sticker/animation, the outbox executor resolves local blobs for provider send, and the composer routes selected files through that command path. Bot API media send, album-aware send, richer progress/cache patch UX and complete provider reconciliation remain missing. |
| Edit | PARTIAL | `POST /api/v1/telegram/messages/{message_id}/edit` records append-only edit versions and a provider-write command row; active TDLib actors execute queued provider edits through the command executor. Provider-observed edit reconciliation and dead-letter UI are still missing. |
| Delete | PARTIAL | `POST /api/v1/telegram/messages/{message_id}/delete` records tombstones and a provider-write command row; active TDLib actors execute queued provider deletes through the command executor. Provider-observed delete reconciliation and dead-letter UI are still missing. |
| Restore visibility | PARTIAL | `POST /api/v1/telegram/messages/{message_id}/restore-visibility` now records local visibility restoration, a durable command row, redacted audit and `telegram.command.status_changed`, but provider parity is still missing. |
| Mark read/unread | PARTIAL | `POST /api/v1/telegram/chats/{chat_id}/read` and `/unread` now record provider-write dialog command rows, recompute projected unread counters, and dispatch queued TDLib `toggleChatIsMarkedAsUnread` commands for active actors. Provider-observed reconciliation and true message-level `viewMessages` read receipt parity are still missing. |
| Message history | PARTIAL | Selected/older history sync exists; observed edit versions/diffs/deletion history missing. |
| Edit versions | PARTIAL | `telegram_message_versions` plus list endpoint exist, and the per-message reference panel now renders local edit-history summaries with relative text-length deltas between captured versions, but provider-observed diffs are still missing. |
| Tombstones | PARTIAL | `telegram_message_tombstones` plus list endpoint exist, and the per-message reference panel now renders provider-delete vs local-visibility state summaries, but provider reconciliation/history UX is still missing. |
| Provider command retry | PARTIAL | `telegram_provider_write_commands` now supports durable outbox fields for due timestamps, execution locks, `dead_letter`, provider-observed state and reconciliation status. The executor atomically claims queued/retrying rows, applies exponential backoff, recovers stale locked executions, dead-letters permanent/exhausted failures and exposes manual retry through `POST /api/v1/telegram/commands/{command_id}/retry`; inspector UI can retry failed/dead-letter rows. Media upload and participant join/leave commands now use the same outbox path; self `join` can reconcile from TDLib roster presence and self `leave` can reconcile from explicit TDLib service-message evidence. Admin commands, Bot API commands, silent/admin participant lifecycle reconciliation and broader provider-observed reconciliation remain missing. |

## Reactions

| Capability | Status | Evidence / Gap |
|---|---|---|
| Add reaction | PARTIAL | Add-reaction endpoint, capability-gated local projection, durable command row, redacted audit, thread picker UX, read-only evidence consumption of `GET /api/v1/telegram/messages/{message_id}/reactions`, and TDLib provider execution through the active command executor exist. TDLib history sync and unsolicited `updateMessageInteractionInfo` runtime updates can now reconcile matching self `react` commands from provider-observed chosen emoji state; non-self actor parity is still missing. |
| Remove reaction | PARTIAL | Remove-reaction endpoint, capability-gated local projection, durable command row, redacted audit, thread remove UX and TDLib provider execution through the active command executor exist. TDLib history sync and unsolicited `updateMessageInteractionInfo` runtime updates can now reconcile matching self `unreact` commands when the target emoji is absent from chosen reaction state; non-self actor parity is still missing. |
| Reaction sync | PARTIAL | Local reaction rows, queued provider execution, `telegram.reaction.changed` events, TDLib aggregate emoji reaction summary projection and sender-level `recent_reactions` upsert from `interaction_info.reactions` exist. Current-actor emoji rows now also deactivate on provider-observed absence via aggregate `is_chosen` state, and runtime `updateMessageInteractionInfo` now emits provider-origin reaction events after refreshing metadata/rows, but non-self actor removal/absence reconciliation and custom reaction row mapping are still missing. |
| Reaction summary | PARTIAL | Message list responses include reaction summary, TDLib aggregate emoji and custom reaction metadata can be projected from source evidence, sender-level recent reactions can populate local reaction rows, current-actor chosen emoji state can reconcile local row activity, and the Vue thread can render emoji chips plus custom aggregate evidence. Non-self actor removal reconciliation and custom reaction row mapping remain missing. |

## Media

| Capability | Status | Evidence / Gap |
|---|---|---|
| Photos | PARTIAL | TDLib `messagePhoto` caption text can be parsed and raw payload retained; gallery exists and downloaded images can open in the viewer, but richer grouping/provider parity is missing. |
| Videos | PARTIAL | TDLib `messageVideo` caption text can be parsed and raw payload retained; downloaded videos can open in the viewer, but provider parity and richer gallery UX are missing. |
| Documents | PARTIAL | TDLib `messageDocument` caption text can be parsed; download can create attachment row when file metadata is supplied. |
| Voice messages | PARTIAL | `messageVoiceNote` text handling exists, and the thread now exposes a dedicated voice tab with inline playback for downloaded local voice/audio attachments plus download fallback for remote-only items, but record/send/transcript flows are still missing. |
| Video notes | PARTIAL | TDLib `messageVideoNote` raw payloads now project remote `video_note` attachment metadata with `tdlib_file_id`, filename, MIME type and download state; existing files/media viewer surfaces can download and preview after local download. Recording/send/transcript/provider parity is still missing. |
| Audio | PARTIAL | `messageAudio` caption text can be parsed; downloaded audio can open in the viewer, and the thread voice tab can play downloaded local audio attachments inline, but richer playback/gallery UX remains missing. |
| Stickers | PARTIAL | TDLib `messageSticker` raw payloads now project remote `sticker` attachment metadata with format-derived MIME type, filename, emoji metadata and `tdlib_file_id`; existing files/media viewer surfaces can download and preview downloaded WebP stickers. Animated sticker/provider parity remains partial. |
| GIF / animation | PARTIAL | TDLib `messageAnimation` raw payloads now project remote `animation` attachment metadata with `tdlib_file_id`, filename, MIME type and download state; existing files/media viewer surfaces can download and preview after local download. Rich GIF grouping/provider parity remains missing. |
| Media albums | PARTIAL | TDLib message ingestion now projects `media_album_id` and `media_album_key` from raw source evidence, and the Files tab groups loaded selected-chat messages by album. Provider album sync beyond loaded history and album-aware send/upload remain missing. |
| Upload media | PARTIAL | Backend provider-side upload/send exists through shared local attachment import plus `send_media` outbox commands; command/audit/realtime events are recorded, malicious imports are rejected before insertion, and the composer exposes a capability-gated local file picker that imports/uploads through the command model. Bot API media send, album-aware upload and richer upload progress/cache patch UX remain missing. |
| Media gallery | PARTIAL | Files tab now renders a Telegram-scoped gallery from `GET /api/v1/telegram/search/media`, merges query-backed media rows with loaded attachment hints so older downloaded media can still preview/download outside the current message window, and workspace media hits can reopen the owning message context in the current dialog while surfacing projected preview/download readiness, but richer grouping/provider parity is still missing. |
| Media search | PARTIAL | `GET /api/v1/telegram/search/media` now supports projection-backed scope/kind/free-text filtering plus attachment download metadata (`tdlib_file_id`, `provider_attachment_id`, `local_path`) for files-tab/media-viewer parity; when `q` and `account_id` are present it first attempts TDLib provider message search and then filters the refreshed projection, returning source/fallback metadata that the workspace media results panel now renders. Remote attachment preview parity and richer provider media filters are still missing. |

## Attachments

| Capability | Status | Evidence / Gap |
|---|---|---|
| Preview | PARTIAL | Telegram workbench now has a dedicated media viewer for downloaded local files, files-tab gallery rows can preserve projected `local_path` metadata even when the source message is outside the loaded thread window, media-search cards surface projected preview/download readiness, and downloaded projected attachments with a Communication `attachment_id` can reuse the shared safe text/image attachment preview endpoint. Remote/not-yet-downloaded attachments still have no real preview. |
| Download | PARTIAL | TDLib media download endpoint exists and fixture runtime fails closed with replayable `telegram.media.download.started/failed` events; non-completed TDLib snapshots emit `telegram.media.download.progress`; completed downloads persist blob/attachment rows, patch projected Telegram message attachment metadata and emit `telegram.media.downloaded`; files-tab gallery rows can reuse projected TDLib/provider attachment metadata to retry downloads outside the loaded thread window, but successful download still requires a TDLib actor. |
| Search | PARTIAL | Telegram workbench now has query-backed dialog/message/media search plus a chat-scoped media gallery; message search triggers provider TDLib search when account scope is available, the UI labels provider/local/fallback search source, Files tab reuses the shared Communication attachment index for downloaded Telegram attachments, and the thread surface now reuses shared Communication saved searches with `channel_kind=telegram`. Provider-side remote attachment search and richer preview/search UX beyond current projection filters are still missing. |
| Deduplication | PARTIAL | Blob SHA-256 dedup exists in shared storage; no Telegram-specific duplicate UX. |
| Scanner-backed clean verdict | MISSING | No real scanner backend marks Telegram attachments `clean`. |
| Persisted preview artifacts | MISSING | No durable Telegram preview artifact pipeline found. |

## Search

| Capability | Status | Evidence / Gap |
|---|---|---|
| Message search | PARTIAL | Workspace UI now uses `GET /api/v1/telegram/search/messages` for query-backed Telegram message results; with account scope, query hooks call provider TDLib search before returning the refreshed projection, the results panel labels the search source, and Telegram can create/select/delete shared Communication saved searches with `channel_kind=telegram`. Provider-side saved-search execution semantics beyond the shared projection query remain missing. |
| Dialog search | PARTIAL | `GET /api/v1/telegram/chats/search` now backs header-driven dialog search results and can reopen projected chats, but it remains projection/local-only and does not reach provider-side TDLib search. |
| Media search | PARTIAL | `GET /api/v1/telegram/search/media` now backs the files-tab media gallery and workspace media hits with projection filters plus TDLib provider-refresh attempts for text/account-scoped queries; workspace media cards can reopen the owning message context in the current dialog and show projected preview/download readiness. Richer provider media filters and remote preview parity remain missing. |
| Provider search | PARTIAL | `SearchMessages` and `SearchChatMessages` runtime command variants call TDLib `searchMessages`/`searchChatMessages`; `search_provider_messages` method on `TelegramRuntimeManager` ingests results before the projection query; `GET /api/v1/telegram/search/messages` and `POST /api/v1/telegram/search/provider` both trigger provider search when `account_id` is present, and the workspace search UI labels provider/local/fallback search mode. Saved-search persistence now reuses the shared Communication saved-search API with `channel_kind=telegram`, but provider-side saved-search scheduling/execution remains missing. |
| Topic search | PARTIAL | `telegram_topics` projection, list endpoint and dedicated title search route now exist (`GET /api/v1/telegram/topics/search`), and Telegram thread side section includes a topic search bar with projected state/provider labels; provider-side topic discovery is still missing. |
| Member search | PARTIAL | `GET /api/v1/telegram/chats/{chat_id}/members?query=&role=&limit=&cursor=` now filters provider roster rows when available and falls back to labeled message-sender search only without provider roster. The inspector members panel exposes local search over those rows, but provider-side remote member search and full cursor pagination are still missing. |

## Realtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| Generic transport | IMPLEMENTED | WebSocket/SSE/long-poll transports exist at platform level, and the Telegram page now relies on the shared frontend realtime bootstrap instead of opening a duplicate page-local socket. |
| New message | PARTIAL | `telegram.message.created` is emitted on fixture ingest, manual send, and now TDLib unsolicited `updateNewMessage` runtime updates. Events are appended to the canonical event log when DB/event store is enabled, include sanitized projected `message` and `chat` snapshots plus deterministic `telegram_chat_id`, and the frontend can upsert matching message/search caches while patching chat list/detail state for current dialog projections without a full reload. Edit/update parity is still partial. |
| Edit message | PARTIAL | `telegram.message.updated` is emitted on lifecycle edit record creation and now also from TDLib unsolicited `updateMessageContent` / `updateMessageEdited` after refreshing projected message text/metadata. Provider content updates now record append-only observed edit versions and can reconcile matching edit commands from provider-observed body text; frontend patching still accepts the legacy `telegram.message.edited` alias for older rows/clients. Raw provider message rows remain append-only, so provider-observed edit source evidence currently lives in version records plus event-log payloads rather than a second raw message row. |
| Delete message | PARTIAL | `telegram.message.deleted` is emitted on local tombstone creation and now also from TDLib unsolicited `updateDeleteMessages` after recording an idempotent `deleted_by_provider` tombstone and reconciling matching delete commands. Payloads include the latest projected message/chat snapshots for cache patching, but tombstone UX is still partial. |
| Reaction update | PARTIAL | `telegram.reaction.changed` is emitted for local add/remove actions and now also from TDLib unsolicited `updateMessageInteractionInfo`; message query caches can be patched from the projected message snapshot/reaction summary. TDLib history sync and runtime interaction updates now reconcile self reaction command status from provider message snapshots, but non-self removal parity is still missing. |
| Sync update | PARTIAL | `telegram.sync.started/progress/completed/failed` are emitted for chat/history sync requests and appended to the event log when available, account-scoped runtime query caches now persist last sync scope/status/count, selected-thread header chips can surface sync state when the cached target matches the current dialog, inspector context shows the sync target, and the Telegram workbench now renders shared WS/SSE/long-poll fallback/offline state from `useRealtimeStatusStore`; richer replay controls and per-event recovery diagnostics are still missing. |
| Typing indicator | PARTIAL | `telegram.typing.changed` contract and TDLib `updateUserChatAction` parser exist; TDLib actors created from runtime, sync, search, send, media download and topic routes now drain unsolicited typing updates into a sanitized actor event bridge that appends/broadcasts `telegram.typing.changed`, frontend realtime invalidation handles typing events, and matching chat caches patch visible typing UI state with a bounded expiry timestamp. Richer explicit provider clear/absence semantics remain missing. |
| Topic update | PARTIAL | `telegram.topic.updated` is emitted from TDLib `updateForumTopicInfo` runtime updates after resolving the projected chat and upserting the existing `telegram_topics` row; frontend cache patching updates `topics` and `topic-search` query results before replay invalidation. Topic write commands and richer unread/live topic state reconciliation remain missing. |
| Media download progress | PARTIAL | Media download requests now emit `telegram.media.download.started`, `telegram.media.download.progress`, `telegram.media.download.failed` and completed `telegram.media.downloaded` events; frontend invalidates Telegram message/media-search caches for the `telegram.media.*` family. Richer retry/dead-letter UX is still missing. |
| Chat flag updates | PARTIAL | Local dialog pin/archive/mute routes now emit `telegram.chat.pinned`, `telegram.chat.archived` and `telegram.chat.muted` with sanitized projected chat snapshots, local read/unread routes emit `telegram.chat.updated` after unread counter recomputation, TDLib provider-observed unread/mention counter updates emit the same `telegram.chat.updated` contract, and TDLib `updateChatIsMarkedAsUnread` / `updateChatNotificationSettings` / `updateChatPosition` now project unread-flag, mute state, archive state and dialog pin state back into chat metadata with realtime event emission. Provider folder-label parity is still missing. |
| Command status | PARTIAL | `telegram.command.status_changed` is emitted for manual send, lifecycle edit/delete/restore/message-pin, reaction add/remove, media upload queueing and current dialog actions (`pin/unpin/archive/unarchive/mute/unmute/read/unread`); active TDLib execution now covers archive/mute/read-unread dialog commands and media upload/send from local blobs. Provider-observed send/reply/forward/media results are ingested into the Communication projection before commands are marked completed and `telegram.command.reconciled` is emitted; self `join` commands now also reconcile from TDLib roster presence, self `join`/`leave` can reconcile from explicit TDLib participant service-message evidence during history sync, self `react`/`unreact` can reconcile from TDLib message `interaction_info.reactions` chosen state during history sync and unsolicited runtime interaction updates, edit commands can now reconcile from TDLib `updateMessageContent`, delete commands can now reconcile from TDLib `updateDeleteMessages`, and dialog `pin`/`unpin`, `mark_read`/`mark_unread`, exact-shape `mute`/`unmute`, plus `archive`/`unarchive` can reconcile from TDLib chat-state updates. Remaining ACK-only writes stay `executing/awaiting_provider` until later provider observation. Runtime and command-list caches patch command events, but admin, Bot API, folder labels/mutations, silent/admin lifecycle and full reconciliation coverage are still missing. |

## AI

| Capability | Status | Evidence / Gap |
|---|---|---|
| Summary | PARTIAL | Telegram projections can feed shared message intelligence; Telegram-specific summary API/UI missing. |
| Translation | MISSING | No Telegram route/UI found. |
| Bilingual reply | MISSING | No Telegram route/UI found. |
| Task extraction | PARTIAL | Telegram messages can refresh obligation-derived task candidates; review engine is out of Telegram scope. |
| Note extraction | MISSING | No Telegram-specific note extraction path found. |
| Event extraction | MISSING | No Telegram-specific event candidate UI/API found. |
| Persona extraction | PARTIAL | Sender metadata is preserved as identity trace; Persona review lifecycle is outside Telegram scope. |
| Organization extraction | MISSING | No Telegram-specific organization candidate review found. |
| Voice transcript summary | MISSING | Fixture transcripts exist for calls; Telegram voice transcript/summary flow missing. |
| AI state lifecycle | MISSING | No Telegram-specific AI state API/events found. |

## UI

| Capability | Status | Evidence / Gap |
|---|---|---|
| Dialog list | PARTIAL | Virtualized chat list exists, the action rail now consumes account-scoped `GET /api/v1/telegram/folders` for projection-backed folder filters with local fallback, selected-chat pin/archive/mute toggles are exposed in the workbench, and projected `mention_count` is now visible in chat-row badges plus selected-thread unread/mention summary chips; provider folders/participant/admin state remain missing. |
| Message timeline | PARTIAL | Selected chat timeline exists; rich thread/reply/deletion history missing. |
| Thread view | PARTIAL | Selected chat thread exists, can show projected reply/forward references per message, can reopen pinned/search results into a focused message row, can reopen single-hop reply/forward references into the thread, and now exposes capability-gated local pin/unpin plus unread/mention summary visibility from projected chat metadata; the thread header also shows selected-chat realtime sync/command chips, and the reference panel shows richer edit/tombstone/command evidence summaries, but full reply-thread/topic navigation is still missing. |
| Composer | PARTIAL | Text send exists, and the attachment affordance now consumes the selected-account capability contract, opens a local file picker when `messages.send_media` is available, and sends files through the shared import plus Telegram `send_media` command path. Voice recording/send and richer upload progress state remain missing. |
| Inspector | PARTIAL | `TelegramRail.vue` now renders read-only context/about/member sections from projected chat detail and member queries, including a dedicated provider roster panel with sync control, source labels, role/admin/owner badges and permission summaries. It also renders TDLib chat permission summaries, runtime status, explicit last sync/command targets, a first-class calls panel with local projected-call search plus transcript evidence, and account-scoped durable command rows with current-chat filtering/search; linked entities, admin mutations and broader provider diagnostics are still missing. |
| Media viewer | PARTIAL | File cards and gallery items can open a dedicated read-only viewer for downloaded local files, files-tab rows can now keep projected local blob paths even when sourced from media search instead of the loaded thread window, downloaded projected attachments can use shared safe text/image previews by `attachment_id`, remote-only files can trigger the existing TDLib download flow directly from the viewer, and voice/audio items now also have a dedicated thread voice tab with inline playback, but remote preview parity and richer navigation are still missing. |
| Search UI | PARTIAL | Header search now opens query-backed Telegram dialog/message/media workspace results, files tab shows a gallery from Telegram media search, media result sections display provider-refresh/projection fallback source labels, and the thread surface exposes shared saved-search create/select/delete UI scoped to the selected Telegram account and `channel_kind=telegram`. Provider/global saved-search scheduling remains missing. |
| Reactions UI | PARTIAL | Thread UI renders reaction summary chips plus capability-gated add/remove controls, and per-message evidence panels can read reaction detail/summary from the dedicated reactions endpoint plus custom reaction aggregate metadata from message source evidence, but provider reconciliation is still missing. |
| Edit/delete UI | PARTIAL | Thread UI now exposes capability-gated edit/delete/restore controls, per-message evidence panels can read projected versions, tombstones and recent command rows with diff/visibility/command-state summaries, and inspector context now exposes account-scoped durable command rows through a dedicated audit panel with retry/dead-letter summaries; active TDLib actors execute queued edit/delete commands, but provider-observed reconciliation UX is still missing. |
| Calls UI | PARTIAL | Call metadata/transcript routes exist, and the inspector context panel now renders a first-class selected-account calls panel from `GET /api/v1/calls` with local projected-call search plus read-only transcript evidence from `GET /api/v1/calls/{call_id}/transcript`, but live UX is still missing. |
| Runtime/account setup UX | PARTIAL | Runtime and QR routes exist, and the inspector `About` tab now contains a local account manager for setup/logout/remove, a real QR login panel (`start/status/password/cancel`) that can apply suggested QR-authorized account metadata into the setup form, and a read-only selected-account capability matrix, but complete credential/session/proxy capability UX is still incomplete. |

Automation / composer note: Telegram workbench now also surfaces `POST /api/v1/policies/telegram-send/dry-run` inside the composer send dropdown with policy selection, variable inputs and rendered preview/hash output, but it still lacks live automation send and broader policy/template management UX.

## Scope Boundary

| Capability | Status | Evidence / Gap |
|---|---|---|
| Obligation Engine | PARTIAL | Telegram can create candidates through shared projection refresh; lifecycle implementation is outside Telegram scope. |
| Decision Engine | PARTIAL | Telegram can create suggested decisions from evidence; lifecycle is outside Telegram scope. |
| Memory Engine | MISSING | No Telegram-specific Memory Engine implementation intended in this phase. |
| Knowledge Engine | MISSING | No Telegram-specific Knowledge Engine implementation intended in this phase. |
| Persona Intelligence | PARTIAL | Identity traces are preserved; canonical Persona lifecycle is outside Telegram scope. |
| Organization Intelligence | MISSING | No Telegram-specific Organization lifecycle intended in this phase. |
| Cross-Domain Integrations | MISSING | Only integration points are documented; implementation is out of scope. |

## Priority Recommendations

### P0 — Foundation before feature sprawl

1. Provider-side execution for lifecycle and reaction commands.
2. Finish frontend cache patching and replay-state UX for remaining provider dialog events and richer Telegram media progress/retry states.
3. Reply/forward/topic projection beyond the current schema foundation.
4. Command executor/outbox behavior for participants/admin, Bot API and provider-observed reconciliation beyond the current durable runtime foundation.

### P1 — Telegram product parity

1. Dialog management: unread, pinned, archive, mute, folders.
2. Reply chains and topics.
3. Reactions.
4. Provider search.
5. Media gallery and viewer.

### P2 — Hermes intelligence surface

1. Telegram summary.
2. Translation.
3. Bilingual reply.
4. AI extraction review.
5. Voice transcript summary and live call UX.

### P3 — Native/infra blocked

1. Bot API runtime.
2. Proxy/session bundles.
3. Voice/video recording.
4. Live calls.
5. Scanner-backed clean verdicts.
