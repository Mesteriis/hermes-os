# Storage Architecture

## Storage Goals

- durable local ownership
- reproducible projections
- source preservation
- efficient search and graph traversal
- clear backup and restore
- replaceable indexes

## Storage Components

| Component | Role |
| --- | --- |
| PostgreSQL | canonical relational data, event tables, graph tables, metadata |
| Object storage | documents, attachments, OCR artifacts, extracted text |
| Tantivy | full text search indexes |
| Vector index | semantic retrieval indexes |
| Backup storage | encrypted local or self-hosted snapshots |

## PostgreSQL Responsibilities

- event envelope and payload storage
- normalized entities
- relationship objects
- task lifecycle
- provider account metadata
- ingestion checkpoints
- projection offsets
- permissions and capability grants
- audit trails

Backend readiness verifies that the embedded SQLx migration ledger has the expected successful migration count and latest version before reporting the service ready.

## Object Storage Responsibilities

- immutable raw attachments
- imported document versions
- generated OCR text
- document previews
- extracted structured artifacts

Object keys must not encode secrets. Metadata that affects business logic belongs in PostgreSQL.

## Index Responsibilities

Indexes are derived. Tantivy and vector indexes may be rebuilt from canonical events, objects and relational state. Index corruption must be recoverable without losing memory.

## Backup Model

Backups must include:

- PostgreSQL dump or physical snapshot
- object storage content
- index rebuild metadata
- application configuration excluding secrets where possible
- encrypted secret export when explicitly requested

Restore must verify schema versions, projection offsets and index consistency.
