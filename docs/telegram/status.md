# Telegram Implementation Status

Статус на 2026-06-15.

Проценты ниже описывают coverage Telegram Channel only. Они не являются оценкой
готовности всего Communications, Memory, Knowledge, Obligation, Decision,
Persona или Timeline продукта.

## Summary Table

| § | Раздел | Статус | % |
|---|---|---|---|
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

## Legend

- ✓ - реализовано для текущего scope.
- ◐ - есть foundation или частичная реализация, но production scope не закрыт.
- ✗ - durable implementation не найден в текущем репозитории.

## Details

**§2 Provider/account model (70%)**: `telegram_user` and `telegram_bot`
provider kinds exist. Multiple accounts are supported. Account lifecycle can
list, logout and remove while preserving evidence. Session import/export,
proxy profiles and live bot runtime are missing.

**§3 Secret references (90%)**: Telegram API hash, bot token and session key
purposes are account-scoped and resolved by secret reference. Host-vault writes
exist for new live credentials. Remaining gap is broader session/proxy secret
bundle UX.

**§4 Capability contract (35%)**: `/api/v1/telegram/capabilities` exists but is
coarse. It does not enumerate the detailed capability matrix required by
ADR-0091 for edits, deletes, reactions, topics, exports, proxies, offline and
destructive actions.

**§6 TDLib QR runtime (60%)**: QR login start/status/password/cancel exists.
`tdlib_qr_authorized` runtime can start actor, sync chats/history, send manual
text and download media when native TDLib and app credentials are configured.
Coverage is not equivalent to production parity.

**§8-10 Dialogs/private/groups/channels (35-60%)**: Chat list projection exists
with `private`, `group`, `channel`, `bot`. TDLib supergroups are collapsed into
`group`; provider permissions, admin actions, participants and folders are not
first-class.

**§12 Message ingestion/projection (85%)**: Fixture and TDLib messages become
append-only raw records and `communication_messages`. TDLib media messages may
have empty text while preserving raw payload. Dedicated reply/reaction/edit
fields are not projected.

**§13 Message lifecycle commands (20%)**: Manual send exists. Edit, delete,
restore, mark read/unread and provider-side state changes are missing.

**§16 History/versioning (25%)**: Selected history sync and older pagination
exist. Observed edit versions, diffs, tombstones and deletion history are not
implemented.

**§17 Media (45%)**: TDLib raw media metadata can be retained, and completed
downloads persist local blobs plus attachment rows. Upload, gallery, provider
search, rich preview and voice/video recording are missing.

**§18 Attachments (25%)**: Downloaded media uses the existing scanner/blob
boundary. Telegram-specific preview/search/dedup UX is missing, though shared
Communication attachment APIs may expose some rows once downloaded.

**§20 Calls/transcripts (35%)**: Call metadata and fixture transcripts exist.
Live call control, audio capture, device selection and real STT are blocked.

**§21 Search (30%)**: Local UI chat/message text filtering exists. Shared
communication search can include Telegram-projected messages by channel kind.
Provider-side Telegram search and dedicated media/dialog search APIs are
missing.

**§22 Realtime (15%)**: Generic WebSocket/SSE/long-poll transports exist.
Telegram-specific event types and frontend cache patches are missing.

**§23 AI (20%)**: Projected Telegram messages can feed shared candidates and
engines. Telegram-specific summary, translation, bilingual reply and extraction
UI/API surfaces are missing.

**§24 Frontend workbench (55%)**: Vue Telegram page has chat list, selected
message timeline, local filters/tabs, composer, sync controls and media
download action. Account setup drawer, inspector content, media viewer,
provider commands and rich thread/reply UX are incomplete.

**§26 Offline/outbox/export/proxy/session bundles (0%)**: No durable Telegram
offline command queue, retry state, export flows, proxy profile persistence or
session bundle import/export were found.
