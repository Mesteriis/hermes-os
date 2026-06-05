# Document Processing Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add asynchronous document processing jobs and artifacts so document extraction state is visible, replayable and decoupled from import.

**Architecture:** Keep `documents` as source metadata and add `document_processing_jobs` plus `document_artifacts` as derived processing state. `DocumentProcessingStore` owns enqueue/read/write transitions, a bounded runner processes queued jobs, and protected read APIs expose status to the desktop UI. The first processor handles Markdown/plain text locally and marks PDF OCR as skipped unless a real OCR backend is added by a later ADR-backed slice.

**Tech Stack:** Rust 1.85/edition 2024, Axum, SQLx/PostgreSQL migrations, existing document import store, SvelteKit 2, Svelte 5 runes, TypeScript, pnpm, Make.

---

## Source Spec

- `docs/superpowers/specs/2026-06-05-v2-memory-workflow-completion-design.md`

## Relevant ADRs

- `docs/adr/ADR-0001-event-sourcing-as-system-spine.md`
- `docs/adr/ADR-0017-document-processing-pipeline.md`
- `docs/adr/ADR-0023-rebuildable-projections.md`
- `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md`
- `docs/adr/ADR-0038-local-event-api-capability-token.md`
- `docs/adr/ADR-0040-local-api-actor-identity.md`
- `docs/adr/ADR-0046-persistent-dev-mail-cache-and-blob-storage.md`

## File Map

- Create: `backend/migrations/0017_create_document_processing.sql`
- Create: `backend/src/document_processing.rs`
- Modify: `backend/src/documents.rs`
- Modify: `backend/src/lib.rs`
- Modify: `backend/Cargo.toml`
- Create: `backend/src/bin/hermes_document_process.rs`
- Create: `backend/tests/document_processing.rs`
- Create: `backend/tests/document_processing_api.rs`
- Modify: `Makefile`
- Modify: `backend/README.md`
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

## Data Contracts

Backend domain values:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentProcessingStep {
    ExtractText,
    Ocr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentProcessingStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentArtifactKind {
    ExtractedText,
    OcrText,
}
```

Frontend status values:

```ts
export type DocumentProcessingStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'skipped';
export type DocumentProcessingStep = 'extract_text' | 'ocr';
export type DocumentArtifactKind = 'extracted_text' | 'ocr_text';
```

## Assumptions

Assumption: The first pipeline does not add an external OCR dependency.
Reason: The repository has no OCR dependency or runtime configuration yet, and `ADR-0017` requires a pipeline boundary before expensive processors.
Risk: PDF OCR is visible as `skipped`, not complete text extraction.

Assumption: Retry is validated through CLI/Make first, not a write API.
Reason: The approved spec allows write/retry commands only when they use an existing command/audit boundary; a CLI runner is enough to establish processing state.
Risk: Users can inspect status in UI but cannot retry from UI in this slice.

---

## Task 1: Schema

**Files:**
- Create: `backend/migrations/0017_create_document_processing.sql`

- [ ] **Step 1: Add migration**

Create `backend/migrations/0017_create_document_processing.sql`:

```sql
CREATE TABLE IF NOT EXISTS document_processing_jobs (
    job_id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    step TEXT NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_error_summary TEXT,
    queued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT document_processing_step_check
        CHECK (step IN ('extract_text', 'ocr')),
    CONSTRAINT document_processing_status_check
        CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'skipped')),
    CONSTRAINT document_processing_attempts_check
        CHECK (attempts >= 0 AND max_attempts >= 1 AND attempts <= max_attempts),
    CONSTRAINT document_processing_job_id_not_empty
        CHECK (length(trim(job_id)) > 0),
    CONSTRAINT document_processing_document_id_not_empty
        CHECK (length(trim(document_id)) > 0),
    CONSTRAINT document_processing_document_step_unique
        UNIQUE (document_id, step)
);

