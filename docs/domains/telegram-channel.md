# Telegram Channel Capability Spec

Status: Target specification with current implementation baseline.

Telegram is a Communications channel in Hermes Hub. It is not a separate
messenger domain and it is not the product thesis. Telegram provides source
evidence, provider interaction commands and a desktop workbench over local
memory.

Canonical flow:

```text
Telegram provider event
  -> raw source record
  -> canonical Communication projection
  -> extracted Knowledge / Memory / Relationships / Obligations / Decisions
  -> review and context
```

## Governing ADRs

- [ADR-0001 Event Sourcing as System Spine](../adr/ADR-0001-event-sourcing-as-system-spine.md)
- [ADR-0018 Provider Adapter Boundary](../adr/ADR-0018-provider-adapter-boundary.md)
- [ADR-0031 Temporary Desktop Only UI Scope](../adr/ADR-0031-temporary-desktop-only-ui-scope.md)
- [ADR-0046 Persistent Dev Mail Cache and Blob Storage](../adr/ADR-0046-persistent-dev-mail-cache-and-blob-storage.md)
- [ADR-0050 V4 Telegram Client, Policy Automation and Call Intelligence](../adr/ADR-0050-v4-telegram-client-policy-and-call-intelligence.md)
- [ADR-0052 Capability Runtime and Action Confirmation Policy](../adr/ADR-0052-capability-runtime-and-action-confirmation-policy.md)
- [ADR-0076 Host Vault on macOS](../adr/ADR-0076-host-vault-on-macos.md)
- [ADR-0083 Telegram Live User Client Runtime](../adr/ADR-0083-telegram-live-user-client-runtime.md)
- [ADR-0085 Communication Spine and Consistency / Contradiction Engine](../adr/ADR-0085-communication-spine-and-contradiction-engine.md)
- [ADR-0091 Telegram Production Client Capability Model](../adr/ADR-0091-telegram-production-client-capability-model.md)

## Current Implementation Baseline

Current repository evidence:

- backend store and projections:
  `backend/src/integrations/telegram/client.rs`;
- backend runtime manager:
  `backend/src/integrations/telegram/runtime.rs`;
- backend route handlers:
  `backend/src/integrations/telegram/api.rs`;
- TDLib JSON boundary:
  `backend/src/integrations/telegram/tdjson.rs`;
- API registration:
  `backend/src/app/router.rs`;
- current tests:
  `backend/tests/telegram.rs`;
- frontend API client:
  `frontend/src/lib/api/endpoints/telegram.ts`;
- frontend service:
  `frontend/src/lib/services/telegram.ts`;
- frontend page:
  `frontend/src/lib/pages/telegram/TelegramPage.svelte`;
- baseline migrations:
  `backend/migrations/0020_create_v4_telegram_policy_calls.sql` and
  `backend/migrations/0058_allow_empty_telegram_tdlib_message_bodies.sql`.

Implemented foundation includes:

- `telegram_user` and `telegram_bot` provider kinds;
- account-scoped Telegram credential purposes;
- fixture account setup;
- live/live-blocked account setup with host-vault credential writes;
- QR-login setup and status endpoints;
- runtime status and start endpoints;
- fixture and TDLib-oriented chat sync;
- selected history sync;
- fixture message ingestion;
- manual send routed through runtime manager;
- media download facade through the communication attachment/blob boundary;
- redacted provider-write audit for manual sends;
- automation policy dry-run foundation;
- call metadata and fixture transcript storage;
- frontend Telegram page with chat list, message thread, runtime actions, media
  download action, call/transcript panels and account wizard hooks.

This baseline is not production parity. New controls must be gated by
capability state until implemented and validated.

## Capability State Model

| State | Meaning |
|---|---|
| `available` | Runtime, storage, policy, audit, UI state and tests exist. |
| `blocked` | Architecturally allowed, but missing a required dependency, adapter, permission, secret or validation gate. |
| `degraded` | The capability exists but current runtime/account state cannot execute it reliably. |
| `unsupported` | Intentionally out of current scope or conflicts with Hermes policy. |

Action classes:

