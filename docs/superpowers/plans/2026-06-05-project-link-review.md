# Project Link Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add event-backed review for project-message and project-document links so keyword-generated project memory can be confirmed, rejected and rebuilt safely.

**Architecture:** Store user review decisions as canonical `project.link_review_state_changed` events and maintain a durable `project_link_reviews` read model from those events. Keep graph tables rebuildable: project graph edges are projected from keyword candidates plus explicit review decisions, not mutated as source-of-truth state. Keep APIs protected by the existing local bearer token plus `X-Hermes-Actor-Id`.

**Tech Stack:** Rust 1.85/edition 2024, Axum, SQLx/PostgreSQL migrations, existing event log and projection runner, SvelteKit 2, Svelte 5 runes, TypeScript, pnpm, Make.

---

## Source Spec

- `docs/superpowers/specs/2026-06-05-project-link-review-design.md`

## Relevant ADRs

- `docs/adr/ADR-0001-event-sourcing-as-system-spine.md` - meaningful changes are canonical events.
- `docs/adr/ADR-0014-canonical-event-envelope.md` - events use versioned envelopes.
- `docs/adr/ADR-0015-command-query-separation.md` - durable mutations pass through commands.
- `docs/adr/ADR-0023-rebuildable-projections.md` - graph/read models are rebuildable.
- `docs/adr/ADR-0038-local-event-api-capability-token.md` - protected APIs use bearer token.
- `docs/adr/ADR-0040-local-api-actor-identity.md` - protected APIs include actor identity.
- `docs/adr/ADR-0045-graph-core-projection.md` - graph is read-only/rebuildable projection.
- `docs/adr/ADR-0047-project-memory-spine.md` - project links start as keyword-derived suggestions.

## File Map

- Create: `docs/adr/ADR-0048-project-link-review-workflow.md`
- Modify: `docs/adr/README.md`
- Create: `backend/migrations/0014_create_project_link_reviews.sql`
- Modify: `backend/src/event_log.rs`
- Modify: `backend/src/audit.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/src/project_link_reviews.rs`
- Modify: `backend/src/projects.rs`
- Modify: `backend/src/graph_projection.rs`
- Modify: `backend/src/graph.rs` only if a parser/export gap is found; expected not needed.
- Create: `backend/tests/project_link_reviews.rs`
- Modify: `backend/tests/projects.rs`
- Modify: `backend/tests/projects_api.rs`
- Modify: `backend/tests/graph_projection.rs`
- Modify: `backend/tests/projection_runner.rs` only if shared helpers are needed; expected not needed.
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

## Data Contracts

