# Telegram Gap Analysis

Status date: 2026-06-16.

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
| Runtime health | PARTIAL | `GET /api/v1/telegram/runtime/status?account_id=` now returns TDLib path/probe diagnostics, split API credential readiness and derived runtime blockers for the selected account, but explicit stop/restart controls and deeper native dependency remediation remain incomplete. |
| Session import/export | MISSING | No bundle schema/API/UI found. |
| Proxy / MTProxy / SOCKS5 | MISSING | No proxy profile schema/API/runtime command found. |

## Dialogs / Chat Management

| Capability | Status | Evidence / Gap |
|---|---|---|
| Dialogs | PARTIAL | `telegram_chats` projection, chat list/detail endpoints, local projected pin/archive/mute/read actions, and realtime chat snapshot patching for current local dialog commands exist; provider folders and provider-synced parity are still missing. |
| Private chats | PARTIAL | `private` chat kind exists and selected history sync works; no full provider parity. |
| Groups | PARTIAL | `group` chat kind exists; participant/admin/group lifecycle actions missing. |
| Supergroups | PARTIAL | TDLib supergroups are collapsed into `group`; no first-class supergroup metadata. |
| Channels | PARTIAL | `channel` chat kind exists; subscribe/unsubscribe/post admin actions missing. |
| Saved Messages | MISSING | No special saved-messages modeling found. |
| Pinned chats | PARTIAL | Local metadata-backed pin/unpin routes exist for projected chats; provider-synced parity is still missing. |
| Archived chats | PARTIAL | Local metadata-backed archive/unarchive routes exist; provider-backed archive state is still missing. |
| Chat folders | PARTIAL | `GET /api/v1/telegram/folders?account_id=` now returns projection-backed local/Telegram folder filters derived from `telegram_chats.metadata.folder_name`, but provider folder sync and folder mutation routes are still missing. |
| Mute/unmute | PARTIAL | Local metadata-backed mute/unmute routes exist; provider-backed mute sync is still missing. |
| Unread counters | PARTIAL | Chat metadata now stores projection-backed `unread_count`, `POST /api/v1/telegram/chats/{chat_id}/read` and `/unread` mutate local read state, and unread badges/filters use that value; provider-synced unread state and unread-mention derivation are still missing. |
| Typing indicators | MISSING | No realtime typing event contract found. |
| Participants / Members | PARTIAL | `GET /api/v1/telegram/chats/{chat_id}/members` now derives top senders from projected message history for the selected chat, and the inspector `Members` tab supports local search over that projected roster, but there is still no provider-synced participant roster, role state or join/leave lifecycle. |
| Roles / Permissions | MISSING | No admin/role/permission projection found. |
| Join / Leave | MISSING | No provider command or lifecycle projection found. |

## Messaging

| Capability | Status | Evidence / Gap |
|---|---|---|
| Topics | PARTIAL | Migration 0086 adds `telegram_topics` table; `upsert_topic`/`list_topics`/`get_topic` queries exist; three API routes (`GET /chats/{id}/topics`, `GET /topics/{id}`, `GET /topics/{id}/messages`) are wired; frontend `TelegramTopic` type, `useTelegramTopicsQuery`/`useTelegramTopicMessagesQuery` hooks, and Topics tab in `TelegramThreadSideSections` are in place; `GetForumTopics` runtime command calls TDLib `getForumTopics` on demand and upserts results via `sync_forum_topics` on the manager — topics projection is now populated for live TDLib accounts on first request. Topic write commands (create/close/reopen), topic-scoped unread/pinned sync, and push-event `updateForumTopicInfo` listener still missing. |
| Replies | PARTIAL | Reply command flow is now complete end-to-end: `POST /api/v1/telegram/messages/{message_id}/reply` records the command, sends `sendMessage` with `reply_to` via TDLib through `ReplyMessage` runtime variant and `send_reply_message` on the manager, and the frontend wires a composer reply-preview banner, Reply action in message action bars, and `useTelegramSendActions` composable routing send vs reply in TelegramPage; multi-hop reply-thread model and deeper graph navigation are still missing. |
| Reply chains | PARTIAL | `GET /api/v1/telegram/messages/{message_id}/reply-chain` now returns projected source/target message summaries and the reference panel can reopen/filter those summaries inside the thread; provider-side reply creation is now supported via the reply command flow above, but deeper graph navigation beyond single-hop is still missing. |
| Forwards | PARTIAL | Forward-chain projection API exists and thread UI can read origin attribution, but there is no forward command flow yet. |
| Forward chains | PARTIAL | `GET /api/v1/telegram/messages/{message_id}/forward-chain` now returns local source-message summaries plus origin metadata, and the reference panel can filter these rows while showing richer origin/source text summaries, but provider-side forward execution and deeper chain UX are still missing. |
| Mentions | PARTIAL | Ingest now derives per-message `mention_count` from TDLib formatted-text entities or fallback `@handle` parsing, and chat metadata recomputes unread mention totals; provider-identity targeting and richer mention semantics are still missing. |
| Pinned messages | PARTIAL | `GET /api/v1/telegram/chats/{chat_id}/pinned-messages` now backs a chat-scoped pinned-message tab from projection metadata, pinned/search reopen can focus a projected target row in the selected thread even if it is outside the currently loaded window, and `POST /api/v1/telegram/messages/{message_id}/pin` now records local pin/unpin projection state, durable command rows, redacted audit and thread-level pin UI, but provider-side message pin/unpin sync is still missing. |
| Message links | MISSING | No stable Telegram message link/permalink projection found. |
| Polls | MISSING | No first-class poll projection/UI found. |
| Locations | MISSING | No first-class location projection/UI found. |
| Contacts | MISSING | No first-class contact-card projection/UI found. |

