# Telegram Gap Analysis

Status date: 2026-06-15.

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
| QR authorization | PARTIAL | QR start/status/password/cancel exists; live readiness depends on TDLib runtime and app credentials. |
| Bot account setup | PARTIAL | Bot token can be stored via host-vault reference; live Bot API runtime is missing. |
| Logout | IMPLEMENTED | Account logout marks lifecycle state and stops actor with audit. |
| Remove account | IMPLEMENTED | Removal preserves raw records/messages and records audit. |
| Runtime health | PARTIAL | Runtime status exists; detailed TDLib dependency diagnostics are incomplete. |
| Session import/export | MISSING | No bundle schema/API/UI found. |
| Proxy / MTProxy / SOCKS5 | MISSING | No proxy profile schema/API/runtime command found. |

## Dialogs / Chat Management

| Capability | Status | Evidence / Gap |
|---|---|---|
| Dialogs | PARTIAL | `telegram_chats` projection and chat list endpoint exist; provider folders/archived/pinned commands are missing. |
| Private chats | PARTIAL | `private` chat kind exists and selected history sync works; no full provider parity. |
| Groups | PARTIAL | `group` chat kind exists; participant/admin/group lifecycle actions missing. |
| Supergroups | PARTIAL | TDLib supergroups are collapsed into `group`; no first-class supergroup metadata. |
| Channels | PARTIAL | `channel` chat kind exists; subscribe/unsubscribe/post admin actions missing. |
| Saved Messages | MISSING | No special saved-messages modeling found. |
| Pinned chats | MISSING | No provider pin/unpin chat command/projection found. |
| Archived chats | MISSING | No archive/unarchive provider state found. |
| Chat folders | MISSING | No Telegram folders/chat lists projection found. |
| Mute/unmute | MISSING | No mute state command/projection found. |
| Unread counters | MISSING | No durable unread counter sync/projection found. |
| Typing indicators | MISSING | No realtime typing event contract found. |
| Participants / Members | MISSING | No first-class participant/member projection found. |
| Roles / Permissions | MISSING | No admin/role/permission projection found. |
| Join / Leave | MISSING | No provider command or lifecycle projection found. |

## Messaging

| Capability | Status | Evidence / Gap |
|---|---|---|
| Topics | MISSING | No topic table/API/projection found. |
| Replies | MISSING | No reply target command/projection fields found. |
| Reply chains | MISSING | No reply graph/thread model found. |
| Forwards | MISSING | No forward command or attribution model found. |
| Forward chains | MISSING | No forward-chain model found. |
| Mentions | PARTIAL | UI can filter `mention_count` metadata if present; parser/projection does not derive mentions. |
| Pinned messages | PARTIAL | UI can derive pinned messages from metadata; provider pin/unpin and sync are missing. |
| Message links | MISSING | No stable Telegram message link/permalink projection found. |
| Polls | MISSING | No first-class poll projection/UI found. |
| Locations | MISSING | No first-class location projection/UI found. |
| Contacts | MISSING | No first-class contact-card projection/UI found. |

## Message Lifecycle

| Capability | Status | Evidence / Gap |
|---|---|---|
| Send text | PARTIAL | Manual text send works through fixture/TDLib QR runtime and redacted audit; queued outbox missing. |
| Send media | MISSING | No durable media send/upload path found. |
| Edit | MISSING | No edit route/command/schema found. |
| Delete | MISSING | No Telegram delete route/command/tombstone schema found. |
| Restore visibility | MISSING | No local tombstone/restore model found. |
| Mark read/unread | MISSING | No provider mark read/unread command/projection found. |
| Message history | PARTIAL | Selected/older history sync exists; observed edit versions/diffs/deletion history missing. |
| Edit versions | MISSING | No append-only edit-version model found. |
| Tombstones | MISSING | No durable delete/tombstone evidence model found. |
| Provider command retry | MISSING | No command outbox/retry model found. |

## Reactions

| Capability | Status | Evidence / Gap |
|---|---|---|
| Add reaction | MISSING | No provider command/API found. |
| Remove reaction | MISSING | No provider command/API found. |
| Reaction sync | MISSING | No parser/projection/realtime contract found. |
| Reaction summary | MISSING | No aggregate reaction UI/state found. |

## Media

