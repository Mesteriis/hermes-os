# ADR-0043 Read-Only Email Provider Networking

Status: Proposed

## Context

ADR-0041 created provider-neutral email ingestion storage, ADR-0042 put provider credentials behind secret references, and the email sync preflight boundary validates provider-specific configuration before adapters run.

The remaining product risk was that Gmail, iCloud Mail and generic IMAP still had no real network path. Keeping networking outside the codebase would hide provider API, TLS, credential and checkpoint assumptions until too late.

## Decision

Implement read-only provider network clients behind the email sync boundary.

Rules:

- Gmail sync uses the Gmail REST API with an OAuth Bearer access token resolved at runtime through the secret boundary.
- Gmail list requests use `users.messages.list`; raw message retrieval uses `users.messages.get` with `format=raw`.
- Gmail provider records preserve Gmail `id`, `threadId`, `historyId`, `internalDate`, labels and raw base64url RFC 2822 payload.
- iCloud and generic IMAP sync use the same IMAP client with provider kind selected by account configuration.
- IMAP sync authenticates with username plus app password/password resolved through the secret boundary.
- IMAP sync opens mailboxes read-only with `EXAMINE`, discovers new messages with `UID SEARCH`, and retrieves raw RFC 2822 payloads with `UID FETCH`.
- IMAP sync must not issue `STORE`, delete, flag mutation or mailbox mutation commands.
- Provider network clients produce `EmailSyncBatch` values that are persisted through the existing raw record and checkpoint stores.
- Secret values must not be stored in raw payloads, provenance, checkpoint JSON, logs or errors.
- OAuth grant/refresh UX and encrypted vault storage are handled by ADR-0044 as account setup and secret acquisition concerns, not provider networking concerns.

## Consequences

- Hermes Hub can execute real read-only Gmail API and iCloud/raw IMAP networking paths.
- Provider networking remains isolated from canonical projections and HTTP handlers.
- Multiple accounts are still handled through account-scoped plans and credentials.
- Gmail API behavior and IMAP wire behavior are covered by local network tests without requiring real external credentials.
- Account setup can provide refreshed runtime access tokens without changing raw ingestion storage.