## Message Lifecycle

| Capability | Status | Evidence / Gap |
|---|---|---|
| Send text | PARTIAL | Manual text send works through fixture/TDLib QR runtime and redacted audit; queued outbox missing. |
| Send media | MISSING | No durable media send/upload path found. |
| Edit | PARTIAL | `POST /api/v1/telegram/messages/{message_id}/edit` records append-only edit versions and a provider-write command row, but does not yet execute provider-side edit. |
| Delete | PARTIAL | `POST /api/v1/telegram/messages/{message_id}/delete` records tombstones and a provider-write command row, but does not yet execute provider-side delete. |
| Restore visibility | PARTIAL | `POST /api/v1/telegram/messages/{message_id}/restore-visibility` now records local visibility restoration, a durable command row, redacted audit and `telegram.command.status_changed`, but provider parity is still missing. |
| Mark read/unread | PARTIAL | `POST /api/v1/telegram/chats/{chat_id}/read` and `/unread` now record local dialog command rows and recompute projected unread counters, but provider-side read-state execution is still missing. |
| Message history | PARTIAL | Selected/older history sync exists; observed edit versions/diffs/deletion history missing. |
| Edit versions | PARTIAL | `telegram_message_versions` plus list endpoint exist, and the per-message reference panel now renders local edit-history summaries with relative text-length deltas between captured versions, but provider-observed diffs are still missing. |
| Tombstones | PARTIAL | `telegram_message_tombstones` plus list endpoint exist, and the per-message reference panel now renders provider-delete vs local-visibility state summaries, but provider reconciliation/history UX is still missing. |
| Provider command retry | PARTIAL | `retry_command` increments `retry_count` up to `max_retries`; the executor polls `status IN ('queued', 'retrying') AND retry_count < max_retries`; final failure marks `failed` with `last_error`. Exponential back-off and dead-letter UI still missing. |

## Reactions

| Capability | Status | Evidence / Gap |
|---|---|---|
| Add reaction | PARTIAL | Add-reaction endpoint, degraded local-write capability state, durable command row, redacted audit, thread picker UX and read-only evidence consumption of `GET /api/v1/telegram/messages/{message_id}/reactions` exist, but provider-side execution is still missing. |
| Remove reaction | PARTIAL | Remove-reaction endpoint, degraded local-write capability state, durable command row, redacted audit and thread remove UX exist, but provider-side execution is still missing. |
| Reaction sync | PARTIAL | Local reaction rows and `telegram.reaction.changed` events exist; provider-side sync/parsing are still missing. |
| Reaction summary | PARTIAL | Message list responses include reaction summary and the Vue thread can render summary chips, but no background/provider reconciliation exists. |

## Media

