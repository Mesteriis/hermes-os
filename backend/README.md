# Backend

Rust backend for Hermes Hub.

Current scope includes an executable backend foundation with configuration parsing, health/readiness endpoints, V1 status API, canonical event append/read API, event log storage, API access audit logging, encrypted secret vault, Gmail/iCloud/IMAP account setup, secret reference metadata, communication ingestion storage, email sync preflight planning, read-only Gmail API and IMAP provider networking, fixture email import/export, local mail blob/attachment metadata storage, message/contact/document projection boundaries, Tantivy search boundary, projection cursors, projection runner batch semantics, V2 graph core projection/read APIs, protected V2 workflow APIs for projects, task candidates, contact identity review and document processing, and V3 local AI workflow APIs backed by Ollama plus pgvector semantic retrieval. OS keychain resolver, full MIME parsing, attachment extraction, graph editing, richer graph inference and autonomous agents are not implemented yet.

## Commands

From the repository root:

```sh
make backend-run
make backend-run-dev
make backend-watch-dev
make backend-smoke-dev
make backend-storage-smoke-dev
make backend-secrets-smoke-dev
make backend-event-log-smoke-dev
make backend-communication-smoke-dev
make backend-email-sync-smoke-dev
make backend-email-provider-network-smoke-dev
make backend-email-sync-cache-dev
make backend-email-fixture-export-icloud-dev
make backend-email-fixture-import-dev
make backend-email-fixture-project-dev
make backend-account-setup-smoke-dev
make backend-email-import-smoke-dev
make backend-messages-smoke-dev
make backend-contacts-smoke-dev
make backend-documents-smoke-dev
make backend-graph-smoke-dev
make backend-v2-workflow-smoke-dev
make backend-ai-smoke-dev
make backend-graph-project-dev
make backend-search-smoke-dev
make backend-projection-smoke-dev
make backend-projection-runner-smoke-dev
make backend-events-api-smoke-dev
make backend-v1-api-smoke-dev
make backend-validate
```

Graph core smoke:

```bash
make backend-graph-smoke-dev
```

This starts the local PostgreSQL container, runs graph store, projection and read API tests with `HERMES_TEST_DATABASE_URL`, then stops the Compose PostgreSQL service on exit. Do not run this while relying on the same Compose PostgreSQL service for an active development database session.

V2 workflow smoke:

```bash
make backend-v2-workflow-smoke-dev
```

This starts the local PostgreSQL container, creates isolated temporary databases on the dev PostgreSQL server, and runs the project, project API, project link review, task candidate, task candidate API, contact identity, contact identity API, document processing and document processing API integration suites serially. The target is included in `make validate`.

V3 AI smoke:

```bash
make backend-ai-smoke-dev
```

This starts the local PostgreSQL container for pgvector/API integration tests and runs live Ollama validation against `http://192.168.1.2:11434` by default. Override the smoke endpoint with `HERMES_AI_SMOKE_OLLAMA_BASE_URL`.

Project current V1 data into the V2 graph tables:

```bash
make backend-graph-project-dev
```

This starts the local PostgreSQL container if needed, applies migrations through the backend storage layer, runs `GraphProjectionService::project_from_v1()` against the current dev database and prints a JSON projection summary. It leaves PostgreSQL running for the active development session and does not connect to Gmail, iCloud or IMAP provider mailboxes.

Export a redacted iCloud IMAP fixture sample from the latest read-only messages:

```bash
HERMES_IMAP_FIXTURE_USERNAME=<icloud-email> \
HERMES_IMAP_FIXTURE_PASSWORD=<app-password> \
HERMES_IMAP_FIXTURE_MAX_MESSAGES=10 \
HERMES_IMAP_FIXTURE_OUTPUT=tmp/email-fixtures/icloud-inbox-redacted.json \
make backend-email-fixture-export-icloud-dev
```

The exporter uses `EXAMINE`, `UID SEARCH` and `BODY.PEEK[]` through the same IMAP network client as provider sync. It writes redacted fixture JSON by default, prints only a non-secret summary, and does not import into PostgreSQL. The default output path is under `tmp/`, which is ignored by git.

Import a redacted fixture JSON sample into the local development database:

```bash
make backend-email-fixture-import-dev
```

Project that fixture through canonical messages, contacts and V2 graph projection:

```bash
make backend-email-fixture-project-dev
```