Use these backend-facing domain values:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkTargetKind {
    Message,
    Document,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

pub struct ProjectLinkReviewCommand {
    pub command_id: String,
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub actor_id: String,
}
```

Use this frontend-facing request shape:

```ts
export type ProjectLinkReviewRequest = {
	command_id: string;
	target_kind: 'message' | 'document';
	target_id: string;
	review_state: 'suggested' | 'user_confirmed' | 'user_rejected';
};
```

---

## Task 1: ADR And Architecture Documentation

**Files:**
- Create: `docs/adr/ADR-0048-project-link-review-workflow.md`
- Modify: `docs/adr/README.md`

- [ ] **Step 1: Add ADR-0048**

Create `docs/adr/ADR-0048-project-link-review-workflow.md` with this decision:

```markdown
# ADR-0048 Project Link Review Workflow

Status: Proposed

## Context

ADR-0047 introduced project nodes and keyword-derived project relationships. Those relationships are suggested because deterministic keyword containment can create false positives and false negatives.

ADR-0001 requires meaningful changes to be represented as canonical events. ADR-0023 and ADR-0045 make graph tables rebuildable projections, so user review decisions cannot live only on graph edges.

## Decision

Add event-backed project link review for direct project-to-message and project-to-document links.

User review commands append `project.link_review_state_changed` events. A durable `project_link_reviews` read model stores only explicit decisions:

- `user_confirmed`
- `user_rejected`

Resetting a link to `suggested` appends an event and removes the explicit decision row. Unreviewed suggested links remain derived from project keyword rules.

Project graph edges remain rebuildable projection state. During graph projection:

- keyword-only active links use `review_state = suggested`;
- confirmed links use `review_state = user_confirmed`;
- rejected links are omitted;
- confirmed links remain active even when current keyword rules do not match.

People and email-address project links remain derived from active project-message links. Direct people review is out of scope for this slice.

Protected local review APIs must require the temporary local bearer token and `X-Hermes-Actor-Id`.

## Non-Goals

- Project create/edit UI.
- Keyword management UI.
- Manual people/contact merge.
- Direct review of project-person edges.
- AI project inference.
- OCR or entity extraction.
- Mobile UI.

## Consequences

Positive:

- False project links can be rejected without editing source messages or documents.
- Important links can be confirmed even if keyword rules later change.
- Review state survives graph rebuild.
- Project detail and graph projection can share the same active-link rules.

Negative:

- Review commands require event/table transaction discipline.
- The first workflow only handles direct message and document links.
- A later keyword editor still needs separate ADR-backed work.
```

- [ ] **Step 2: Update ADR index**

Add this line to `docs/adr/README.md` in numeric order:

```markdown
- [ADR-0048 Project Link Review Workflow](ADR-0048-project-link-review-workflow.md)
```

- [ ] **Step 3: Validate documentation references**

Run:

```sh
test -f docs/adr/ADR-0048-project-link-review-workflow.md
rg -n "ADR-0048" docs/adr/README.md docs/adr/ADR-0048-project-link-review-workflow.md
```

Expected: both commands pass and print the ADR reference.

- [ ] **Step 4: Commit documentation**

Run:

```sh
git add docs/adr/ADR-0048-project-link-review-workflow.md docs/adr/README.md
git commit -m "docs: add project link review ADR"
```

---

## Task 2: Schema, Event Append Transaction And Review Store

**Files:**
- Create: `backend/migrations/0014_create_project_link_reviews.sql`
- Modify: `backend/src/event_log.rs`
- Create: `backend/src/project_link_reviews.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/project_link_reviews.rs`

- [ ] **Step 1: Write failing schema/store tests**

Create `backend/tests/project_link_reviews.rs` with live PostgreSQL tests for:

- confirmed review is stored and returned;
- rejected review excludes a keyword match from active candidates;
- suggested reset clears the explicit decision;
- `project.link_review_state_changed` event can be replayed into the review table.

Use test names:

```rust
#[tokio::test]
async fn project_link_review_command_appends_event_and_updates_review_against_postgres() {}

#[tokio::test]
async fn project_link_review_reset_removes_explicit_decision_against_postgres() {}

#[tokio::test]
async fn project_link_review_projection_rebuilds_review_state_from_event_against_postgres() {}
```

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test project_link_reviews -- --nocapture --test-threads=1
```

Expected: fails because migration/module does not exist.

- [ ] **Step 2: Add migration 0014**

Create `backend/migrations/0014_create_project_link_reviews.sql`:

```sql
CREATE TABLE IF NOT EXISTS project_link_reviews (
    project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
    target_kind TEXT NOT NULL,
    target_id TEXT NOT NULL,
    review_state TEXT NOT NULL,
    event_id TEXT NOT NULL REFERENCES event_log(event_id),
    actor_id TEXT NOT NULL,
    reviewed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT project_link_reviews_target_kind_check
        CHECK (target_kind IN ('message', 'document')),
    CONSTRAINT project_link_reviews_review_state_check
        CHECK (review_state IN ('user_confirmed', 'user_rejected')),
    CONSTRAINT project_link_reviews_actor_id_not_empty
        CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT project_link_reviews_project_id_not_empty
        CHECK (length(trim(project_id)) > 0),
    CONSTRAINT project_link_reviews_target_id_not_empty
        CHECK (length(trim(target_id)) > 0),
    PRIMARY KEY (project_id, target_kind, target_id)
);

CREATE INDEX IF NOT EXISTS project_link_reviews_event_id_idx
    ON project_link_reviews (event_id);

CREATE INDEX IF NOT EXISTS project_link_reviews_review_state_idx
    ON project_link_reviews (review_state, updated_at);
```

- [ ] **Step 3: Add transaction-capable event append**

Modify `backend/src/event_log.rs`:

```rust
use sqlx::{Postgres, Row, Transaction};
```

Add to `impl EventStore`:

```rust
pub async fn append_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    event: &NewEventEnvelope,
) -> Result<i64, EventStoreError> {
    let position = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO event_log (
            event_id,
            event_type,
            schema_version,
            occurred_at,
            source,
            actor,
            subject,
            payload,
            provenance,
            causation_id,
            correlation_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING position
        "#,
    )
    .bind(&event.event_id)
    .bind(&event.event_type)
    .bind(event.schema_version)
    .bind(event.occurred_at)
    .bind(&event.source)
    .bind(&event.actor)
    .bind(&event.subject)
    .bind(&event.payload)
    .bind(&event.provenance)
    .bind(&event.causation_id)
    .bind(&event.correlation_id)
    .fetch_one(&mut **transaction)
    .await?;

    Ok(position)
}
```

Refactor `append` to open a transaction and call `append_in_transaction`, then commit. Keep existing behavior unchanged.

- [ ] **Step 4: Add `project_link_reviews` module**

Create `backend/src/project_link_reviews.rs` with:

- enums for target kind and review state;
- validation helpers;
- `ProjectLinkReviewStore`;
- `set_review_state`;
- `apply_review_event`;
- active candidate read methods for messages/documents.

Core method signatures:

```rust
impl ProjectLinkReviewStore {
    pub fn new(pool: PgPool) -> Self;

    pub async fn set_review_state(
        &self,
        command: &ProjectLinkReviewCommand,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewError>;

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), ProjectLinkReviewError>;

    pub async fn explicit_review(
        &self,
        project_id: &str,
        target_kind: ProjectLinkTargetKind,
        target_id: &str,
    ) -> Result<Option<ProjectLinkReview>, ProjectLinkReviewError>;
}
```

`set_review_state` must:

- validate `command_id`, `project_id`, `target_id` and `actor_id`;
- validate project exists;
- validate message/document target exists;
- build `project.link_review_state_changed`;
- append event and update/delete `project_link_reviews` in one transaction;
- return the applied review state and event ID.

- [ ] **Step 5: Export the module**

Modify `backend/src/lib.rs` near existing module exports:

```rust
pub mod project_link_reviews;
```

- [ ] **Step 6: Run store tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test project_link_reviews -- --nocapture --test-threads=1
```

Expected: all project link review store tests pass.

- [ ] **Step 7: Commit schema/store**

Run:

```sh
git add backend/migrations/0014_create_project_link_reviews.sql backend/src/event_log.rs backend/src/project_link_reviews.rs backend/src/lib.rs backend/tests/project_link_reviews.rs
git commit -m "feat: add project link review store"
```

---

## Task 3: Protected Review APIs

**Files:**
- Modify: `backend/src/audit.rs`
- Modify: `backend/src/lib.rs`
- Modify: `backend/tests/projects_api.rs`

- [ ] **Step 1: Add failing API tests**

Extend `backend/tests/projects_api.rs` with tests:

```rust
#[tokio::test]
async fn project_link_candidates_reject_missing_local_api_token() {}

#[tokio::test]
async fn project_link_candidates_return_safe_message_and_document_candidates() {}

#[tokio::test]
async fn put_project_link_review_requires_actor_and_updates_review_state() {}

#[tokio::test]
async fn put_project_link_review_rejects_missing_target() {}
```

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test projects_api project_link -- --nocapture --test-threads=1
```

Expected: fails because routes do not exist.

- [ ] **Step 2: Add audit helper**

Modify `backend/src/audit.rs`:

```rust
pub fn project_link_review_set(
    actor_id: impl Into<String>,
    project_id: impl Into<String>,
    target_kind: impl Into<String>,
    target_id: impl Into<String>,
) -> Self {
    let project_id = project_id.into();
    let target_kind = target_kind.into();
    let target_id = target_id.into();

    Self {
        actor_kind: LOCAL_API_TOKEN_ACTOR_KIND.to_owned(),
        actor_id: actor_id.into(),
        operation: "project.link_review.set".to_owned(),
        method: "PUT".to_owned(),
        path_template: "/api/v2/projects/{project_id}/link-reviews".to_owned(),
        target_kind: "project_link".to_owned(),
        target_id: Some(format!("{project_id}:{target_kind}:{target_id}")),
        metadata: json!({
            "project_id": project_id,
            "target_kind": target_kind,
            "target_id": target_id
        }),
    }
}
```

- [ ] **Step 3: Add request/response types**

Modify `backend/src/lib.rs` and add API DTOs near project query DTOs:

```rust
#[derive(Deserialize)]
struct ProjectLinkReviewApiRequest {
    command_id: String,
    target_kind: String,
    target_id: String,
    review_state: String,
}

#[derive(Serialize)]
struct ProjectLinkReviewApiResponse {
    project_id: String,
    target_kind: String,
    target_id: String,
    review_state: String,
    event_id: String,
}
```

Add `ApiError` variants:

```rust
InvalidProjectLinkReview(&'static str),
ProjectLinkTargetNotFound,
ProjectLinkReview(ProjectLinkReviewError),
```

Map them to:

- `400 invalid_project_link_review`
- `404 project_link_target_not_found`
- `500 project_link_review_store_error`

- [ ] **Step 4: Add routes**

Modify router construction in `backend/src/lib.rs`:

```rust
.route(
    "/api/v2/projects/{project_id}/link-candidates",
    get(get_project_link_candidates),
)
.route(
    "/api/v2/projects/{project_id}/link-reviews",
    put(put_project_link_review),
)
```

Ensure `put` is imported from `axum::routing`.

- [ ] **Step 5: Implement handlers**

Add handlers:

```rust
async fn get_project_link_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectLinkCandidateListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_project_link_candidates_query(raw_query.as_deref())?;
    let candidates = project_link_review_store(&state)?
        .list_candidates(&project_id, query.limit)
        .await?;

    Ok(Json(ProjectLinkCandidateListResponse { items: candidates }))
}

async fn put_project_link_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(request): Json<ProjectLinkReviewApiRequest>,
) -> Result<Json<ProjectLinkReviewApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let command = request.into_command(project_id, actor.actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::project_link_review_set(
            command.actor_id.clone(),
            command.project_id.clone(),
            command.target_kind.as_str(),
            command.target_id.clone(),
        ))
        .await?;

    let result = project_link_review_store(&state)?
        .set_review_state(&command)
        .await?;

    Ok(Json(ProjectLinkReviewApiResponse::from(result)))
}
```

- [ ] **Step 6: Run API tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test projects_api project_link -- --nocapture --test-threads=1
```

Expected: all project link API tests pass.

- [ ] **Step 7: Commit API**

Run:

```sh
git add backend/src/audit.rs backend/src/lib.rs backend/tests/projects_api.rs
git commit -m "feat: expose project link review API"
```

---

## Task 4: Apply Review Decisions To Project Reads And Graph Projection

**Files:**
- Modify: `backend/src/projects.rs`
- Modify: `backend/src/project_link_reviews.rs`
- Modify: `backend/src/graph_projection.rs`
- Modify: `backend/tests/projects.rs`
- Modify: `backend/tests/graph_projection.rs`

- [ ] **Step 1: Add failing project detail tests**

Extend `backend/tests/projects.rs` with tests:

```rust
#[tokio::test]
async fn project_detail_excludes_rejected_keyword_message_against_postgres() {}

#[tokio::test]
async fn project_detail_includes_confirmed_non_keyword_message_against_postgres() {}
```

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test projects project_detail -- --nocapture --test-threads=1
```

Expected: fails because project detail still uses keyword-only queries.

- [ ] **Step 2: Add active link query helpers**

In `backend/src/project_link_reviews.rs`, add read helpers that return active message/document target IDs:

```rust
pub async fn active_message_ids_for_project(
    &self,
    project_id: &str,
) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError>;

pub async fn active_document_ids_for_project(
    &self,
    project_id: &str,
) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError>;
```

The SQL rule must be:

```sql
WITH keyword_matches AS (...),
confirmed AS (...),
rejected AS (...),
active AS (
    SELECT target_id, 'suggested' AS review_state FROM keyword_matches
    UNION
    SELECT target_id, 'user_confirmed' AS review_state FROM confirmed
)
SELECT active.target_id, max(active.review_state) AS review_state
FROM active
WHERE NOT EXISTS (
    SELECT 1 FROM rejected WHERE rejected.target_id = active.target_id
)
GROUP BY active.target_id
```

Implement separately for messages and documents so target existence remains explicit and SQL stays readable.

- [ ] **Step 3: Switch project detail queries to active links**

Modify `backend/src/projects.rs` so these methods use active reviewed links instead of raw keyword `EXISTS` clauses:

- `project_stats`
- `project_messages`
- `project_documents`
- `project_people`
- `project_timeline`
- `matching_project_messages`
- `matching_project_documents`

Do not expose message bodies. Do not change public project API shape unless tests require adding review state to candidate endpoints only.

- [ ] **Step 4: Add failing graph projection tests**

Extend `backend/tests/graph_projection.rs` with:

```rust
#[tokio::test]
async fn graph_projection_omits_rejected_project_link_against_postgres() {}

#[tokio::test]
async fn graph_projection_marks_confirmed_project_link_user_confirmed_against_postgres() {}
```

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test graph_projection project_link -- --nocapture --test-threads=1
```

Expected: fails because projection still emits all keyword matches as `suggested`.

- [ ] **Step 5: Apply review state in graph projection**

Modify `backend/src/graph_projection.rs`:

- load active reviewed message/document links for each project;
- pass review state into `project_project_message`, `project_project_document` and `project_project_people`;
- map `ProjectLinkReviewState::Suggested` to `GraphReviewState::Suggested`;
- map `ProjectLinkReviewState::UserConfirmed` to `GraphReviewState::UserConfirmed`;
- never emit rejected links.

Keep `PROJECT_KEYWORD_CONFIDENCE` for suggested links. Use `1.0` confidence for user-confirmed links.

- [ ] **Step 6: Run project and graph tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test projects --test project_link_reviews --test graph_projection project -- --nocapture --test-threads=1
```

Expected: all selected tests pass.

- [ ] **Step 7: Commit projection/read changes**

Run:

```sh
git add backend/src/projects.rs backend/src/project_link_reviews.rs backend/src/graph_projection.rs backend/tests/projects.rs backend/tests/graph_projection.rs
git commit -m "feat: apply project link reviews to projections"
```

---

## Task 5: Frontend Review Queue

**Files:**
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Add frontend API types**

Modify `frontend/src/lib/api.ts`:

```ts
export type ProjectLinkTargetKind = 'message' | 'document';

export type ProjectLinkReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export type ProjectLinkCandidate = {
	project_id: string;
	target_kind: ProjectLinkTargetKind;
	target_id: string;
	graph_node_id: string;
	title: string;
	subtitle: string;
	source_label: string;
	occurred_at: string;
	review_state: ProjectLinkReviewState;
	evidence_excerpt: string | null;
};

export type ProjectLinkCandidateListResponse = {
	items: ProjectLinkCandidate[];
};

export type ProjectLinkReviewRequest = {
	command_id: string;
	target_kind: ProjectLinkTargetKind;
	target_id: string;
	review_state: ProjectLinkReviewState;
};

export type ProjectLinkReviewResponse = {
	project_id: string;
	target_kind: ProjectLinkTargetKind;
	target_id: string;
	review_state: ProjectLinkReviewState;
	event_id: string;
};
```

- [ ] **Step 2: Add frontend API functions**

Modify `frontend/src/lib/api.ts`:

```ts
export async function fetchProjectLinkCandidates(
	baseUrl: string,
	token: string,
	actorId: string,
	projectId: string,
	limit = 25
): Promise<ProjectLinkCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/projects/${encodeURIComponent(projectId)}/link-candidates?${params.toString()}`,
		'Project link candidates request failed'
	);
}

