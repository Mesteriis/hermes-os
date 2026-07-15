# ADR-0041 Email Provider Ingestion Foundation

Status: Proposed

## Context

Version 1.0 requires the first communication source and an event-backed ingestion pipeline. Email must support more than one provider shape from the start:

- Gmail through the Gmail API and OAuth.
- iCloud Mail through IMAP with app-specific credentials.
- Generic IMAP for self-hosted or provider-neutral mailboxes.

Implementing a concrete adapter before defining raw source preservation, idempotency and checkpoints would push provider quirks into application code and make retries unsafe.

## Decision

Create a provider-neutral email ingestion storage boundary before implementing concrete provider adapters.

Rules:

- Supported initial email provider kinds are `gmail`, `icloud` and `imap`.
- Provider account records store non-secret account metadata and non-secret adapter configuration only.
- OAuth tokens, app passwords and mailbox passwords must stay behind the secret boundary from ADR-0016 and must not be stored in provider account config.
- Provider account credentials are represented through secret references from ADR-0042.
- Raw provider records are stored append-only in `communication_raw_records`.
- Raw provider record identity is idempotent by `(account_id, record_kind, provider_record_id)`.
- Raw records keep `source_fingerprint`, `import_batch_id`, provider payload and provenance for replay/debugging.
- Ingestion checkpoints are stored per `(account_id, stream_id)` with provider-specific JSON payloads.
- Gmail adapters should checkpoint Gmail history streams, for example `stream_id = gmail:history`.
- iCloud and generic IMAP adapters should checkpoint mailbox streams, for example `stream_id = imap:INBOX`, with UID validity and last seen UID data.
- This decision establishes the storage boundary before provider networking. ADR-0043 adds read-only Gmail API and IMAP networking against this boundary.

## Consequences

- Gmail, iCloud and generic IMAP can share one ingestion persistence contract.
- Provider adapters can be retried without duplicating raw source records.
- Raw records remain available for replay, parser fixes and future projection rebuilds.
- Provider account metadata is separated from secrets.
- The next implementation slice can build a read-only adapter against this boundary instead of inventing persistence inside the adapter.
