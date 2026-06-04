# Backend

Rust backend for Hermes Hub.

Current scope is intentionally small: an executable backend foundation with configuration parsing, health/readiness endpoints, V1 read API status, canonical event append/read API, event log storage, API access audit logging, secret reference metadata, communication ingestion storage, message/contact/document projection boundaries, Tantivy search boundary, projection cursors and projection runner batch semantics. Provider adapters, knowledge graph integration and agent runtime are not implemented yet.

## Commands

From the repository root:

```sh
make backend-run
make backend-run-dev
make backend-smoke-dev
make backend-storage-smoke-dev
make backend-secrets-smoke-dev
make backend-event-log-smoke-dev
make backend-communication-smoke-dev
make backend-email-import-smoke-dev
make backend-messages-smoke-dev
make backend-contacts-smoke-dev
make backend-documents-smoke-dev
make backend-search-smoke-dev
make backend-projection-smoke-dev
make backend-projection-runner-smoke-dev
make backend-events-api-smoke-dev
make backend-v1-api-smoke-dev
make backend-validate
```

Direct Cargo commands:

```sh
cargo run --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

## Environment

Supported environment variables:

- `HERMES_HTTP_ADDR` - backend bind address, defaults to `127.0.0.1:8080`.
- `DATABASE_URL` - optional PostgreSQL URL. The current health endpoint does not require a database connection.
- `HERMES_LOCAL_API_TOKEN` - temporary local capability token required for local event API endpoints.
- `HERMES_LOCAL_WRITE_TOKEN` - legacy fallback for `HERMES_LOCAL_API_TOKEN` during transition from ADR-0037.

## Endpoints

- `GET /healthz` - returns backend health status and service name.
- `GET /readyz` - returns readiness status; it is `503` when PostgreSQL is not configured, unavailable or missing required SQLx migrations.
- `GET /api/v1/status` - returns enabled V1 surfaces. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `POST /api/events` - appends a canonical event through the application/API boundary. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `GET /api/events/{event_id}` - loads a canonical event by ID. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `GET /api/audit/events` - returns event API audit records. Supports `target_id`, `actor_id`, `after_audit_id` and `limit` query parameters. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.

Authorized event API calls are recorded in `api_audit_log` with `actor_kind` and `actor_id`. The API token value is never stored.

## Migrations

Backend startup applies local PostgreSQL migrations when `DATABASE_URL` is configured.
Readiness checks verify that the embedded SQLx migration ledger has the expected successful migration count and latest version.

Current schema:

- `event_log` - append-only canonical event log with JSONB envelope fields, replay ordering, idempotent source index and mutation-prevention triggers.
- `projection_cursors` - monotonic per-projection replay cursor positions.
- `api_audit_log` - append-only operational audit records for local event API access attempts, including non-secret local actor IDs.
- `secret_references` - non-secret metadata pointers to external secret stores; secret values are never stored in PostgreSQL.
- `communication_provider_accounts` - non-secret email provider account metadata for `gmail`, `icloud` and `imap`.
- `communication_raw_records` - append-only raw provider records with idempotent provider identity, source fingerprints, import batches and provenance.
- `communication_ingestion_checkpoints` - per-account provider stream checkpoints for retryable ingestion.
- `communication_provider_account_secret_refs` - maps provider accounts to secret references by credential purpose.
- `communication_messages` - canonical message projection records derived from raw communication records.
- `contacts` - contact projection records keyed by unique email address.
- `documents` - imported document records with source fingerprints and extracted text.

Relevant design documents:

- [Architecture Overview](../docs/architecture/architecture-overview.md)
- [Event Model](../docs/architecture/event-model.md)
- [Storage Architecture](../docs/architecture/storage-architecture.md)
- [ADR Index](../docs/adr/README.md)