| Class | Examples |
|---|---|
| `read` | list chats, open message, preview local media, search local cache |
| `local_write` | save draft, local hide, local saved message, local folder overlay |
| `provider_write` | send, edit, react, mark read, archive, join, leave, pin |
| `destructive` | provider delete message, delete chat, remove participant, delete topic |
| `export` | export chat, files, Markdown, session bundle |
| `secret_access` | resolve API hash, bot token, session key, proxy password |
| `automation` | policy-authorized send dry-run or future live send |

## Accounts

| Capability | Description | Production requirement |
|---|---|---|
| Add account | Create `telegram_user` or `telegram_bot` account metadata and bind required secrets. | Host-vault secret storage, account-scoped validation, audit event. |
| Remove account | Disable account and stop runtime. | Preserve source evidence by default; destructive purge requires separate confirmation and capability. |
| Authorization | Support QR, phone/user flow where implemented, bot token setup and 2-step password handling. | No credentials in ordinary tables, logs or audit; visible setup state and expiry. |
| Logout | Stop runtime and revoke or discard session according to owner choice. | Explicit confirmation; local evidence remains. |
| Switch account | Change active UI account/chat scope. | Pure UI/read-model state; no runtime restart unless account start/stop changes. |
| Multiple accounts | Run several account actors simultaneously. | One actor, TDLib path, checkpoint set and secret binding namespace per account. |
| Import session | Restore an encrypted account-scoped TDLib/session bundle. | Host-vault unlock, manifest validation, account mismatch protection and audit. |
| Export session | Export encrypted session bundle for backup or migration. | Explicit export scope, no plaintext secrets, no export into project tree by default. |
| Proxy | Configure runtime networking through proxy profile. | Secret fields behind vault; active profile and errors visible. |
| MTProxy | Telegram MTProto proxy profile with host, port and secret. | Secret is vault payload; TDLib proxy operation through actor. |
| SOCKS5 | SOCKS5 proxy profile with host, port and optional username/password. | Password is vault payload; test/connect status redacted. |

## Chats

Chats are provider conversations plus local Hermes overlays.

| Capability | Description | Production requirement |
|---|---|---|
| List chats | Show account-scoped chats ordered by recent activity or local sort. | Reads from local projections and exposes sync status. |
| List groups | Filter chats with group/supergroup semantics. | Provider kind and permissions preserved in metadata. |
| List channels | Filter channel publication surfaces. | Channel posts remain Communication evidence. |
| List forums | Show forum-enabled chats and topics. | Topic identity and parent chat are first-class projections. |
| Archive | Move chat into archived provider/local view. | Provider write if mirrored; local overlay if not. |
| Pin | Pin chat in local or provider list. | Scope must be explicit: local-only or provider-synced. |
| Sort | Sort by time, unread, pinned, account, type, folder or local priority. | Derived read model; no provider mutation. |
| Filter | Filter by unread, mentions, pinned, projects, bots, archived, groups, channels and account. | Derived read model; counts must handle stale sync. |
| Folders | Group chats into local folders or provider-synced folders. | Local folder writes are local; provider folder writes are provider commands. |
| Search chats | Search local chat projections and optionally provider-side search. | Results label cache vs provider source. |
| Mark read | Mark chat or message range as read. | Provider write when synced; local read overlay otherwise. |
| Mark unread | Mark chat unread locally or provider-side where supported. | Capability must expose scope. |
| Archive/unarchive | Toggle archived view. | Audit provider writes if mirrored. |
| Hide | Hide from local workbench without provider mutation. | Local overlay and reversible state. |
| Delete | Delete chat/provider history request. | Destructive capability, confirmation, tombstone and retained local evidence. |

## Messages

