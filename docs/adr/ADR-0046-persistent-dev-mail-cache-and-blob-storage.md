# ADR-0046 Persistent Dev Mail Cache and Blob Storage

Status: Proposed

## Context

ADR-0041 defines provider-neutral email ingestion, ADR-0043 requires read-only provider networking, ADR-0044 keeps credentials behind the secret boundary, and ADR-0032 keeps persistent development data under `docker/data/`.

The desktop UI now needs realistic local mail data for development: after starting the dev stack, previously downloaded messages should be visible without reconnecting to the provider or re-entering credentials. At the same time, email messages can contain large raw MIME payloads and attachments that do not belong in PostgreSQL as ordinary row data.

## Decision

Use a persistent local mail cache split by responsibility:

- PostgreSQL stores provider accounts, mailbox checkpoints, message metadata, searchable extracted content, attachment metadata, projections, graph references and search index references.
- Local blob storage stores heavy or opaque mail bytes: raw `.eml` payloads, attachment bytes, previews and future extracted attachment artifacts.
- Development blob storage lives under `docker/data/mail/` and is ignored by Git.
- Database rows reference local blobs by stable metadata: storage kind, relative storage path, SHA-256 digest, byte size, content type and optional filename.
- Attachments are represented as first-class metadata records linked to canonical messages and source raw records.
- The system must not store mailbox credentials, OAuth tokens or app passwords in blob paths, blob metadata, database payloads, logs or fixture files.
- Read-only provider sync must remain non-mutating: IMAP uses `EXAMINE` plus `BODY.PEEK[]`; Gmail uses read-only API scopes.
- `make dev` should be allowed to reuse already downloaded local cache data and should not require provider connectivity for the UI to display previously downloaded messages.
- `make reset-data` remains the explicit destructive command for local development cache removal.

Initial implementation may keep fixture import redacted and attachment-free. Full provider sync should evolve toward storing raw MIME and attachments through the blob store while projecting only normalized metadata and extracted text into PostgreSQL.

## Consequences

- The UI can be built against persistent realistic local data instead of synthetic mocks.
- PostgreSQL remains optimized for queries, search and relationships instead of becoming a large binary object store.
- Local development data is durable across `make dev` restarts but remains outside Git.
- Backup and restore must eventually include both PostgreSQL state and `docker/data/mail/` blob state.
- Blob garbage collection, attachment extraction quality, previews and remote/self-hosted object storage require later ADR-backed implementation details if they change this local-first storage contract.
