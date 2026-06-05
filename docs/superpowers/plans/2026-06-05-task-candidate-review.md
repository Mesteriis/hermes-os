# Task Candidate Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add source-backed task candidates from messages and documents with explicit review before an active local task exists.

**Architecture:** Keep generated task candidates as deterministic PostgreSQL read-model rows, and keep user decisions as canonical `task_candidate.review_state_changed` events. `TaskCandidateStore` owns generation, review commands and active-task reads; Axum exposes protected local APIs; the Svelte Tasks tab reads backend data instead of static literals.

**Tech Stack:** Rust 1.85/edition 2024, Axum, SQLx/PostgreSQL migrations, existing event log/audit patterns, SvelteKit 2, Svelte 5 runes, TypeScript, pnpm, Make.

---

## Source Spec

- `docs/superpowers/specs/2026-06-05-v2-memory-workflow-completion-design.md`

## Relevant ADRs

- `docs/adr/ADR-0001-event-sourcing-as-system-spine.md`
- `docs/adr/ADR-0015-command-query-separation.md`
- `docs/adr/ADR-0020-task-candidate-lifecycle.md`
- `docs/adr/ADR-0023-rebuildable-projections.md`
- `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md`
- `docs/adr/ADR-0038-local-event-api-capability-token.md`
- `docs/adr/ADR-0040-local-api-actor-identity.md`

## File Map

- Create: `backend/migrations/0015_create_task_candidates.sql`
- Create: `backend/src/task_candidates.rs`
- Modify: `backend/src/audit.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/task_candidates.rs`
- Create: `backend/tests/task_candidates_api.rs`
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

## Data Contracts

