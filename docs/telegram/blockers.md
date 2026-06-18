# Telegram Architectural Blockers

Status date: 2026-06-18.

Base Telegram Domain has no active architectural blockers.

## Closed Blockers

| Blocker | Resolution |
|---|---|
| Capability contract granularity | Operation-level capability contract includes `available`, `blocked`, `degraded`, `unsupported` and `planned`, with action class, reason and confirmation flags. |
| Provider-write command parity | Base provider writes use durable command rows, audit and provider-observed reconciliation. ACK is not success. |
| Message lifecycle evidence | Edit versions, tombstones, provider observations and diff metadata are durable. |
| Reply/forward projection | Reply/forward refs are idempotent; chains are bounded and cycle-guarded. |
| Topic projection/realtime | Topic projection, unread state, runtime topic events and reconciliation are implemented. |
| Provider search parity | Provider message/media search refreshes projection before returning UI-visible results. |
| Media boundary | Gallery, album metadata, preview, upload and download use the command/query model and shared Communication attachment boundary. |
| Frontend state ownership | Telegram production UI uses TanStack Query composables and shared realtime bootstrap. |

## Planned Work Outside Base Telegram

| Initiative | Reason |
|---|---|
| Bot Runtime | Separate provider/runtime model from TDLib user runtime. |
| Voice Recording / Voice Send | Requires explicit desktop microphone permission boundary. |
| Video Recording / Live Calls | Requires separate native device/call permission design. |
| Session Export / Session Import | Requires encrypted local-first session bundle ADR and audit. |
| MTProxy / SOCKS5 | Requires proxy profile model and connection policy. |
| AI Summary / Translation / Bilingual Reply / AI Review Flows | Belongs to AI Layer and shared review engines, not base Telegram. |

Hidden recording, Telegram private-data fine-tuning and untrusted third-party
plugin execution remain unsupported.