| Capability | Status | Evidence / Gap |
|---|---|---|
| Photos | PARTIAL | TDLib `messagePhoto` caption text can be parsed and raw payload retained; first-class photo metadata/gallery missing. |
| Videos | PARTIAL | TDLib `messageVideo` caption text can be parsed and raw payload retained; viewer/gallery missing. |
| Documents | PARTIAL | TDLib `messageDocument` caption text can be parsed; download can create attachment row when file metadata is supplied. |
| Voice messages | PARTIAL | `messageVoiceNote` text handling exists; voice playback/record/send UI missing. |
| Video notes | MISSING | No dedicated parser/UI/runtime path found. |
| Audio | PARTIAL | `messageAudio` caption text can be parsed; audio playback/gallery missing. |
| Stickers | MISSING | No first-class sticker parser/projection/UI found. |
| GIF / animation | MISSING | No first-class GIF/animation parser/projection/UI found. |
| Media albums | MISSING | No media-album grouping found. |
| Upload media | MISSING | No provider send/upload media path found. |
| Media gallery | MISSING | No first-class Telegram media gallery found. |
| Media search | MISSING | No Telegram media search endpoint found. |

## Attachments

| Capability | Status | Evidence / Gap |
|---|---|---|
| Preview | PARTIAL | Shared Communication attachment preview may work for downloaded blobs; Telegram workbench has no dedicated preview surface. |
| Download | PARTIAL | TDLib media download endpoint exists and fixture runtime fails closed; not available without TDLib actor. |
| Search | PARTIAL | Shared attachment search may include downloaded Telegram attachments; no Telegram-specific media search API/UI. |
| Deduplication | PARTIAL | Blob SHA-256 dedup exists in shared storage; no Telegram-specific duplicate UX. |
| Scanner-backed clean verdict | MISSING | No real scanner backend marks Telegram attachments `clean`. |
| Persisted preview artifacts | MISSING | No durable Telegram preview artifact pipeline found. |

## Search

| Capability | Status | Evidence / Gap |
|---|---|---|
| Message search | PARTIAL | Local thread search/filter and shared Communication search exist; provider-side Telegram message search missing. |
| Dialog search | PARTIAL | UI filters loaded chat title locally; no dedicated backend dialog search endpoint. |
| Media search | MISSING | No Telegram media gallery/search endpoint found. |
| Provider search | MISSING | No stable TDLib provider-search API/UI found. |
| Topic search | MISSING | No topic search because topic projection is missing. |
| Member search | MISSING | No participant/member projection or search found. |

## Realtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| Generic transport | IMPLEMENTED | WebSocket/SSE/long-poll transports exist at platform level. |
| New message | PARTIAL | Generic event invalidation exists; no `telegram.message.created` event contract or cache patch found. |
| Edit message | MISSING | No edit event contract found. |
| Delete message | MISSING | No delete/tombstone event contract found. |
| Reaction update | MISSING | No reaction event contract found. |
| Sync update | MISSING | No Telegram sync progress event contract found. |
| Typing indicator | MISSING | No typing realtime contract found. |
| Media download progress | MISSING | No Telegram-specific media progress event contract found. |
| Command status | MISSING | No provider-command status event contract found. |

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
| Dialog list | PARTIAL | Virtualized chat list exists; provider folders/participant/admin state missing. |
| Message timeline | PARTIAL | Selected chat timeline exists; rich thread/reply/deletion history missing. |
| Thread view | PARTIAL | Selected chat thread exists; no reply-thread/topic view. |
| Composer | PARTIAL | Text send exists; attachment/voice commands disabled or missing. |
| Inspector | MISSING | Rail exists as placeholder; members/about/context missing. |
| Media viewer | MISSING | File cards and download button exist, but no viewer/gallery. |
| Search UI | PARTIAL | Local chat/thread search exists; no provider/global Telegram search UI. |
| Reactions UI | MISSING | No reaction UI found. |
| Edit/delete UI | MISSING | No provider command UI found. |
| Calls UI | PARTIAL | Call metadata/transcript routes exist; live UX missing. |
| Runtime/account setup UX | PARTIAL | Runtime and QR routes exist; complete account setup/capability UX incomplete. |

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

1. Detailed capability matrix.
2. Telegram typed realtime event contracts.
3. Message lifecycle schema: versions, tombstones, reply/forward/reaction/topic projection.
4. Provider-write command model beyond send.

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
5. Voice/call transcript review.

### P3 — Native/infra blocked

1. Bot API runtime.
2. Proxy/session bundles.
3. Voice/video recording.
4. Live calls.
5. Scanner-backed clean verdicts.