export async function putProjectLinkReview(
	baseUrl: string,
	token: string,
	actorId: string,
	projectId: string,
	request: ProjectLinkReviewRequest
): Promise<ProjectLinkReviewResponse> {
	return putJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/projects/${encodeURIComponent(projectId)}/link-reviews`,
		request,
		'Project link review request failed'
	);
}
```

If `putJson` does not exist, add it next to `postJson` using the same headers and error handling, with `method: 'PUT'`.

- [ ] **Step 3: Add Projects tab state**

Modify `frontend/src/routes/+page.svelte`:

```svelte
let projectLinkCandidates = $state<ProjectLinkCandidate[]>([]);
let isProjectLinkCandidatesLoading = $state(false);
let projectLinkReviewError = $state('');
let projectLinkReviewPendingKey = $state('');
```

Add derived values:

```svelte
const suggestedProjectLinks = $derived(
	projectLinkCandidates.filter((item) => item.review_state === 'suggested')
);

const confirmedProjectLinks = $derived(
	projectLinkCandidates.filter((item) => item.review_state === 'user_confirmed')
);
```

- [ ] **Step 4: Load candidates with project detail**

After successful `loadProjectDetail(projectId)`, call:

```svelte
await loadProjectLinkCandidates(projectId);
```

Add:

```svelte
async function loadProjectLinkCandidates(projectId: string) {
	if (!projectId) {
		projectLinkCandidates = [];
		return;
	}

	isProjectLinkCandidatesLoading = true;
	try {
		const response = await fetchProjectLinkCandidates(apiBaseUrl, apiToken, actorId, projectId, 25);
		projectLinkCandidates = response.items;
		projectLinkReviewError = '';
	} catch (error) {
		projectLinkReviewError =
			error instanceof Error ? error.message : 'Unknown project link review error';
	} finally {
		isProjectLinkCandidatesLoading = false;
	}
}
```

- [ ] **Step 5: Add review action function**

Add:

```svelte
async function setProjectLinkReview(candidate: ProjectLinkCandidate, reviewState: ProjectLinkReviewState) {
	if (!selectedProjectRecord) {
		return;
	}

	const pendingKey = `${candidate.target_kind}:${candidate.target_id}:${reviewState}`;
	projectLinkReviewPendingKey = pendingKey;
	try {
		await putProjectLinkReview(apiBaseUrl, apiToken, actorId, selectedProjectRecord.project_id, {
			command_id: crypto.randomUUID(),
			target_kind: candidate.target_kind,
			target_id: candidate.target_id,
			review_state: reviewState
		});
		await loadProjectDetail(selectedProjectRecord.project_id);
		await loadProjectLinkCandidates(selectedProjectRecord.project_id);
		projectLinkReviewError = '';
	} catch (error) {
		projectLinkReviewError =
			error instanceof Error ? error.message : 'Unknown project link review error';
	} finally {
		projectLinkReviewPendingKey = '';
	}
}
```

- [ ] **Step 6: Render review queue**

In the Projects view overview, add one panel near recent communications/documents:

```svelte
<section class="panel project-link-review-panel">
	<div class="panel-heading">
		<div>
			<h3>Link Review</h3>
			<p>Confirm or reject project matches from local evidence.</p>
		</div>
		<span class="badge">{suggestedProjectLinks.length}</span>
	</div>

	{#if projectLinkReviewError}
		<p class="inline-error">{projectLinkReviewError}</p>
	{/if}

	{#if isProjectLinkCandidatesLoading}
		<p class="muted">Loading project links...</p>
	{:else if suggestedProjectLinks.length === 0}
		<p class="muted">No suggested links need review.</p>
	{:else}
		<div class="project-link-review-list">
			{#each suggestedProjectLinks as candidate}
				<article class="project-link-review-item">
					<div>
						<span class="eyebrow">{candidate.target_kind}</span>
						<strong>{candidate.title}</strong>
						<p>{candidate.subtitle}</p>
					</div>
					<div class="project-link-review-actions">
						<button
							type="button"
							class="icon-action"
							disabled={projectLinkReviewPendingKey !== ''}
							onclick={() => setProjectLinkReview(candidate, 'user_confirmed')}
							title="Confirm link"
						>
							<Icon icon="tabler:check" />
						</button>
						<button
							type="button"
							class="icon-action"
							disabled={projectLinkReviewPendingKey !== ''}
							onclick={() => setProjectLinkReview(candidate, 'user_rejected')}
							title="Reject link"
						>
							<Icon icon="tabler:x" />
						</button>
					</div>
				</article>
			{/each}
		</div>
	{/if}
</section>
```

Keep styling consistent with existing panels; do not change the global palette, sidebar language or tab colors.

- [ ] **Step 7: Run frontend validation**

Run:

```sh
cd frontend && pnpm check
cd frontend && pnpm build
```

Expected: both pass.

- [ ] **Step 8: Commit frontend**

Run:

```sh
git add frontend/src/lib/api.ts frontend/src/routes/+page.svelte
git commit -m "feat: add project link review UI"
```

---

## Task 6: Full Validation And Final Commit Gate

**Files:**
- Inspect working tree only unless validation finds defects.

- [ ] **Step 1: Run formatting check**

Run:

```sh
cargo fmt --manifest-path backend/Cargo.toml -- --check
```

Expected: pass. If it fails, run `cargo fmt --manifest-path backend/Cargo.toml`, inspect the diff, then rerun the check.

- [ ] **Step 2: Run targeted backend tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test project_link_reviews --test projects --test projects_api --test graph_projection project -- --nocapture --test-threads=1
```

Expected: pass.

- [ ] **Step 3: Run backend clippy**

Run:

```sh
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: pass.

- [ ] **Step 4: Run frontend validation**

Run:

```sh
cd frontend && pnpm check
cd frontend && pnpm build
```

Expected: both pass.

- [ ] **Step 5: Run repository validation gate**

Run:

```sh
make validate
```

Expected: pass.

- [ ] **Step 6: Browser smoke**

With dev services running, open the Projects tab at `http://127.0.0.1:5174/` and verify:

- Projects tab renders without console errors.
- Link Review panel appears for the selected project.
- Confirming a suggested link reloads project detail and the candidate list.
- Rejecting a suggested link removes it from project detail after reload.
- No mobile viewport validation is performed because `ADR-0031` keeps mobile UI out of scope.

- [ ] **Step 7: Final git status**

Run:

```sh
git status --short
```

Expected: clean after the final commit.

If there are remaining implementation changes, commit them with a focused message:

```sh
git add <changed-files>
git commit -m "fix: complete project link review validation"
```

## Self-Review Checklist

- [ ] ADR-0048 exists before schema/API implementation.
- [ ] Review decisions are represented by canonical events.
- [ ] `project_link_reviews` can be rebuilt from events.
- [ ] Graph edges remain rebuildable projection state.
- [ ] Rejected links are omitted from project detail and graph projection.
- [ ] Confirmed links survive keyword changes.
- [ ] Project API responses do not expose message bodies.
- [ ] Protected API calls require bearer token and actor ID.
- [ ] Frontend preserves existing desktop visual language and does not add mobile work.