Both commands default to `tmp/email-fixtures/icloud-inbox-redacted.json`, create or update the local `dev-icloud-fixture` provider account, print JSON summaries and leave PostgreSQL running for the active development session. Override path and account metadata with `HERMES_EMAIL_FIXTURE_PATH`, `HERMES_EMAIL_FIXTURE_ACCOUNT_ID`, `HERMES_EMAIL_FIXTURE_DISPLAY_NAME`, `HERMES_EMAIL_FIXTURE_EXTERNAL_ACCOUNT_ID`, `HERMES_EMAIL_FIXTURE_IMPORT_BATCH_ID` and `HERMES_EMAIL_FIXTURE_PROVIDER`.

Fetch iCloud/raw IMAP mail into the persistent local development cache:

```bash
HERMES_EMAIL_SYNC_USERNAME=<imap-login> \
HERMES_EMAIL_SYNC_PASSWORD=<app-password> \
HERMES_EMAIL_SYNC_PROVIDER=icloud \
HERMES_EMAIL_SYNC_MAX_MESSAGES=25 \
make backend-email-sync-cache-dev
```

The command uses read-only IMAP, writes raw `.eml` blobs under `docker/data/mail/`, stores only metadata and blob references in PostgreSQL, and projects canonical messages plus contacts for the UI. It does not support Gmail OAuth yet; Gmail cache sync should use the same pipeline after account setup exposes refreshed access tokens to the dev command.

Direct Cargo commands:

```sh
cargo run --manifest-path backend/Cargo.toml
cargo run --manifest-path backend/Cargo.toml --bin hermes-graph-project
cargo run --manifest-path backend/Cargo.toml --bin hermes-email-fixture-export
cargo run --manifest-path backend/Cargo.toml --bin hermes-email-fixture-dev
cargo run --manifest-path backend/Cargo.toml --bin hermes-email-sync-dev
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

For the normal full-stack development loop, use `make dev` from the repository root. It starts PostgreSQL, runs this backend with auto-restart on Rust/TOML/SQL changes, and starts the SvelteKit frontend with Vite HMR. Backend auto-restart requires either `watchexec` or `cargo-watch`.

## Environment

Supported environment variables:

- `HERMES_HTTP_ADDR` - backend bind address, defaults to `127.0.0.1:8080`.
- `DATABASE_URL` - optional PostgreSQL URL. The current health endpoint does not require a database connection.
- `HERMES_LOCAL_API_TOKEN` - temporary local capability token required for local event API endpoints.
- `HERMES_LOCAL_WRITE_TOKEN` - legacy fallback for `HERMES_LOCAL_API_TOKEN` during transition from ADR-0037.
- `HERMES_SECRET_VAULT_PATH` - local encrypted vault file used by account setup.
- `HERMES_SECRET_VAULT_KEY` - local encrypted vault master key; do not commit or log this value.
- `HERMES_OLLAMA_BASE_URL` - Ollama runtime URL, defaults to `http://127.0.0.1:11434`.
- `HERMES_OLLAMA_CHAT_MODEL` - Ollama chat model, defaults to `qwen3:4b`.
- `HERMES_OLLAMA_EMBED_MODEL` - Ollama embedding model, defaults to `qwen3-embedding:4b`.
- `HERMES_OLLAMA_TIMEOUT_SECONDS` - Ollama request timeout, defaults to `120`.

## Endpoints

- `GET /healthz` - returns backend health status and service name.
- `GET /readyz` - returns readiness status; it is `503` when PostgreSQL is not configured, unavailable or missing required SQLx migrations.
- `GET /api/v1/status` - returns enabled V1 surfaces. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `GET /api/v2/graph/summary` - returns graph node, edge and evidence summary counts. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `GET /api/v2/graph/search` - searches graph nodes by `q` with optional `limit`. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `GET /api/v2/graph/neighborhood` - returns the depth-1 graph neighborhood for `node_id`, including neighboring nodes, edges and evidence. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `POST /api/v1/email-accounts/gmail/oauth/start` - starts Gmail OAuth account setup and returns a PKCE authorization URL. Requires local API headers and encrypted vault config.
- `GET /api/v1/email-accounts/gmail/oauth/callback` - displays OAuth callback code/state for the desktop setup flow.
- `POST /api/v1/email-accounts/gmail/oauth/complete` - exchanges a Gmail authorization code, stores the encrypted token bundle and creates provider account bindings. Requires local API headers, PostgreSQL and encrypted vault config.
- `POST /api/v1/email-accounts/imap` - creates iCloud/raw IMAP account metadata and stores the password/app-password in the encrypted vault. Requires local API headers, PostgreSQL and encrypted vault config.
- `POST /api/events` - appends a canonical event through the application/API boundary. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `GET /api/events/{event_id}` - loads a canonical event by ID. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.
- `GET /api/audit/events` - returns event API audit records. Supports `target_id`, `actor_id`, `after_audit_id` and `limit` query parameters. Requires `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.

Authorized event API calls are recorded in `api_audit_log` with `actor_kind` and `actor_id`. The API token value is never stored.

## V2 Workflow APIs

Available endpoints below require both `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.