Backend domain values:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateSourceKind {
    Message,
    Document,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

pub struct TaskCandidateReviewCommand {
    pub command_id: String,
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub actor_id: String,
}
```

Frontend request shape:

```ts
export type TaskCandidateReviewRequest = {
	command_id: string;
	review_state: 'suggested' | 'user_confirmed' | 'user_rejected';
};
```

## Assumptions

Assumption: First-slice task extraction is deterministic and rule-based.
Reason: `ADR-0020` allows AI candidates later, but the repository does not yet have an Ollama/provider boundary implementation.
Risk: Rule-based candidates will miss some tasks; review UX and extractor provenance leave room for a later AI-backed extractor.

Assumption: Active tasks are local records only.
Reason: The repository has no task provider adapter or calendar write boundary.
Risk: Users can review and view active tasks locally, but no external task system is updated.

---

## Task 1: Schema

**Files:**
- Create: `backend/migrations/0015_create_task_candidates.sql`

- [ ] **Step 1: Add migration**

Create `backend/migrations/0015_create_task_candidates.sql` with this SQL:

```sql
CREATE TABLE IF NOT EXISTS task_candidates (
    task_candidate_id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    due_text TEXT,
    assignee_label TEXT,
    confidence DOUBLE PRECISION NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    evidence_excerpt TEXT NOT NULL,
    event_id TEXT,
    actor_id TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_candidates_source_kind_check
        CHECK (source_kind IN ('message', 'document')),
    CONSTRAINT task_candidates_review_state_check
        CHECK (review_state IN ('suggested', 'user_confirmed', 'user_rejected')),
    CONSTRAINT task_candidates_confidence_check
        CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT task_candidates_id_not_empty
        CHECK (length(trim(task_candidate_id)) > 0),
    CONSTRAINT task_candidates_source_id_not_empty
        CHECK (length(trim(source_id)) > 0),
    CONSTRAINT task_candidates_title_not_empty
        CHECK (length(trim(title)) > 0),
    CONSTRAINT task_candidates_evidence_excerpt_not_empty
        CHECK (length(trim(evidence_excerpt)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS task_candidates_source_title_idx
    ON task_candidates (source_kind, source_id, lower(title));

CREATE INDEX IF NOT EXISTS task_candidates_review_state_idx
    ON task_candidates (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS task_candidates_project_idx
    ON task_candidates (project_id);

CREATE TABLE IF NOT EXISTS tasks (
    task_id TEXT PRIMARY KEY,
    task_candidate_id TEXT NOT NULL UNIQUE
        REFERENCES task_candidates(task_candidate_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_from_event_id TEXT NOT NULL,
    created_by_actor_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT tasks_source_kind_check
        CHECK (source_kind IN ('message', 'document')),
    CONSTRAINT tasks_status_check
        CHECK (status IN ('active')),
    CONSTRAINT tasks_id_not_empty CHECK (length(trim(task_id)) > 0),
    CONSTRAINT tasks_title_not_empty CHECK (length(trim(title)) > 0)
);

CREATE INDEX IF NOT EXISTS tasks_project_idx ON tasks (project_id);
CREATE INDEX IF NOT EXISTS tasks_source_idx ON tasks (source_kind, source_id);
```

- [ ] **Step 2: Run migration syntax check through backend tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test task_candidates -- --nocapture --test-threads=1
```

Expected: FAIL because `backend/tests/task_candidates.rs` does not exist yet.

- [ ] **Step 3: Commit schema**

Run:

```sh
git add backend/migrations/0015_create_task_candidates.sql
git commit -m "feat: add task candidate schema"
```

## Task 2: Store And Deterministic Candidate Generation

**Files:**
- Create: `backend/src/task_candidates.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/task_candidates.rs`

- [ ] **Step 1: Write failing store tests**

Create `backend/tests/task_candidates.rs` with live PostgreSQL tests named:

```rust
#[tokio::test]
async fn task_candidate_refresh_creates_message_and_document_candidates_against_postgres() {}

#[tokio::test]
async fn task_candidate_review_confirm_creates_active_task_against_postgres() {}

#[tokio::test]
async fn task_candidate_review_reset_removes_active_task_against_postgres() {}

#[tokio::test]
async fn task_candidate_review_event_rebuilds_state_against_postgres() {}
```

The tests must follow the existing skip pattern:

```rust
let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
    eprintln!("skipping live task candidate test: HERMES_TEST_DATABASE_URL is not set");
    return;
};
```

Seed messages through `CommunicationIngestionStore` plus `MessageProjectionStore`, and seed documents through `DocumentImportStore`. Use subjects/text containing `Action:` and `Please` so deterministic rules produce candidates.

- [ ] **Step 2: Run tests to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test task_candidates -- --nocapture --test-threads=1
```

Expected: FAIL with unresolved import `hermes_hub_backend::task_candidates`.

- [ ] **Step 3: Add module export**

Add this line near the existing public modules in `backend/src/lib.rs`:

```rust
pub mod task_candidates;
```

- [ ] **Step 4: Create task candidate store**

Create `backend/src/task_candidates.rs` with these public items:

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::event_log::{EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope};

const TASK_CANDIDATE_REVIEW_EVENT_TYPE: &str = "task_candidate.review_state_changed";
const TASK_CANDIDATE_REVIEW_SOURCE_KIND: &str = "task_candidate_review";
const TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER: &str = "local_api";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum TaskCandidateSourceKind {
    Message,
    Document,
}

impl TaskCandidateSourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum TaskCandidateReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl TaskCandidateReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCandidateReviewCommand {
    pub command_id: String,
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskCandidate {
    pub task_candidate_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub due_text: Option<String>,
    pub assignee_label: Option<String>,
    pub confidence: f64,
    pub review_state: String,
    pub evidence_excerpt: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActiveTask {
    pub task_id: String,
    pub task_candidate_id: String,
    pub title: String,
    pub source_kind: String,
    pub source_id: String,
    pub project_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskCandidateStore {
    pool: PgPool,
}

impl TaskCandidateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_deterministic_candidates(&self, limit: i64) -> Result<usize, TaskCandidateError> {
        let limit = validate_limit(limit)?;
        let message_count = self.refresh_message_candidates(limit).await?;
        let document_count = self.refresh_document_candidates(limit).await?;
        Ok(message_count + document_count)
    }
}
```

Implement the private functions with these exact behavior rules:

- `refresh_message_candidates` scans `communication_messages.subject` and `communication_messages.body_text`.
- `refresh_document_candidates` scans `documents.title` and `documents.extracted_text`.
- candidates are generated when normalized text contains `action:`, `please `, `follow up`, or `next step`.
- `task_candidate_id` is deterministic: `task_candidate:v1:{source_kind}:{source_id}:{fnv1a64(title)}`.
- generated candidates never overwrite `user_confirmed` or `user_rejected` review state.
- evidence excerpts use `text_preview(value, 180)`.

- [ ] **Step 5: Add review command behavior**

In `TaskCandidateStore`, add:

```rust
pub async fn set_review_state(
    &self,
    command: &TaskCandidateReviewCommand,
) -> Result<TaskCandidateReviewResult, TaskCandidateError>
```

It must:

- validate non-empty `command_id`, `task_candidate_id` and `actor_id`;
- append event ID `task_candidate_review:{command_id}`;
- update `task_candidates.review_state`;
- create or upsert `tasks` when state is `user_confirmed`;
- delete the matching `tasks` row when state is `suggested` or `user_rejected`;
- execute event append and table updates in one SQL transaction.

- [ ] **Step 6: Add replay behavior**

In `TaskCandidateStore`, add:

```rust
pub async fn apply_review_event(&self, event: &EventEnvelope) -> Result<(), TaskCandidateError>
```

It must parse payload fields `task_candidate_id` and `review_state`, read `actor.actor_id`, and apply the same table transition as the command path without appending a second event.

- [ ] **Step 7: Add read methods**

Add:

```rust
pub async fn list_candidates(&self, limit: Option<i64>) -> Result<Vec<TaskCandidate>, TaskCandidateError>
pub async fn list_tasks(&self, limit: Option<i64>) -> Result<Vec<ActiveTask>, TaskCandidateError>
```

Both methods must clamp limits to `1..=100` and order by newest `updated_at`.

- [ ] **Step 8: Run store tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test task_candidates -- --nocapture --test-threads=1
```

Expected: PASS when `HERMES_TEST_DATABASE_URL` is set, or skipped with the test skip message when it is not set.

- [ ] **Step 9: Commit store**

Run:

```sh
git add backend/src/task_candidates.rs backend/src/lib.rs backend/tests/task_candidates.rs
git commit -m "feat: add task candidate review store"
```

## Task 3: Protected API

**Files:**
- Modify: `backend/src/audit.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/task_candidates_api.rs`

- [ ] **Step 1: Write failing API tests**

Create `backend/tests/task_candidates_api.rs` with tests named:

```rust
#[tokio::test]
async fn task_candidates_reject_missing_local_api_token() {}

#[tokio::test]
async fn task_candidates_returns_safe_candidate_payload() {}

#[tokio::test]
async fn put_task_candidate_review_requires_actor_and_confirms_task() {}

#[tokio::test]
async fn put_task_candidate_review_rejects_missing_candidate() {}
```

Expected API error bodies:

```json
{"error":"invalid_api_token","message":"missing or invalid bearer token"}
{"error":"missing_actor_id","message":"missing X-Hermes-Actor-Id header"}
{"error":"task_candidate_not_found","message":"task candidate was not found"}
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test task_candidates_api -- --nocapture --test-threads=1
```

Expected: FAIL because `/api/v2/task-candidates` is not routed.

- [ ] **Step 3: Add audit helper**

In `backend/src/audit.rs`, add:

```rust
pub fn task_candidate_review_set(
    actor_id: impl Into<String>,
    task_candidate_id: impl Into<String>,
) -> Self {
    let task_candidate_id = task_candidate_id.into();
    Self {
        actor_kind: LOCAL_API_TOKEN_ACTOR_KIND.to_owned(),
        actor_id: actor_id.into(),
        operation: "task_candidate.review.set".to_owned(),
        method: "PUT".to_owned(),
        path_template: "/api/v2/task-candidates/{task_candidate_id}/review".to_owned(),
        target_kind: "task_candidate".to_owned(),
        target_id: Some(task_candidate_id),
        metadata: json!({}),
    }
}
```

- [ ] **Step 4: Add routes**

In `build_router_with_database`, add:

```rust
.route("/api/v2/task-candidates", get(get_task_candidates))
.route(
    "/api/v2/task-candidates/{task_candidate_id}/review",
    put(put_task_candidate_review),
)
.route("/api/v2/tasks", get(get_tasks))
```

- [ ] **Step 5: Add request/response types**

In `backend/src/lib.rs`, add:

```rust
#[derive(Serialize)]
struct TaskCandidateListResponse {
    items: Vec<crate::task_candidates::TaskCandidate>,
}

#[derive(Serialize)]
struct TaskListResponse {
    items: Vec<crate::task_candidates::ActiveTask>,
}

#[derive(Deserialize)]
struct TaskCandidateReviewApiRequest {
    command_id: String,
    review_state: String,
}
```

Add parsing so accepted review state strings are exactly `suggested`, `user_confirmed`, and `user_rejected`.

- [ ] **Step 6: Add handlers**

Add handlers with this shape:

```rust
async fn get_task_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskCandidateListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_limit_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?.list_candidates(query.limit).await?;
    Ok(Json(TaskCandidateListResponse { items }))
}

async fn get_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_limit_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?.list_tasks(query.limit).await?;
    Ok(Json(TaskListResponse { items }))
}
```

The review handler must verify actor ID, write an audit record with `task_candidate_review_set`, call `TaskCandidateStore::set_review_state`, and return the command result.

- [ ] **Step 7: Map errors**

Extend `ApiError` so:

- invalid review payload maps to `400 invalid_task_candidate_review`;
- missing candidate maps to `404 task_candidate_not_found`;
- store SQL errors map to `500 task_candidate_store_error`.

- [ ] **Step 8: Run API tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test task_candidates_api -- --nocapture --test-threads=1
```

Expected: PASS or live-test skip when `HERMES_TEST_DATABASE_URL` is unset.

- [ ] **Step 9: Commit API**

Run:

```sh
git add backend/src/audit.rs backend/src/lib.rs backend/tests/task_candidates_api.rs
git commit -m "feat: expose task candidate review APIs"
```

## Task 4: Frontend Tasks Tab

**Files:**
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Add frontend API types**

In `frontend/src/lib/api.ts`, add:

```ts
export type TaskCandidateReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export type TaskCandidate = {
	task_candidate_id: string;
	source_kind: 'message' | 'document';
	source_id: string;
	project_id: string | null;
	title: string;
	due_text: string | null;
	assignee_label: string | null;
	confidence: number;
	review_state: TaskCandidateReviewState;
	evidence_excerpt: string;
	generated_at: string;
	reviewed_at: string | null;
	updated_at: string;
};

export type ActiveTask = {
	task_id: string;
	task_candidate_id: string;
	title: string;
	source_kind: 'message' | 'document';
	source_id: string;
	project_id: string | null;
	status: 'active';
	created_at: string;
	updated_at: string;
};

export type TaskCandidateListResponse = {
	items: TaskCandidate[];
};

export type TaskListResponse = {
	items: ActiveTask[];
};
```

- [ ] **Step 2: Add frontend API functions**

Add:

```ts
export async function fetchTaskCandidates(apiBaseUrl: string, token: string, actorId: string, limit = 50) {
	return fetchJson<TaskCandidateListResponse>(
		`${apiBaseUrl}/api/v2/task-candidates?limit=${limit}`,
		token,
		actorId
	);
}

export async function fetchTasks(apiBaseUrl: string, token: string, actorId: string, limit = 50) {
	return fetchJson<TaskListResponse>(`${apiBaseUrl}/api/v2/tasks?limit=${limit}`, token, actorId);
}

export async function reviewTaskCandidate(
	apiBaseUrl: string,
	token: string,
	actorId: string,
	taskCandidateId: string,
	reviewState: TaskCandidateReviewState
) {
	return fetchJson(`${apiBaseUrl}/api/v2/task-candidates/${encodeURIComponent(taskCandidateId)}/review`, token, actorId, {
		method: 'PUT',
		body: JSON.stringify({
			command_id: `task-candidate-review-${crypto.randomUUID()}`,
			review_state: reviewState
		})
	});
}
```

- [ ] **Step 3: Add page state**

In `frontend/src/routes/+page.svelte`, import the new types/functions and add:

```svelte
let taskCandidates = $state<TaskCandidate[]>([]);
let activeTasks = $state<ActiveTask[]>([]);
let isTasksLoading = $state(false);
let tasksError = $state('');

const suggestedTaskCandidates = $derived(
	taskCandidates.filter((item) => item.review_state === 'suggested')
);
```

- [ ] **Step 4: Load tasks**

Add:

```svelte
async function loadTaskReviewState() {
	isTasksLoading = true;
	try {
		const [candidateResponse, taskResponse] = await Promise.all([
			fetchTaskCandidates(apiBaseUrl, apiToken, actorId, 50),
			fetchTasks(apiBaseUrl, apiToken, actorId, 50)
		]);
		taskCandidates = candidateResponse.items;
		activeTasks = taskResponse.items;
		tasksError = '';
	} catch (error) {
		tasksError = error instanceof Error ? error.message : 'Unknown task candidate error';
	} finally {
		isTasksLoading = false;
	}
}
```

Call it from the existing `onMount` block.

- [ ] **Step 5: Add review action**

Add:

```svelte
async function setTaskCandidateReview(candidate: TaskCandidate, reviewState: TaskCandidateReviewState) {
	try {
		await reviewTaskCandidate(
			apiBaseUrl,
			apiToken,
			actorId,
			candidate.task_candidate_id,
			reviewState
		);
		await loadTaskReviewState();
	} catch (error) {
		tasksError = error instanceof Error ? error.message : 'Unknown task candidate review error';
	}
}
```

- [ ] **Step 6: Replace Tasks tab static data path**

Replace the `currentView === 'tasks'` body with API-backed sections:

- top metrics from `activeTasks.length` and `suggestedTaskCandidates.length`;
- active task rows from `activeTasks`;
- review queue rows from `suggestedTaskCandidates`;
- confirm/reject buttons that call `setTaskCandidateReview`;
- loading, empty and error states.

Do not add mobile-specific layout. Keep the existing panel and table visual language.

- [ ] **Step 7: Run frontend validation**

Run:

```sh
pnpm --dir frontend check
pnpm --dir frontend build
```

Expected: both commands pass.

- [ ] **Step 8: Commit frontend**

Run:

```sh
git add frontend/src/lib/api.ts frontend/src/routes/+page.svelte
git commit -m "feat: render task candidate review workflow"
```

## Task 5: Final Validation

**Files:**
- No new files.

- [ ] **Step 1: Run backend targeted tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test task_candidates --test task_candidates_api -- --nocapture --test-threads=1
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

- [ ] **Step 5: Check git state**

Run:

```sh
git status --short
```

Expected: only intentional task-candidate files are modified.

## Self-Review Checklist

- [ ] Task candidates are deterministic suggestions until reviewed.
- [ ] Review decisions append canonical events.
- [ ] Confirmed candidates create active local tasks.
- [ ] Rejected and reset candidates do not appear as active tasks.
- [ ] API responses do not expose full message bodies.
- [ ] Protected commands require bearer token and actor ID.
- [ ] Frontend Tasks tab no longer depends on static task literals for the main workflow.

## Closure Status

Closed on 2026-06-05 as part of the V2 workflow slices.

Implemented:
- deterministic message/document task candidate storage;
- canonical `task_candidate.review_state_changed` event recording and replay;
- active local task read model for confirmed candidates;
- protected local APIs for candidates, review commands and active tasks;
- desktop Tasks tab backed by local API data.

Validated:
- `cargo test --manifest-path backend/Cargo.toml --test task_candidates --test task_candidates_api -- --nocapture`;
- `make backend-validate`;
- `pnpm --dir frontend check`;
- `pnpm --dir frontend build`.

Not run:
- full `make validate`, because that also runs Docker-backed smoke checks outside this slice;
- browser screenshot smoke, because Playwright was not available in `frontend/node_modules` in this session.