| Capability | Status | Evidence / Gap |
|---|---|---|
| Photos | PARTIAL | TDLib `messagePhoto` caption text can be parsed and raw payload retained; gallery exists and downloaded images can open in the viewer, but richer grouping/provider parity is missing. |
| Videos | PARTIAL | TDLib `messageVideo` caption text can be parsed and raw payload retained; downloaded videos can open in the viewer, but provider parity and richer gallery UX are missing. |
| Documents | PARTIAL | TDLib `messageDocument` caption text can be parsed; download can create attachment row when file metadata is supplied. |
| Voice messages | PARTIAL | `messageVoiceNote` text handling exists, and the thread now exposes a dedicated voice tab with inline playback for downloaded local voice/audio attachments plus download fallback for remote-only items, but record/send/transcript flows are still missing. |
| Video notes | MISSING | No dedicated parser/UI/runtime path found. |
| Audio | PARTIAL | `messageAudio` caption text can be parsed; downloaded audio can open in the viewer, and the thread voice tab can play downloaded local audio attachments inline, but richer playback/gallery UX remains missing. |
| Stickers | MISSING | No first-class sticker parser/projection/UI found. |
| GIF / animation | MISSING | No first-class GIF/animation parser/projection/UI found. |
| Media albums | MISSING | No media-album grouping found. |
| Upload media | MISSING | No provider send/upload media path found. |
| Media gallery | PARTIAL | Files tab now renders a Telegram-scoped gallery from `GET /api/v1/telegram/search/media`, merges query-backed media rows with loaded attachment hints so older downloaded media can still preview/download outside the current message window, and workspace media hits can reopen the owning message context in the current dialog, but richer grouping/provider parity is still missing. |
| Media search | PARTIAL | `GET /api/v1/telegram/search/media` now supports projection-backed scope/kind/free-text filtering plus attachment download metadata (`tdlib_file_id`, `provider_attachment_id`, `local_path`) for files-tab/media-viewer parity, but it still does not reach provider-side TDLib search. |

## Attachments

| Capability | Status | Evidence / Gap |
|---|---|---|
| Preview | PARTIAL | Telegram workbench now has a dedicated media viewer for downloaded local files, and files-tab gallery rows can preserve projected `local_path` metadata even when the source message is outside the loaded thread window, but remote/not-yet-downloaded attachments still have no real preview. |
| Download | PARTIAL | TDLib media download endpoint exists and fixture runtime fails closed; completed downloads now persist blob/attachment rows, patch projected Telegram message attachment metadata and emit `telegram.media.downloaded`, and files-tab gallery rows can now reuse projected TDLib/provider attachment metadata to retry downloads outside the loaded thread window, but the path is still unavailable without a TDLib actor. |
| Search | PARTIAL | Telegram workbench now has query-backed dialog/message/media search plus a chat-scoped media gallery, but still lacks provider-side search and richer preview/search UX beyond current projection filters. |
| Deduplication | PARTIAL | Blob SHA-256 dedup exists in shared storage; no Telegram-specific duplicate UX. |
| Scanner-backed clean verdict | MISSING | No real scanner backend marks Telegram attachments `clean`. |
| Persisted preview artifacts | MISSING | No durable Telegram preview artifact pipeline found. |

## Search

| Capability | Status | Evidence / Gap |
|---|---|---|
| Message search | PARTIAL | Workspace UI now uses `GET /api/v1/telegram/search/messages` for query-backed Telegram message results, but provider-side TDLib search and saved searches are still missing. |
| Dialog search | PARTIAL | `GET /api/v1/telegram/chats/search` now backs header-driven dialog search results and can reopen projected chats, but it remains projection/local-only and does not reach provider-side TDLib search. |
| Media search | PARTIAL | `GET /api/v1/telegram/search/media` now backs the files-tab media gallery and workspace media hits with projection/local free-text filters plus download metadata for files-tab parity, and workspace media cards can reopen the owning message context in the current dialog, but it remains projection-local and does not reach provider-side TDLib search. |
| Provider search | PARTIAL | `SearchMessages` and `SearchChatMessages` runtime command variants call TDLib `searchMessages`/`searchChatMessages`; `search_provider_messages` method on `TelegramRuntimeManager` ingests results before the projection query; `GET /api/v1/telegram/search/messages` now triggers live TDLib search when `account_id` is present. No dedicated provider-only search UI or saved searches. |
| Topic search | MISSING | `telegram_topics` projection and list endpoint now exist (Slice 3), but no dedicated topic search route or UI has been built yet. |
| Member search | PARTIAL | The inspector `Members` tab now supports local search over the projected member roster returned by `GET /api/v1/telegram/chats/{chat_id}/members`, but there is still no dedicated provider/member search route or provider-synced roster. |

## Realtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| Generic transport | IMPLEMENTED | WebSocket/SSE/long-poll transports exist at platform level, and the Telegram page now relies on the shared frontend realtime bootstrap instead of opening a duplicate page-local socket. |
| New message | PARTIAL | `telegram.message.created` is emitted on fixture ingest and manual send, appended to the canonical event log when DB/event store is enabled, now includes sanitized projected `message` and `chat` snapshots plus deterministic `telegram_chat_id`, and the frontend can upsert matching message/search caches while patching chat list/detail state for current dialog projections without a full reload; broader runtime/provider patching is still partial. |
| Edit message | PARTIAL | `telegram.message.updated` is emitted on lifecycle edit record creation and appended to the event log when available; payloads now include sanitized projected message snapshots, projected chat snapshots when a dialog projection exists, pin metadata and deterministic `telegram_chat_id`, and frontend patching accepts the legacy `telegram.message.edited` alias for older rows/clients. |
| Delete message | PARTIAL | `telegram.message.deleted` is emitted on tombstone creation and appended to the event log when available; payloads now include the latest projected message/chat snapshots for cache patching, but tombstone UX is still partial. |
| Reaction update | PARTIAL | `telegram.reaction.changed` is emitted for local add/remove actions and message query caches can be patched, but provider sync is still missing. |
| Sync update | PARTIAL | `telegram.sync.started/progress/completed/failed` are emitted for chat/history sync requests and appended to the event log when available, and account-scoped runtime query caches now persist last sync scope/status/count for inspector/runtime UX, but richer dialog-level sync state is still missing. |
| Typing indicator | MISSING | No typing realtime contract found. |
| Media download progress | PARTIAL | `telegram.media.downloaded` now covers completed-download projection/cache refresh, but there is still no dedicated progress/in-flight Telegram media event contract. |
| Command status | PARTIAL | `telegram.command.status_changed` is emitted for manual send, lifecycle edit/delete/restore/message-pin, reaction add/remove and current local dialog actions (`pin/unpin/archive/unarchive/mute/unmute/read/unread`), runtime query caches retain the last account-scoped command outcome, and local dialog command payloads now include projected chat snapshots that patch chat list/detail caches; broader provider-command coverage and richer command-status surfaces are still missing. |

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
| Thread view | PARTIAL | Selected chat thread exists, can show projected reply/forward references per message, can reopen pinned/search results into a focused message row, can reopen single-hop reply/forward references into the thread, and now exposes capability-gated local pin/unpin plus unread/mention summary visibility from projected chat metadata; the reference panel also shows richer edit/tombstone/command evidence summaries, but full reply-thread/topic navigation is still missing. |
| Composer | PARTIAL | Text send exists; attachment/voice commands disabled or missing. |
| Inspector | PARTIAL | `TelegramRail.vue` now renders read-only context/about/member sections from projected chat detail and member queries, including local search over the projected member roster, and context can surface runtime status, a first-class calls panel with local projected-call search plus transcript evidence, and account-scoped durable command rows with current-chat filtering/search; the account manager in `About` also exposes the selected-account capability matrix, but permissions, topics, linked entities and broader provider diagnostics are still missing. |
| Media viewer | PARTIAL | File cards and gallery items can open a dedicated read-only viewer for downloaded local files, files-tab rows can now keep projected local blob paths even when sourced from media search instead of the loaded thread window, and voice/audio items now also have a dedicated thread voice tab with inline playback, but remote preview parity and richer navigation are still missing. |
| Search UI | PARTIAL | Header search now opens query-backed Telegram dialog/message/media workspace results, and files tab shows a gallery from Telegram media search; provider/global saved searches remain missing. |
| Reactions UI | PARTIAL | Thread UI renders reaction summary chips plus capability-gated add/remove controls, and per-message evidence panels can read reaction detail/summary from the dedicated reactions endpoint, but provider reconciliation is still missing. |
| Edit/delete UI | PARTIAL | Thread UI now exposes capability-gated edit/delete/restore controls, per-message evidence panels can read projected versions, tombstones and recent command rows with diff/visibility/command-state summaries, and inspector context now exposes account-scoped durable command rows through a dedicated audit panel, but provider execution parity is still missing. |
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
4. Command executor/outbox behavior beyond durable command rows.

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