- `GET /api/v2/projects` - lists local project records with derived stats.
- `GET /api/v2/projects/{project_id}` - returns project detail, timeline, messages, documents and people.
- `GET /api/v2/projects/{project_id}/link-candidates` - returns safe project message/document link candidates.
- `PUT /api/v2/projects/{project_id}/link-reviews` - records project link review state as a canonical event.
- `GET /api/v2/task-candidates` - lists source-backed task candidates.
- `PUT /api/v2/task-candidates/{task_candidate_id}/review` - records task candidate review state as a canonical event.
- `GET /api/v2/tasks` - lists active local tasks created from confirmed candidates.
- `GET /api/v2/identity-candidates` - lists contact identity candidates.
- `PUT /api/v2/identity-candidates/{identity_candidate_id}/review` - records identity candidate review state as a canonical event.
- `GET /api/v2/contacts/{contact_id}/identity` - returns confirmed identity links for one contact.
- `GET /api/v2/documents/{document_id}/processing` - returns processing jobs and artifacts for one document.
- `GET /api/v2/document-processing/jobs` - lists recent document processing jobs.
- `POST /api/v2/document-processing/jobs/{job_id}/retry` - requeues a failed processing job through a canonical retry event. The JSON body requires `command_id`; the response returns `job_id`, `status` and `event_id`.

## V3 AI APIs

Available endpoints below require both `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>` and `X-Hermes-Actor-Id`.

- `GET /api/v3/ai/status` - returns Ollama runtime/model availability.
- `GET /api/v3/agents` - lists V3 agents `HESTIA`, `HERMES`, `MNEMOSYNE` and `ATHENA`.
- `GET /api/v3/ai/runs` - lists persisted local AI runs.
- `GET /api/v3/ai/runs/{run_id}` - returns one persisted AI run.
- `POST /api/v3/ai/answers` - creates a source-backed answer with citations.
- `POST /api/v3/ai/task-candidates/refresh` - refreshes AI-suggested task candidates only.
- `POST /api/v3/ai/meeting-prep` - creates a source-backed local meeting prep packet without calendar/provider writes.

## Migrations

Backend startup applies local PostgreSQL migrations when `DATABASE_URL` is configured.
Readiness checks verify that the embedded SQLx migration ledger has the expected successful migration count and latest version.

Current schema:

- `event_log` - append-only canonical event log with JSONB envelope fields, replay ordering, idempotent source index and mutation-prevention triggers.
- `projection_cursors` - monotonic per-projection replay cursor positions.
- `api_audit_log` - append-only operational audit records for local event API access attempts, including non-secret local actor IDs.
- `secret_references` - non-secret metadata pointers to external secret stores; secret values are never stored in PostgreSQL.
- encrypted vault file - local encrypted credential values for provider account setup.
- `communication_provider_accounts` - non-secret email provider account metadata for `gmail`, `icloud` and `imap`.
- `communication_raw_records` - append-only raw provider records with idempotent provider identity, source fingerprints, import batches and provenance.
- `communication_ingestion_checkpoints` - per-account provider stream checkpoints for retryable ingestion.
- `communication_provider_account_secret_refs` - maps provider accounts to secret references by credential purpose.
- `communication_messages` - canonical message projection records derived from raw communication records.
- `contacts` - contact projection records keyed by unique email address.
- `documents` - imported document records with source fingerprints and extracted text.
- `graph_nodes` - rebuildable graph projection nodes derived from contacts, messages and documents.
- `graph_edges` - rebuildable graph projection relationships with confidence and review state.
- `graph_evidence` - rebuildable graph projection evidence records that preserve edge provenance.
- `ai_agent_runs` - persisted local AI run provenance, answer/citation payloads and timings.
- `semantic_embeddings` - rebuildable pgvector `halfvec(2560)` embeddings for local source retrieval.

Relevant design documents:

- [Architecture Overview](../docs/architecture/architecture-overview.md)
- [Event Model](../docs/architecture/event-model.md)
- [Storage Architecture](../docs/architecture/storage-architecture.md)
- [ADR Index](../docs/adr/README.md)
