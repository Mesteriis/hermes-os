# Telegram Architectural Blockers

Статус: audit blockers на 2026-06-15.

Блокеры ниже фиксируют причины, последствия и план решения. Они не являются
разрешением на реализацию новых крупных подсистем вне Telegram Channel.

## 1. Capability Contract Granularity

**Причина**: `/api/v1/telegram/capabilities` currently reports coarse runtime
and automation states. ADR-0091 requires every Telegram operation to be
represented in backend capability state before UI exposure.

**Последствия**: UI cannot reliably distinguish missing, blocked, degraded and
unsupported operations for edits, deletes, reactions, topics, exports, proxies,
session bundles, calls, media capture and destructive actions.

**План решения**:

- extend backend capability contract;
- model per-operation capability;
- include action class, scope, reason and confirmation gate;
- add fixture tests for every new state;
- block UI controls unless capability state allows them.

## 2. TDLib Runtime Dependency

**Причина**: Live user runtime depends on loadable native TDLib JSON runtime,
Telegram app credentials, QR-authorized account metadata and account-scoped
TDLib state paths.

**Последствия**: Live sync/send/media features are not generally available in CI
or on machines without configured TDLib resources. Fixture runtime must remain
the deterministic validation path.

**План решения**:

- keep live validation opt-in;
- document TDLib resource setup;
- preserve fixture runtime;
- only mark live capabilities `available` when runtime checks pass;
- expose runtime health diagnostics to UI.

## 3. Bot Runtime Missing

**Причина**: `telegram_bot` account setup and secret references exist, but no
Bot API runtime adapter is implemented.

**Последствия**: Bot accounts can be represented, but live bot send/sync must
remain `blocked`.

**План решения**:

- add separate ADR-backed Bot API runtime slice;
- keep bot credentials account-scoped and host-vault backed;
- separate user TDLib and Bot API capability matrices;
- avoid pretending bot is just another user runtime, because that path leads to
  the kind of architecture swamp humans keep lovingly recreating.

## 4. Mail-Named Blob / Storage Facade

**Причина**: Telegram media persistence currently uses mail-named storage modules
and tables, such as `MailStorageStore` and `communication_mail_blobs`, as the
existing Communication attachment/blob boundary.

**Последствия**: The implementation is functionally usable but semantically
confusing. It may incorrectly imply Telegram belongs to Mail.

**План решения**:

- introduce provider-neutral Communication attachment/blob facade;
- keep compatibility table names until a scoped storage refactor;
- update docs and code naming around public abstractions first;
- do not rename tables during documentation/audit phase.

## 5. No Tombstone / Version Schema

**Причина**: Current message projection upserts canonical communication rows.
There is no durable Telegram tombstone table, observed edit-version table or
deletion-history event model.

**Последствия**: Delete, restore visibility, edit history and diff views cannot
be safely implemented without losing source-evidence semantics.

**План решения**:

- add ADR before destructive commands;
- add append-only observed edit-version records;
- add tombstone/deletion evidence records;
- preserve raw provider evidence;
- expose local visibility separately from provider deletion;
- emit sanitized realtime events.

## 6. No Telegram Realtime Event Contracts

**Причина**: Generic WebSocket/SSE/long-poll transports exist, but Telegram does
not emit typed event contracts for new messages, edits, deletes, reactions,
media downloads or sync progress.

**Последствия**: Frontend must reload/query-invalidate manually and cannot
provide reliable live Telegram UX.

**План решения**:

- define sanitized `telegram.*` event payloads;
- never include message bodies/media bytes/secrets;
- add backend event emission at projection/command boundaries;
- add frontend cache patch handlers after backend contracts stabilize;
- preserve replay cursor behavior.

## 7. No Topic / Reaction / Reply / Forward Projection Schema

**Причина**: TDLib raw payload is preserved, but dedicated projection fields for
topics, replies, forwards, forward chains, mentions and reactions are not
modeled.