| Capability | Description | Production requirement |
|---|---|---|
| Send message | Submit text or attachment message. | Provider-write command with idempotency key, preview hash, audit and retry state. |
| Edit message | Replace message content where provider permits. | Observed new version recorded; previous local version retained. |
| Delete message | Request provider deletion or local tombstone. | Destructive provider request and local tombstone history. |
| Reply | Send message linked to a target message. | Target account/chat/message identifiers preserved. |
| Forward | Forward message or local copy according to selected mode. | Source attribution and provider limits visible. |
| Copy | Copy text or attachment reference to clipboard. | Local UI action; no durable state unless saved/exported. |
| Multi-select | Select multiple messages for bulk operations. | Bulk command expands to per-message results. |
| Reactions | Add/remove reaction. | Provider write; unavailable reactions exposed as blocked/degraded. |
| Pin message | Pin/unpin in chat or topic. | Provider write with permission check. |
| Save message | Save to local saved view or Telegram Saved Messages. | Local save is local write; provider save is provider write. |
| Export message | Export selected message and attachments. | Export policy, explicit scope and source citation. |
| Go to message | Navigate to message in thread. | Read/navigation action. |
| Open reply thread | Show replies and thread context. | Provider thread IDs or local reply graph preserved. |

## Soft Delete

Local Telegram message evidence is not physically deleted by default.

| Requirement | Description |
|---|---|
| Messages never disappear locally by default | Raw source records and canonical events remain. |
| Delete becomes tombstone | A tombstone event marks local projection state. |
| Deleted by me | Tombstone reason `deleted_by_owner`. |
| Deleted by counterparty | Tombstone reason `deleted_by_counterparty` when observed from provider updates. |
| Deleted by Telegram | Tombstone reason `deleted_by_provider` or `moderation_removed`. |
| Show reason | UI displays known reason, unknown otherwise. |
| Show deletion time | UI displays observed deletion time and source event. |
| Deletion history | UI can show all tombstone/update events for the message. |

## Message History

Hermes versions observed message changes; it does not invent versions that were
never synced.

| Capability | Description |
|---|---|
| Version 1 | Initial observed message projection. |
| Version 2 | First observed edit/update. |
| Version 3 | Later observed edit/update. |
| Diff between versions | Compare local observed versions. |
| Edit time | Use provider edit timestamp when available, otherwise observed time. |
| Edit author | Use provider actor when available; otherwise mark unknown/provider-observed. |

## Attachments

Attachment classes:

| Type | Treatment |
|---|---|
| Images | Attachment metadata plus optional preview/blob. |
| Video | Attachment metadata, local blob and preview when generated. |
| Documents | Attachment metadata; may be saved into Documents domain. |
| PDF | Document-like attachment with extraction path. |
| DOCX | Document-like attachment with extraction path. |
| XLSX | Document-like attachment with extraction path. |
| ZIP | Opaque archive unless scanner/extractor allows inspection. |
| Audio | Media attachment with playback/download. |
| Voice | Audio attachment with waveform/speed/replay UI when available. |
| Video message | Video-note style media attachment. |
| Contact card | Source evidence that may create Persona identity candidates. |
| Location | Source evidence with map/location metadata. |
| GIF | Animation attachment. |
| Sticker | Sticker media metadata and local preview. |
| Link | Extracted link evidence and preview metadata. |

Attachment actions:

| Action | Description | Requirement |
|---|---|---|
| Download | Fetch provider media into local blob storage. | Size limits, progress, scanner state. |
| Open | Open local cached file. | No remote fetch without owner action unless policy allows. |
| Preview | Show safe preview when cached/scanned or explicitly allowed. | Unknown files remain constrained. |
| Save locally | Persist blob under local storage. | Stable path metadata and hash. |
| Save in Documents | Create/link Document artifact. | Document provenance references original message. |
| Share | Export or OS share action. | Explicit scope and privacy check. |
| Export | Include in chat/message export. | No secrets; include hashes and source citations. |

## Voice Messages

| Capability | Description | Requirement |
|---|---|---|
| Record | Capture microphone input. | Desktop permission, visible recording state and cancellation. |
| Send | Upload/send captured voice message. | Provider-write command and local attachment evidence. |
| Listen | Play local or downloaded voice attachment. | Local media UI. |
| Speed | Change playback speed. | Local UI state. |
| Download | Fetch voice media into blob storage. | Same scanner/blob rules as attachments. |
| Replay | Replay downloaded/cached voice message. | No provider mutation. |

## Video Messages

