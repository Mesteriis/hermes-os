# Telegram Extended Status Details

Status date: 2026-06-18.

Base Telegram Domain: `COMPLETED`.

## Search

- Dialog, message, provider, media, topic and member search routes are
  implemented.
- Provider message/media search refreshes TDLib results into local projections
  before returning UI-visible results.
- Search UI uses TanStack Query composables and projection-backed result panels.

## Media And Attachments

- Telegram media metadata is projected from source evidence.
- Album metadata is preserved in message attachment metadata and gallery views.
- Upload uses shared Communication attachment import plus Telegram provider
  command rows.
- Download emits started/progress/failed/completed realtime events, persists
  local blob/attachment rows and patches projected attachment metadata.
- Preview uses the shared Communication attachment preview boundary and local
  downloaded media paths.

## Realtime

- Telegram runtime and API events use the shared event bus/bootstrap.
- Frontend cache patching covers message, chat, command, media, typing, topic,
  pinned/search and runtime event families.
- Realtime is preferred over polling where a realtime path exists. QR login
  status remains a bounded authorization-status flow because it is not a
  message/dialog runtime event stream.

## Frontend

- Production Telegram server state is accessed through TanStack Query
  composables.
- Component-level `fetch(` is absent from Telegram production components.
- Page-level orchestration delegates backend state to query/mutation composables
  and keeps Pinia for local UI state only.

## Audit And Safety

- Destructive and provider-write actions record redacted audit metadata.
- Message bodies, media bytes and secrets are not logged in Telegram event
  payloads.
- Secret payloads remain outside PostgreSQL; provider account records store
  metadata and secret references only.

## Scope Boundary

Telegram emits evidence and traces for shared systems. It does not implement
Memory, Knowledge, Persona, Organization, Project, Obligation or Decision
lifecycle.

The following remain separate planned initiatives: Bot Runtime, Voice,
Video/Calls, Session import/export, MTProxy, SOCKS5 and Telegram-specific AI
flows.
