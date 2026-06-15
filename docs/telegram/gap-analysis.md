# Telegram Gap Analysis

Status date: 2026-06-15.

Labels:

- `IMPLEMENTED` - реализовано и подтверждено текущими files/tests/docs audit.
- `PARTIAL` - есть meaningful foundation, но production scope не закрыт.
- `BROKEN` - реализация есть, но current evidence показывает, что она не работает.
- `MISSING` - durable implementation в текущем репозитории не найден.
- `REGRESSION` - current behavior хуже ранее задокументированного поведения.

Evidence sources: `backend/src/integrations/telegram/`, `backend/tests/telegram.rs`,
`backend/migrations/0020_create_v4_telegram_policy_calls.sql`,
`backend/migrations/0058_allow_empty_telegram_tdlib_message_bodies.sql`,
`backend/src/platform/audit/telegram.rs`, `backend/src/platform/calls/`,
`frontend/src/domains/telegram/`, `docs/adr/ADR-0050-*`,
`docs/adr/ADR-0083-*`, `docs/adr/ADR-0091-*`.

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
| Session import/export | MISSING | No bundle schema/API/UI found. |
| Proxy / MTProxy / SOCKS5 | MISSING | No proxy profile schema/API/runtime command found. |

## Messaging

| Capability | Status | Evidence / Gap |
|---|---|---|
| Dialogs | PARTIAL | `telegram_chats` projection and chat list endpoint exist; provider folders/archived/pinned commands are missing. |
| Private Chats | PARTIAL | `private` chat kind exists and selected history sync works; no full provider parity. |
| Groups | PARTIAL | `group` chat kind exists; participant/admin/group lifecycle actions missing. |
| Supergroups | PARTIAL | TDLib supergroups are collapsed into `group`; no first-class supergroup metadata. |
| Channels | PARTIAL | `channel` chat kind exists; subscribe/unsubscribe/post admin actions missing. |
| Topics | MISSING | No topic table/API/projection found. |
| Replies | MISSING | No reply target command/projection fields found. |
| Reply Chains | MISSING | No reply graph/thread model found. |
| Forwards | MISSING | No forward command or attribution model found. |
| Forward Chains | MISSING | No forward-chain model found. |
| Mentions | PARTIAL | UI can filter `mention_count` metadata if present; parser/projection does not derive mentions. |
| Pinned Messages | PARTIAL | UI can derive pinned messages from metadata; provider pin/unpin and sync are missing. |

## Message Lifecycle

| Capability | Status | Evidence / Gap |
|---|---|---|
| Send | PARTIAL | Manual text send works through fixture/TDLib QR runtime and redacted audit; attachment sends and queued outbox are missing. |
| Edit | MISSING | No edit route/command/schema found. |
| Delete | MISSING | No Telegram delete route/command/tombstone schema found. |
| Restore Visibility | MISSING | No local tombstone/restore model found. |
| Message History | PARTIAL | Selected/older history sync exists; observed edit versions/diffs/deletion history missing. |

## Reactions

| Capability | Status | Evidence / Gap |
|---|---|---|
| Add Reaction | MISSING | No provider command/API found. |
| Remove Reaction | MISSING | No provider command/API found. |
| Reaction Sync | MISSING | No parser/projection/realtime contract found. |

## Media

| Capability | Status | Evidence / Gap |
|---|---|---|
| Photos | PARTIAL | TDLib `messagePhoto` caption text can be parsed and raw payload retained; first-class photo metadata/gallery missing. |
| Videos | PARTIAL | TDLib `messageVideo` caption text can be parsed and raw payload retained; viewer/gallery missing. |
| Documents | PARTIAL | TDLib `messageDocument` caption text can be parsed; download can create attachment row when file metadata is supplied. |
| Voice Messages | PARTIAL | `messageVoiceNote` text handling exists; voice playback/record/send UI missing. |
| Video Notes | MISSING | No dedicated parser/UI/runtime path found. |
| Audio | PARTIAL | `messageAudio` caption text can be parsed; audio playback/gallery missing. |
| Stickers | MISSING | No first-class sticker parser/projection/UI found. |
| GIF | MISSING | No first-class GIF parser/projection/UI found. |

## Attachments

| Capability | Status | Evidence / Gap |
|---|---|---|
| Preview | PARTIAL | Shared Communication attachment preview may work for downloaded blobs; Telegram workbench has no dedicated preview surface. |
| Download | PARTIAL | TDLib media download endpoint exists and fixture runtime fails closed; not available without TDLib actor. |
| Search | PARTIAL | Shared attachment search may include downloaded Telegram attachments; no Telegram-specific media search API/UI. |
| Deduplication | PARTIAL | Blob SHA-256 dedup exists in shared storage; no Telegram-specific duplicate UX. |

## Search

| Capability | Status | Evidence / Gap |
|---|---|---|
| Message Search | PARTIAL | Local thread search/filter and shared Communication search exist; provider-side Telegram message search missing. |
| Dialog Search | PARTIAL | UI filters loaded chat title locally; no dedicated backend dialog search endpoint. |
| Media Search | MISSING | No Telegram media gallery/search endpoint found. |

## Realtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| New Message | PARTIAL | Generic event transports exist; no `telegram.message.created` event contract or cache patch found. |
| Edit Message | MISSING | No edit event contract found. |
| Delete Message | MISSING | No delete/tombstone event contract found. |
| Reaction Update | MISSING | No reaction event contract found. |
| Sync Update | MISSING | No Telegram sync progress event contract found. |

## AI

| Capability | Status | Evidence / Gap |
|---|---|---|
| Summary | PARTIAL | Telegram projections can feed shared message intelligence; Telegram-specific summary API/UI missing. |
| Translation | MISSING | No Telegram route/UI found. |
| Bilingual Reply | MISSING | No Telegram route/UI found. |
| Task Extraction | PARTIAL | Telegram messages can refresh obligation-derived task candidates; review engine is out of Telegram scope. |
| Note Extraction | MISSING | No Telegram-specific note extraction path found. |
| Event Extraction | MISSING | No Telegram-specific event candidate UI/API found. |
| Persona Extraction | PARTIAL | Sender metadata is preserved as identity trace; Persona review lifecycle is outside Telegram scope. |

## UI

| Capability | Status | Evidence / Gap |
|---|---|---|
| Dialog List | PARTIAL | Virtualized chat list exists; provider folders/participant/admin state missing. |
| Message Timeline | PARTIAL | Selected chat timeline exists; rich thread/reply/deletion history missing. |
| Thread View | PARTIAL | Selected chat thread exists; no reply-thread/topic view. |
| Media Viewer | MISSING | File cards and download button exist, but no viewer. |
| Search UI | PARTIAL | Local chat/thread search exists; no provider/global Telegram search UI. |

## Scope Boundary

| Capability | Status | Evidence / Gap |
|---|---|---|
| Obligation Engine | PARTIAL | Telegram can create candidates through shared projection refresh; implementation is outside this audit scope. |
| Decision Engine | PARTIAL | Telegram can create suggested decisions from evidence; lifecycle is outside Telegram scope. |
| Memory Engine | MISSING | No Telegram-specific Memory Engine implementation intended in this phase. |
| Knowledge Engine | MISSING | No Telegram-specific Knowledge Engine implementation intended in this phase. |
| Cross-Domain Integrations | MISSING | Only integration points are documented; implementation is out of scope. |