| Capability | Description | Requirement |
|---|---|---|
| Record | Capture camera/microphone video note. | Desktop media permission and visible recording state. |
| Send | Upload/send video message. | Provider-write command and local attachment evidence. |
| View | Play downloaded/cached video message. | Local media UI. |
| Download | Fetch video message media. | Same scanner/blob rules as attachments. |

## Calls

Current implementation stores call metadata and fixture transcripts. Live call
control is a later runtime slice.

| Capability | Description | Production requirement |
|---|---|---|
| Audio call | Start or accept audio call. | Native runtime, media permission, provider capability, audit. |
| Video call | Start or accept video call. | Separate runtime work; blocked until validated. |
| Call history | List observed calls. | Source-backed call records. |
| Accept | Accept incoming call. | Provider write and visible state transition. |
| Decline | Decline incoming call. | Provider write and audit. |
| Repeat call | Redial previous peer. | Provider write and confirmation. |
| Microphone | Choose input device. | Desktop permission and device boundary. |
| Camera | Choose video input device. | Desktop permission and device boundary. |
| Speakers | Choose output device. | Desktop permission/device state. |

Hidden recording is unsupported. Transcription is local by default and must be
policy/account/chat scoped.

## Channels

| Capability | Description | Requirement |
|---|---|---|
| Subscribe | Join or follow channel where provider permits. | Provider write and confirmation. |
| Unsubscribe | Leave/unfollow channel. | Provider write and confirmation. |
| Read posts | Sync/read channel publications. | Source evidence and pagination. |
| Search posts | Search local cache and optionally provider. | Results label source. |
| Pin | Pin channel locally or provider-side. | Scope explicit. |
| Group | Group channels in local folders. | Local write unless provider folder sync selected. |
| Archive | Archive channel. | Provider write or local overlay. |

## Groups

| Capability | Description | Requirement |
|---|---|---|
| Create | Create group. | Provider write, capability check and confirmation. |
| Delete | Delete group or local group view. | Destructive capability and tombstone. |
| Join | Join group or accept invite. | Provider write. |
| Leave | Leave group. | Provider write and confirmation. |
| Invite participants | Invite users. | Provider write and permission check. |
| Remove participants | Remove users. | Destructive/admin capability. |
| View participants | Read participant list. | Provider permissions and rate limit handling. |
| Search participants | Search local/provider participant list. | Results label cache/provider source. |

Participant observations may create Persona identity candidates. They do not
create a separate Contacts domain.

## Forums

| Capability | Description | Requirement |
|---|---|---|
| Create topic | Create forum topic. | Provider write and permission check. |
| Close topic | Close topic. | Provider write/admin capability. |
| Delete topic | Delete topic. | Destructive capability and local tombstone. |
| Reply in topic | Send message scoped to topic. | Provider write with topic ID. |
| Search topics | Search local/provider topic list. | Cache/provider source labeling. |
| Pin topic | Pin topic. | Provider write/admin capability. |

## Search

| Search mode | Description |
|---|---|
| Global search | Search across Telegram and other Hermes evidence. |
| Messages | Text/body search over projected messages. |
| Files | Attachment filename/content metadata search. |
| Images | Image attachment metadata and OCR/vision-derived text when available. |
| Video | Video attachment metadata. |
| Links | Extracted URL and preview search. |
| People | Persona identity traces from Telegram participants. |
| Groups | Group chat search. |
| Channels | Channel search. |
| Date range | Filter by occurred/observed/projected time. |
| Sender | Filter by provider sender or resolved Persona. |
| Attachment type | Filter by attachment/media class. |

Search indexes are derived and rebuildable.

## Drafts

| Capability | Description |
|---|---|
| Drafts | Local draft records keyed by account/chat/topic/reply target. |
| Autosave | Save draft changes without provider mutation. |
| Restore | Restore draft when returning to chat/thread. |
| Delete draft | Remove local draft; provider draft deletion is separate provider write. |

## Notifications

| Capability | Description |
|---|---|
| New messages | Notify from provider updates/local sync. |
| Replies | Notify for reply-thread activity. |
| Reactions | Notify for message reactions. |
| Mentions | Notify when owner is mentioned. |
| Calls | Notify for incoming/missed calls. |
| Notification settings | Store typed local settings; provider sync only if implemented. |
| Mute chat | Local or provider mute, scope labeled. |
| Mute group | Local or provider mute, scope labeled. |
| Mute channel | Local or provider mute, scope labeled. |