**Последствия**: UI can only show shallow selected-chat timelines and metadata
derived opportunistically from raw payload. Provider parity features would be
fragile if implemented directly against raw JSON.

**План решения**:

- add explicit projection tables or JSON contracts;
- model topic identity;
- model reply target and reply graph;
- model forward attribution;
- model reaction state;
- model mention state;
- migrate UI away from raw-payload guessing.

## 8. Provider-Write Command Model Beyond Send

**Причина**: Manual text send is the only implemented Telegram provider-write
command. Edit, delete, react, pin, mark read/unread, archive, join/leave and
admin commands do not share a durable command/outbox model.

**Последствия**: High-risk and destructive actions cannot be retried, audited,
explained, correlated or rolled back consistently.

**План решения**:

- design account-scoped provider command model;
- add idempotency keys;
- add per-target result records;
- require capability decisions;
- write sanitized audit metadata;
- emit command status events;
- add retry/degraded state.

## 9. Desktop Media Permissions

**Причина**: Voice/video recording and live calls require desktop microphone,
camera and device-selection boundaries. Current code has fixture call metadata
and fixture STT only.

**Последствия**: Voice/video messages, real local transcription, call accept,
decline, redial and audio capture remain blocked. Hidden recording stays
unsupported.

**План решения**:

- add separate Tauri/native permission ADR;
- model microphone/camera/device permission state;
- keep visible user confirmation;
- add local retention policy;
- reject hidden recording paths.

## 10. Provider Search Parity Missing

**Причина**: Current search is mostly local loaded chat/message filtering plus
shared Communication search. Provider-side TDLib search is not exposed as a
stable API/UI capability.

**Последствия**: Large Telegram histories cannot be searched reliably unless
already synced/projected locally.

**План решения**:

- add provider search capability state;
- add provider-side message/dialog/media search routes;
- add cursor pagination;
- merge provider search results with local projections without inventing source
  evidence;
- mark unsynced provider hits as preview/evidence candidates until projected.

## 11. Session / Proxy Bundle Missing

**Причина**: Session import/export and proxy profile persistence are absent.
Telegram runtime currently depends on local setup and account-scoped state.

**Последствия**: Local-first portability, backup/reconnect UX and proxy-bound
accounts are incomplete.

**План решения**:

- define encrypted session bundle contract;
- keep secrets host-vault backed;
- add proxy profile schema;
- add import/export audit records;
- reject raw secret-bearing exports by default.

## 12. Telegram AI Surface Missing

**Причина**: Telegram projected messages can feed shared engines, but Telegram
specific AI APIs/UI for summary, translation, bilingual reply, extraction review
and voice transcript review are missing.

**Последствия**: Telegram cannot yet match Mail’s AI-assisted communication
experience.

**План решения**:

- reuse Mail patterns where applicable;
- add Telegram-specific source evidence and confidence metadata;
- keep AI output review-only unless user/policy confirms;
- avoid implementing Obligation/Decision/Memory lifecycle inside Telegram.

## 13. Attachment Scanner Backend Missing

**Причина**: Telegram downloaded media uses the shared attachment/blob scanner
boundary. No real scanner backend emits `clean` verdicts.

**Последствия**: Attachments may remain `not_scanned`, and safe preview UX must
stay conservative.

**План решения**:

- reuse global attachment scanner backend once available;
- keep heuristic/no-op fallback;
- never mark downloaded Telegram media `clean` without scanner-backed verdict;
- quarantine suspicious/malicious media after scanner integration.

## 14. Cross-Domain Temptation

**Причина**: Telegram naturally exposes tasks, decisions, obligations, people,
organizations and project context. This makes it tempting to implement those
engines inside Telegram.

**Последствия**: Hermes would get channel-specific mini-domains and duplicate
business logic.

**План решения**:

- Telegram may emit candidates only;
- lifecycle belongs to shared engines;
- do not implement Obligation, Decision, Memory, Persona Intelligence or
  Organization Intelligence in Telegram Channel.
