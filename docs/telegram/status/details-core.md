# Telegram Core Status Details

Status date: 2026-06-18.

Base Telegram Domain: `COMPLETED`.

## Account And Runtime

- Fixture and live user account metadata are implemented.
- QR authorization routes and UI are implemented.
- Runtime status/start/stop/restart diagnostics are implemented.
- Bot Runtime is deferred as `planned` under ADR-0094.

## Capability Contract

- Global and account-scoped capability responses expose operation status,
  action class, reason, confirmation requirement and account overrides.
- Supported states are `available`, `blocked`, `degraded`, `unsupported` and
  `planned`.
- Deferred initiatives are API-visible as `planned`.

## Dialogs

- Chat projections, detail routes, folder filters and selected-chat controls are
  implemented.
- Pin/archive/mute/read/unread/folder commands use provider-write command rows.
- TDLib chat-state events reconcile provider-observed state before commands are
  completed.
- Folder add/remove/reassign use provider folder evidence and realtime patches.

## Messages

- Source-backed message projection and sanitized raw evidence access are
  implemented.
- Text send, edit, delete, restore, pin, reply, forward and reaction commands
  use capability gates and command/audit records.
- Provider-observed edits and deletes produce versions/tombstones and realtime
  events.
- Diff metadata records previous/new previews, lengths and hashes.

## References

- Reply refs and forward refs are idempotent.
- Reply chains traverse ancestors and descendants with fixed bounds and cycle
  guard.
- Forward chains traverse locally projected provider attribution with fixed
  bounds and cycle guard.
- UI consumes projected summaries, not raw TDLib payloads.

## Topics

- Forum topics are projected, listed, searched and shown in the workbench.
- Topic unread state and last-message state are persisted.
- Runtime topic events patch frontend caches and reconcile close/reopen
  commands from provider-observed state.

## Reactions

- Add/remove commands use the outbox and audit.
- TDLib history/runtime interaction updates project provider reaction aggregates.
- Self reaction commands reconcile from provider-observed chosen state.