OS notifications must support privacy-safe previews.

## Telegram Address Book Data

The Telegram address book is provider data. Hermes maps observed users to
Persona identity traces and Relationship candidates.

| Capability | Description |
|---|---|
| List | Show observed Telegram users/address book entries. |
| Search | Search by name, username, phone hash where allowed and local identity. |
| Add | Provider write to add Telegram contact where supported. |
| Delete | Provider write/destructive remove operation. |
| Edit | Provider write where supported. |
| Import | Import address book data as source evidence. |
| Export | Sensitive export action. |

Do not duplicate this as a Contacts domain.

## Media Gallery

| Gallery | Description |
|---|---|
| Images | Derived from image attachments. |
| Video | Derived from video attachments. |
| Documents | Derived from document attachments and Documents links. |
| Links | Derived from extracted links and previews. |
| Filter media | Filter by account, chat, sender, date, type, scan state and local cache state. |

## Offline

| Capability | Description |
|---|---|
| Local cache | Read synced chats, messages, metadata and cached media without provider connectivity. |
| Work offline | Compose drafts, review memory, search local cache and queue provider writes. |
| Outbox queue | Store pending provider-write commands with idempotency keys. |
| Retry send | Retry queued commands with backoff and visible failure state. |
| Background sync | Sync accounts according to settings and runtime state. |
| Manual sync | Owner-triggered sync with progress and checkpoint reporting. |

Offline commands must not silently execute under the wrong account after
reconnect.

## Export

| Capability | Description |
|---|---|
| Export chat | Export one chat with selected message/media/history scope. |
| Export group | Export group evidence with participant/topic metadata. |
| Export channel | Export channel posts and selected media. |
| Export files | Export selected attachment blobs and metadata. |
| Export messages | Export selected messages. |
| Markdown export | Render ordered messages with sender, time, source citations, tombstones, edit history and attachment references. |

Exports must not include secrets, tokens, session keys, proxy passwords or
private runtime paths unless explicitly transformed into safe relative manifest
entries.

## Desktop UX

Hermes Telegram UX targets desktop only while ADR-0031 is active.

| Capability | Description |
|---|---|
| Drag and drop files | Create attachment upload draft. |
| Drag and drop images | Create image upload draft/preview. |
| Context menu | Route to the same backend commands as visible actions. |
| Hotkeys | Keyboard access for navigation and safe commands. |
| Multi-window mode | Open multiple chat views over one shared runtime/cache. |
| Tray minimize | Keep UI available without implying sync is enabled. |
| OS notifications | Use privacy-safe notification rendering. |
| Multiple chats open | Show several chat panes/windows without duplicate account actors. |

## Production Readiness Checklist

A Telegram capability is production-ready only when all of these are true:

- backend capability state exists and is exposed;
- storage schema or projection is durable and replayable where required;
- source evidence and canonical event behavior are defined;
- provider writes go through capability policy;
- destructive actions create tombstones and require confirmation;
- audit records are redacted and fail closed for high-risk actions;
- secrets are resolved through the host-vault boundary;
- local blob storage and scanner state are used for media bytes;
- frontend UI shows available, blocked, degraded and unsupported states;
- fixture tests cover success, rejection and degraded paths;
- live TDLib smoke validation is opt-in and documented;
- docs and ADRs match the implemented behavior.

## Suggested Delivery Slices

1. Account/session/proxy lifecycle and capability contract.
2. Chat projections, folders, read/unread/archive/pin overlays and search.
3. Message command model, drafts, replies, forwards, edits and reactions.
4. Tombstone and observed message-version persistence.
5. Attachment/media gallery hardening and Documents save path.
6. Offline outbox, retry and background/manual sync progress.
7. Export and Markdown rendering with source citations.
8. Voice/video message capture and send.
9. Calls runtime, device selection and local transcription policy.
10. Desktop UX hardening across multi-window, tray, hotkeys and OS
    notifications.
