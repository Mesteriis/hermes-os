# Telegram Implementation Status

Status date: 2026-06-18.

Base Telegram channel capability set: `COMPLETED`.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

## Summary

| Area | Status |
|---|---|
| Communication Channel framing | DONE |
| Provider account/runtime metadata | DONE |
| Capability contract with `planned` state | DONE |
| Provider-write outbox and audit | DONE |
| Provider-observed reconciliation | DONE |
| Dialogs, folders and unread state | DONE |
| Message lifecycle evidence | DONE |
| Reply/forward graph evidence | DONE |
| Topics | DONE |
| Reactions | DONE |
| Search | DONE |
| Media and attachments | DONE |
| Realtime event bus/bootstrap | DONE |
| TanStack Query frontend state | DONE |
| Documentation closure | DONE |

## Deferred Initiatives

ADR-0094 and ADR-0097 move the following outside the base Telegram channel
capability set: Bot Runtime, Voice, Video/Calls, Session import/export, MTProxy,
SOCKS5 and Telegram-specific AI flows. Their capability state is `planned`.

## Validation Policy

Live TDLib validation is opt-in and depends on local credentials/native
resources. Deterministic closure uses fixture, projection, outbox and realtime
regression tests.

## Navigation

- [Pass Log](status/pass-log.md)
- [Core Details](status/details-core.md)
- [Extended Details](status/details-extended.md)