CREATE INDEX IF NOT EXISTS document_processing_status_idx
    ON document_processing_jobs (status, queued_at);

CREATE INDEX IF NOT EXISTS document_processing_document_idx
    ON document_processing_jobs (document_id);

CREATE TABLE IF NOT EXISTS document_artifacts (
    artifact_id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    job_id TEXT NOT NULL REFERENCES document_processing_jobs(job_id) ON DELETE CASCADE,
    artifact_kind TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    text_content TEXT,
    storage_kind TEXT,
    storage_path TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT document_artifact_kind_check
        CHECK (artifact_kind IN ('extracted_text', 'ocr_text')),
    CONSTRAINT document_artifact_id_not_empty
        CHECK (length(trim(artifact_id)) > 0),
    CONSTRAINT document_artifact_sha_not_empty
        CHECK (length(trim(content_sha256)) > 0),
    CONSTRAINT document_artifact_text_or_storage
        CHECK (text_content IS NOT NULL OR storage_path IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS document_artifacts_document_kind_idx
    ON document_artifacts (document_id, artifact_kind);

CREATE INDEX IF NOT EXISTS document_artifacts_job_idx
    ON document_artifacts (job_id);
```

- [ ] **Step 2: Run migration test command to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test document_processing -- --nocapture --test-threads=1
```

Expected: FAIL because `backend/tests/document_processing.rs` does not exist yet.

- [ ] **Step 3: Commit schema**

Run:

```sh
git add backend/migrations/0017_create_document_processing.sql
git commit -m "feat: add document processing schema"
```

## Task 2: Store And Runner

**Files:**
- Create: `backend/src/document_processing.rs`
- Modify: `backend/src/documents.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/document_processing.rs`

- [ ] **Step 1: Write failing store tests**

Create `backend/tests/document_processing.rs` with tests named:

```rust
#[tokio::test]
async fn document_processing_enqueue_creates_extract_and_ocr_jobs_against_postgres() {}

#[tokio::test]
async fn document_processing_runner_succeeds_markdown_extraction_against_postgres() {}

#[tokio::test]
async fn document_processing_runner_skips_pdf_ocr_without_backend_against_postgres() {}

#[tokio::test]
async fn document_processing_runner_records_safe_failure_summary_against_postgres() {}
```

Seed documents with `DocumentImportStore`. Assert artifacts contain extracted text for Markdown and that PDF OCR jobs finish as `skipped`.

- [ ] **Step 2: Run tests to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test document_processing -- --nocapture --test-threads=1
```

Expected: FAIL with unresolved import `hermes_hub_backend::document_processing`.

- [ ] **Step 3: Add module export**

Add this line near existing public modules in `backend/src/lib.rs`:

```rust
pub mod document_processing;
```

- [ ] **Step 4: Create document processing store**

Create `backend/src/document_processing.rs` with these public items:

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DocumentProcessingStep {
    ExtractText,
    Ocr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DocumentProcessingStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DocumentArtifactKind {
    ExtractedText,
    OcrText,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentProcessingJob {
    pub job_id: String,
    pub document_id: String,
    pub step: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error_summary: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentArtifact {
    pub artifact_id: String,
    pub document_id: String,
    pub job_id: String,
    pub artifact_kind: String,
    pub content_sha256: String,
    pub text_content: Option<String>,
    pub storage_kind: Option<String>,
    pub storage_path: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct DocumentProcessingStore {
    pool: PgPool,
}

impl DocumentProcessingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

- [ ] **Step 5: Implement enqueue**

Add:

```rust
pub async fn enqueue_document(&self, document_id: &str) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError>
```

Behavior:

- validate document exists;
- create `extract_text` job for every document;
- create `ocr` job for `pdf` documents;
- use deterministic job IDs `document_processing:v1:{document_id}:{step}`;
- preserve existing terminal job status and do not reset succeeded/skipped jobs.

- [ ] **Step 6: Wire enqueue into document import**

In `backend/src/documents.rs`, add a method that can be called by tests and later import paths:

```rust
pub async fn import_document_and_enqueue_processing(
    &self,
    document: &NewDocumentImport,
    processing: &crate::document_processing::DocumentProcessingStore,
) -> Result<ImportedDocument, DocumentImportError>
```

It must call existing `import_document`, then `processing.enqueue_document(&imported.document_id)`. Add a `DocumentImportError::Processing` variant using `#[from] crate::document_processing::DocumentProcessingError`.

- [ ] **Step 7: Implement bounded runner**

Add:

```rust
pub async fn run_queued_batch(&self, limit: i64) -> Result<DocumentProcessingRunReport, DocumentProcessingError>
```

Behavior:

- clamp limit to `1..=100`;
- select queued jobs ordered by `queued_at`;
- mark each job `running` and increment attempts;
- `extract_text` for Markdown writes an `extracted_text` artifact with `documents.extracted_text`;
- `extract_text` for PDF writes a skipped job if no extracted text exists;
- `ocr` writes `skipped` with summary `ocr backend is not configured`;
- failures store `last_error_summary` truncated to 240 characters and do not include document text.

- [ ] **Step 8: Add read methods**

Add:

```rust
pub async fn document_processing_status(&self, document_id: &str) -> Result<DocumentProcessingDetail, DocumentProcessingError>
pub async fn list_jobs(&self, limit: Option<i64>) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError>
```

`DocumentProcessingDetail` must include `jobs` and `artifacts`.

- [ ] **Step 9: Run store tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test document_processing -- --nocapture --test-threads=1
```

Expected: PASS or live-test skip when `HERMES_TEST_DATABASE_URL` is unset.

- [ ] **Step 10: Commit store**

Run:

```sh
git add backend/src/document_processing.rs backend/src/documents.rs backend/src/lib.rs backend/tests/document_processing.rs
git commit -m "feat: add document processing runner"
```

## Task 3: CLI And Make Target

**Files:**
- Create: `backend/src/bin/hermes_document_process.rs`
- Modify: `backend/Cargo.toml`
- Modify: `Makefile`
- Modify: `backend/README.md`

- [ ] **Step 1: Add CLI binary**

Create `backend/src/bin/hermes_document_process.rs`:

```rust
use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::document_processing::DocumentProcessingStore;
use hermes_hub_backend::storage::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    hermes_hub_backend::init_tracing();
    let config = AppConfig::from_env()?;
    let database = Database::connect(config.database_url()).await?;
    let pool = database.pool().ok_or("DATABASE_URL is required")?.clone();
    let store = DocumentProcessingStore::new(pool);
    let report = store.run_queued_batch(100).await?;
    println!(
        "document processing: processed={} succeeded={} skipped={} failed={}",
        report.processed, report.succeeded, report.skipped, report.failed
    );
    Ok(())
}
```

- [ ] **Step 2: Register binary**

In `backend/Cargo.toml`, add:

```toml
[[bin]]
name = "hermes-document-process"
path = "src/bin/hermes_document_process.rs"
```

- [ ] **Step 3: Add Make target**

In `Makefile`, add `backend-document-processing-dev` to `.PHONY`, then add:

```make
backend-document-processing-dev: docker-env
	@set -a; . docker/.env; set +a; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo run --manifest-path $(BACKEND_MANIFEST) --bin hermes-document-process
```

- [ ] **Step 4: Document command**

In `backend/README.md`, add a short entry:

````markdown
### Document Processing

Run queued local document processing jobs against development PostgreSQL:

```sh
make backend-document-processing-dev
```

The first processor extracts stored Markdown/plain text and marks PDF OCR as skipped unless a real OCR backend is added later.
````

- [ ] **Step 5: Validate CLI compile**

Run:

```sh
cargo check --manifest-path backend/Cargo.toml --bin hermes-document-process
```

Expected: PASS.

- [ ] **Step 6: Commit CLI**

Run:

```sh
git add backend/Cargo.toml backend/src/bin/hermes_document_process.rs Makefile backend/README.md
git commit -m "feat: add document processing dev runner"
```

## Task 4: Protected Read API

**Files:**
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/document_processing_api.rs`

- [ ] **Step 1: Write failing API tests**

Create `backend/tests/document_processing_api.rs` with tests named:

```rust
#[tokio::test]
async fn document_processing_status_rejects_missing_local_api_token() {}

#[tokio::test]
async fn document_processing_status_returns_jobs_and_artifacts() {}

#[tokio::test]
async fn document_processing_jobs_returns_recent_jobs() {}

#[tokio::test]
async fn document_processing_status_rejects_missing_document() {}
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test document_processing_api -- --nocapture --test-threads=1
```

Expected: FAIL because the routes are not registered.

- [ ] **Step 3: Add routes**

In `build_router_with_database`, add:

```rust
.route(
    "/api/v2/documents/{document_id}/processing",
    get(get_document_processing_status),
)
.route(
    "/api/v2/document-processing/jobs",
    get(get_document_processing_jobs),
)
```

- [ ] **Step 4: Add handlers**

Add:

```rust
async fn get_document_processing_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> Result<Json<crate::document_processing::DocumentProcessingDetail>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let detail = document_processing_store(&state)?
        .document_processing_status(&document_id)
        .await?;
    Ok(Json(detail))
}

async fn get_document_processing_jobs(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<DocumentProcessingJobListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_limit_query(raw_query.as_deref())?;
    let items = document_processing_store(&state)?.list_jobs(query.limit).await?;
    Ok(Json(DocumentProcessingJobListResponse { items }))
}
```

Add helper `document_processing_store(&AppState) -> Result<DocumentProcessingStore, ApiError>`.

- [ ] **Step 5: Map errors**

Extend `ApiError` so:

- missing document maps to `404 document_not_found`;
- store SQL errors map to `500 document_processing_store_error`.

- [ ] **Step 6: Run API tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test document_processing_api -- --nocapture --test-threads=1
```

Expected: PASS or live-test skip when `HERMES_TEST_DATABASE_URL` is unset.

- [ ] **Step 7: Commit API**

Run:

```sh
git add backend/src/lib.rs backend/tests/document_processing_api.rs
git commit -m "feat: expose document processing status APIs"
```

## Task 5: Frontend Document Status

**Files:**
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Add frontend API types**

In `frontend/src/lib/api.ts`, add:

```ts
export type DocumentProcessingStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'skipped';
export type DocumentProcessingStep = 'extract_text' | 'ocr';
export type DocumentArtifactKind = 'extracted_text' | 'ocr_text';

export type DocumentProcessingJob = {
	job_id: string;
	document_id: string;
	step: DocumentProcessingStep;
	status: DocumentProcessingStatus;
	attempts: number;
	max_attempts: number;
	last_error_summary: string | null;
	queued_at: string;
	started_at: string | null;
	finished_at: string | null;
	updated_at: string;
};

export type DocumentArtifact = {
	artifact_id: string;
	document_id: string;
	job_id: string;
	artifact_kind: DocumentArtifactKind;
	content_sha256: string;
	text_content: string | null;
	storage_kind: string | null;
	storage_path: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
};

export type DocumentProcessingDetail = {
	document_id: string;
	jobs: DocumentProcessingJob[];
	artifacts: DocumentArtifact[];
};

export type DocumentProcessingJobListResponse = {
	items: DocumentProcessingJob[];
};
```

- [ ] **Step 2: Add frontend API functions**

Add:

```ts
export async function fetchDocumentProcessingJobs(apiBaseUrl: string, token: string, actorId: string, limit = 50) {
	return fetchJson<DocumentProcessingJobListResponse>(
		`${apiBaseUrl}/api/v2/document-processing/jobs?limit=${limit}`,
		token,
		actorId
	);
}

export async function fetchDocumentProcessingDetail(
	apiBaseUrl: string,
	token: string,
	actorId: string,
	documentId: string
) {
	return fetchJson<DocumentProcessingDetail>(
		`${apiBaseUrl}/api/v2/documents/${encodeURIComponent(documentId)}/processing`,
		token,
		actorId
	);
}
```

- [ ] **Step 3: Add Svelte state**

In `frontend/src/routes/+page.svelte`, add:

```svelte
let documentProcessingJobs = $state<DocumentProcessingJob[]>([]);
let documentProcessingError = $state('');
let isDocumentProcessingLoading = $state(false);
```

- [ ] **Step 4: Load processing jobs**

Add:

```svelte
async function loadDocumentProcessingJobs() {
	isDocumentProcessingLoading = true;
	try {
		const response = await fetchDocumentProcessingJobs(apiBaseUrl, apiToken, actorId, 50);
		documentProcessingJobs = response.items;
		documentProcessingError = '';
	} catch (error) {
		documentProcessingError =
			error instanceof Error ? error.message : 'Unknown document processing error';
	} finally {
		isDocumentProcessingLoading = false;
	}
}
```

Call it from `onMount`.

- [ ] **Step 5: Render document processing status**

In the Documents view, add API-backed status sections:

- recent processing jobs from `documentProcessingJobs`;
- status badge per job;
- failed job error summary;
- skipped OCR summary;
- empty/loading/error states.

Do not add retry buttons because this slice uses the CLI runner, not a protected write API.

- [ ] **Step 6: Run frontend validation**

Run:

```sh
pnpm --dir frontend check
pnpm --dir frontend build
```

Expected: PASS.

- [ ] **Step 7: Commit frontend**

Run:

```sh
git add frontend/src/lib/api.ts frontend/src/routes/+page.svelte
git commit -m "feat: render document processing status"
```

## Task 6: Final Validation

**Files:**
- No new files.

- [ ] **Step 1: Run targeted backend tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test document_processing --test document_processing_api -- --nocapture --test-threads=1
```

Expected: PASS or live-test skip when `HERMES_TEST_DATABASE_URL` is unset.

- [ ] **Step 2: Run backend validation**

Run:

```sh
make backend-validate
```

Expected: PASS.

- [ ] **Step 3: Run frontend validation**

Run:

```sh
make frontend-check
make frontend-build
```

Expected: PASS.

- [ ] **Step 4: Run full validation**

Run:

```sh
make validate
```

Expected: PASS.

## Self-Review Checklist

- [ ] Document import remains decoupled from processing.
- [ ] Processing jobs have explicit terminal states.
- [ ] Markdown/plain text extraction creates an artifact.
- [ ] PDF OCR is skipped without claiming clean OCR output.
- [ ] Failure summaries do not store private document text.
- [ ] UI has no retry control unless a backend write command is added.

## Closure Status

Closed on 2026-06-05 as part of the V2 workflow slices.

Implemented:
- document processing jobs and artifacts schema;
- bounded local runner for queued extraction/OCR jobs;
- Markdown/plain-text extraction and skipped OCR behavior without a fake OCR backend;
- protected local read APIs for document processing detail and recent jobs;
- `hermes-document-process` CLI plus `make backend-document-processing-dev`;
- desktop processing status surface without retry controls.

Validated:
- `cargo test --manifest-path backend/Cargo.toml --test document_processing --test document_processing_api -- --nocapture`;
- `make backend-validate`;
- `pnpm --dir frontend check`;
- `pnpm --dir frontend build`.

Not run:
- full `make validate`, because that also runs Docker-backed smoke checks outside this slice;
- browser screenshot smoke, because Playwright was not available in `frontend/node_modules` in this session.
