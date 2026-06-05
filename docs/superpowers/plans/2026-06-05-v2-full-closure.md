# V2 Full Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close Hermes Hub Version 2 as a release-quality, graph-backed memory workflow milestone with explicit acceptance gates, non-destructive contact merge/split review, document-processing retry, and full validation coverage.

**Architecture:** Keep V2 local-first, event-backed and review-first. Do not introduce Version 3 AI agent runtime, Ollama extraction, remote OCR, fuzzy automatic identity collapse, provider writes or mobile UI. Closing work strengthens the current Rust/Axum/PostgreSQL backend, SvelteKit desktop shell and Makefile validation gate around already implemented V2 workflow slices.

**Tech Stack:** Rust 1.85/edition 2024, Axum, SQLx/PostgreSQL migrations, canonical event log, API audit log, SvelteKit 2, Svelte 5 runes, TypeScript, pnpm, Make, Docker Compose development PostgreSQL.

---

## Current Baseline

Verified repository state before this plan:

- `docs/roadmap/v1-closure-checklist.md` is fully checked.
- `docs/roadmap/v2-graph-core-checklist.md` is fully checked for the first V2 graph core slice.
- Existing V2 implementation includes graph core, graph explorer, project memory spine, project link review, task candidates, contact identity review, document processing jobs/artifacts, and live mail blob cache.
- `make backend-validate`, `make frontend-check`, and `make frontend-build` passed in the status pass before this plan.
- Full `make validate` was not run in that pass.

## V2 Closure Definition

V2 is closed when:

- graph-backed memory links contacts, messages, documents and projects with evidence;
- project timelines and project link review use backend data;
- task candidates from messages/documents are reviewable before active local tasks exist;
- contact identity supports explicit non-destructive merge and split review;
- document processing exposes extraction/OCR status and lets failed jobs be retried through a protected command;
- all V2 protected APIs require bearer token and `X-Hermes-Actor-Id` where commands are involved;
- full `make validate` passes with V2 workflow live PostgreSQL smoke tests included;
- the desktop shell has no V2-critical static surfaces for graph, projects, tasks, identity review or document processing.

## Out Of Scope

- Version 3 agent runtime.
- Ollama or AI-backed extraction.
- Embedding provider or retrieval planner.
- Remote OCR service.
- Fuzzy automatic identity merge.
- Graph editing.
- Provider task/calendar writes.
- Mobile UI design, implementation or validation.

## Relevant ADRs

- `docs/adr/ADR-0001-event-sourcing-as-system-spine.md`
- `docs/adr/ADR-0008-knowledge-graph-first.md`
- `docs/adr/ADR-0015-command-query-separation.md`
- `docs/adr/ADR-0017-document-processing-pipeline.md`
- `docs/adr/ADR-0019-contact-identity-resolution.md`
- `docs/adr/ADR-0020-task-candidate-lifecycle.md`
- `docs/adr/ADR-0023-rebuildable-projections.md`
- `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md`
- `docs/adr/ADR-0038-local-event-api-capability-token.md`
- `docs/adr/ADR-0040-local-api-actor-identity.md`
- `docs/adr/ADR-0045-graph-core-projection.md`
- `docs/adr/ADR-0047-project-memory-spine.md`
- `docs/adr/ADR-0048-project-link-review-workflow.md`

## File Map

- Create: `docs/roadmap/v2-closure-checklist.md` - full Version 2 release checklist and acceptance gate.
- Modify: `docs/roadmap/product-roadmap.md` - point Version 2 readers at the full closure checklist.
- Modify: `backend/README.md` - document V2 workflow APIs, smoke target and document retry command.
- Modify: `frontend/README.md` - document desktop V2 surfaces and validation commands.
- Modify: `backend/src/contact_identity.rs` - generate split candidates for confirmed merge decisions and expose active identity links.
- Modify: `backend/tests/contact_identity.rs` - live PostgreSQL coverage for split candidate generation and active identity behavior.
- Modify: `backend/tests/contact_identity_api.rs` - API response coverage for confirmed links and split candidates.
- Modify: `frontend/src/lib/api.ts` - contact identity detail response shape and document retry helper.
- Modify: `frontend/src/routes/+page.svelte` - render split controls for confirmed identity links and retry controls for failed document processing jobs.
- Modify: `backend/src/document_processing.rs` - add event-backed retry command for failed jobs.
- Modify: `backend/src/audit.rs` - audit document processing retry API calls.
- Modify: `backend/src/lib.rs` - route document processing retry command and map stable errors.
- Modify: `backend/tests/document_processing.rs` - store-level retry coverage.
- Modify: `backend/tests/document_processing_api.rs` - protected retry API coverage.
- Modify: `Makefile` - add V2 workflow live smoke target and include it in `make validate`.

---

### Task 1: V2 Closure Checklist And Release Documentation

**Files:**
- Create: `docs/roadmap/v2-closure-checklist.md`
- Modify: `docs/roadmap/product-roadmap.md`
- Modify: `backend/README.md`
- Modify: `frontend/README.md`

- [ ] **Step 1: Create the full V2 closure checklist**

Create `docs/roadmap/v2-closure-checklist.md` with this content:

```markdown
# V2 Closure Checklist

## Release Goal

Version 2.0 is complete when Hermes Hub makes graph-backed memory central: messages, contacts, documents and projects are connected through rebuildable graph projections, reviewable workflow candidates, visible document processing state and desktop-only backend-backed UI surfaces.

## In Scope

- V2 graph core projection from contacts, messages and documents.
- Project memory spine with project timelines and keyword-derived evidence-backed links.
- Project link review commands backed by canonical events.
- Source-backed task candidates from messages and documents with explicit review before active local tasks exist.
- Contact identity merge/split review without ambiguous automatic identity collapse.
- Document processing jobs and artifacts for Markdown/text extraction and OCR state.
- Protected read/write APIs using the local bearer token and `X-Hermes-Actor-Id` for commands.
- Desktop/laptop SvelteKit surfaces for graph, projects, task candidates, contact identity and document processing.
- Full local validation through `make validate`.

## Out Of Scope For V2

- Version 3 agent runtime.
- Ollama or AI-backed extraction.
- Embedding provider and retrieval planner.
- Remote OCR service.
- Provider task/calendar writes.
- Graph editing.
- Mobile UI design, implementation or validation.

## Acceptance Gate Status

- [x] V2 graph core projection is implemented and covered by live PostgreSQL smoke validation.
- [x] Knowledge Graph explorer reads summary, search, picker and neighborhood APIs.
- [x] Project memory spine is implemented with project records, timelines and graph links.
- [x] Project link review commands append canonical events and survive graph rebuild.
- [x] Task candidate review creates active local tasks only after explicit confirmation.
- [x] Contact identity review creates conservative merge candidates without mutating contacts.
- [x] Document processing jobs/artifacts exist and Markdown extraction is implemented.
- [ ] Contact identity supports explicit split review for confirmed merge links.
- [ ] Document processing failed jobs can be retried through a protected event-backed command.
- [ ] `make validate` includes live PostgreSQL smoke coverage for V2 workflow APIs.
- [ ] Backend README documents all V2 workflow APIs and dev commands.
- [ ] Frontend README documents V2 desktop surfaces and validation commands.
- [ ] Full `make validate` passes from a clean checkout with Docker available.
- [ ] Desktop browser smoke validates graph, projects, tasks, contacts and document-processing surfaces.
```

- [ ] **Step 2: Verify the checklist file exists and has the expected gates**

Run:

```sh
test -f docs/roadmap/v2-closure-checklist.md
rg -n "Contact identity supports explicit split review|Document processing failed jobs can be retried|make validate" docs/roadmap/v2-closure-checklist.md
```

Expected: both commands pass and `rg` prints the three gate lines.

- [ ] **Step 3: Link the full checklist from the product roadmap**

In `docs/roadmap/product-roadmap.md`, add this paragraph at the end of the `Version 2.0 - Knowledge Graph and Documents` section, after `Dependencies`:

```markdown
Closure tracking:

- [V2 Closure Checklist](v2-closure-checklist.md)
```

- [ ] **Step 4: Add V2 workflow API notes to backend README**

In `backend/README.md`, add this section after the existing V2 graph API bullets:

```markdown
## V2 Workflow APIs

All endpoints below require `Authorization: Bearer <HERMES_LOCAL_API_TOKEN>`. Review and retry commands also require `X-Hermes-Actor-Id`.

- `GET /api/v2/projects` - lists local project records with derived stats.
- `GET /api/v2/projects/{project_id}` - returns project detail, timeline, messages, documents and people.
- `GET /api/v2/projects/{project_id}/link-candidates` - returns safe project message/document link candidates.
- `PUT /api/v2/projects/{project_id}/link-reviews` - records project link review state as a canonical event.
- `GET /api/v2/task-candidates` - lists source-backed task candidates.
- `PUT /api/v2/task-candidates/{task_candidate_id}/review` - records task candidate review state as a canonical event.
- `GET /api/v2/tasks` - lists active local tasks created from confirmed candidates.
- `GET /api/v2/identity-candidates` - lists contact identity candidates.
- `PUT /api/v2/identity-candidates/{identity_candidate_id}/review` - records identity candidate review state as a canonical event.
- `GET /api/v2/contacts/{contact_id}/identity` - returns confirmed identity links and available split reviews for one contact.
- `GET /api/v2/documents/{document_id}/processing` - returns processing jobs and artifacts for one document.
- `GET /api/v2/document-processing/jobs` - lists recent document processing jobs.
- `POST /api/v2/document-processing/jobs/{job_id}/retry` - requeues a failed processing job through a canonical retry event.
```

- [ ] **Step 5: Add frontend V2 surface notes**

In `frontend/README.md`, add this section:

````markdown
## V2 Desktop Surfaces

The desktop shell is intentionally desktop/laptop scoped under ADR-0031. The V2 surfaces are:

- Knowledge Graph explorer using graph summary, node picker, search and neighborhood APIs.
- Projects tab using project records, timelines and project link review commands.
- Tasks tab using task candidate and active task APIs.
- Contacts identity review surface using identity candidates and confirmed identity links.
- Document processing status surface using document processing job and artifact APIs.

Validate frontend changes with:

```sh
pnpm check
pnpm build
```
````

- [ ] **Step 6: Validate docs references**

Run:

```sh
rg -n "V2 Closure Checklist|V2 Workflow APIs|V2 Desktop Surfaces" docs/roadmap/product-roadmap.md backend/README.md frontend/README.md docs/roadmap/v2-closure-checklist.md
git diff --check
```

Expected: both commands pass.

- [ ] **Step 7: Commit release documentation**

Run:

```sh
git add docs/roadmap/v2-closure-checklist.md docs/roadmap/product-roadmap.md backend/README.md frontend/README.md
git commit -m "docs: add v2 closure checklist"
```

---

### Task 2: Contact Identity Merge/Split Closure

**Files:**
- Modify: `backend/src/contact_identity.rs`
- Modify: `backend/tests/contact_identity.rs`
- Modify: `backend/tests/contact_identity_api.rs`
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Add failing store coverage for split candidates**

Append this test to `backend/tests/contact_identity.rs`:

```rust
#[tokio::test]
async fn contact_identity_refresh_creates_split_candidate_for_confirmed_merge_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity split test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Split Candidate {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("split.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("split.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    context.store.refresh_candidates(100).await.expect("refresh");
    let merge_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);
    context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: format!("identity-merge-before-split-{suffix}"),
            identity_candidate_id: merge_candidate_id.clone(),
            review_state: ContactIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge");

    context.store.refresh_candidates(100).await.expect("refresh split candidate");
    let split_candidate_id = merge_candidate_id.replacen("merge_contacts", "split_contact", 1);
    let row: (String, String, String) = sqlx::query_as(
        r#"
        SELECT identity_candidate_id, candidate_kind, review_state
        FROM contact_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&split_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("split candidate row");

    assert_eq!(row.0, split_candidate_id);
    assert_eq!(row.1, "split_contact");
    assert_eq!(row.2, "suggested");
}
```

- [ ] **Step 2: Run the split candidate test to verify RED**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test contact_identity contact_identity_refresh_creates_split_candidate_for_confirmed_merge_against_postgres -- --nocapture --test-threads=1
```

Expected: FAIL with `split candidate row` not found when `HERMES_TEST_DATABASE_URL` is set. If the variable is not set, the test prints the skip message; set it through `make db-up` and `docker/.env` before continuing.

- [ ] **Step 3: Generate split candidates from confirmed merge candidates**

In `backend/src/contact_identity.rs`, update `refresh_candidates` so after the existing same-display-name merge loop it also creates split candidates for confirmed merges:

```rust
        let split_rows = sqlx::query(
            r#"
            SELECT
                left_contact_id,
                right_contact_id,
                evidence_summary
            FROM contact_identity_candidates
            WHERE candidate_kind = 'merge_contacts'
              AND review_state = 'user_confirmed'
              AND right_contact_id IS NOT NULL
            ORDER BY updated_at DESC, identity_candidate_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        for row in split_rows {
            let left = row.try_get::<String, _>("left_contact_id")?;
            let right = row.try_get::<String, _>("right_contact_id")?;
            let candidate = ContactIdentityCandidatePayload {
                candidate_kind: ContactIdentityCandidateKind::SplitContact,
                left_contact_id: left,
                right_contact_id: Some(right),
                email_address: None,
                evidence_summary: format!(
                    "Previously confirmed merge can be split: {}",
                    row.try_get::<String, _>("evidence_summary")?
                ),
                confidence: 1.0,
            };
            upsert_candidate(
                &self.pool,
                &candidate,
                candidate.identity_candidate_id(),
                ContactIdentityReviewState::Suggested,
            )
            .await?;
            count += 1;
        }
```

- [ ] **Step 4: Run the split candidate test to verify GREEN**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test contact_identity contact_identity_refresh_creates_split_candidate_for_confirmed_merge_against_postgres -- --nocapture --test-threads=1
```

Expected: PASS when `HERMES_TEST_DATABASE_URL` is set.

- [ ] **Step 5: Add failing store coverage for active merge suppression by confirmed split**

Append this test to `backend/tests/contact_identity.rs`:

```rust
#[tokio::test]
async fn contact_identity_confirmed_split_removes_merge_from_detail_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity split detail test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Split Detail {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("split.detail.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("split.detail.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    context.store.refresh_candidates(100).await.expect("refresh");
    let merge_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);
    context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: format!("identity-confirm-before-split-{suffix}"),
            identity_candidate_id: merge_candidate_id.clone(),
            review_state: ContactIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge");

    context.store.refresh_candidates(100).await.expect("refresh split");
    let split_candidate_id = merge_candidate_id.replacen("merge_contacts", "split_contact", 1);
    context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: format!("identity-split-{suffix}"),
            identity_candidate_id: split_candidate_id,
            review_state: ContactIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm split");

    let detail = context
        .store
        .contact_identity(&left.contact_id)
        .await
        .expect("identity detail");
    assert!(
        detail.items.is_empty(),
        "confirmed split must suppress the previously confirmed merge link"
    );
}
```

- [ ] **Step 6: Run the split suppression test to verify RED**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test contact_identity contact_identity_confirmed_split_removes_merge_from_detail_against_postgres -- --nocapture --test-threads=1
```

Expected: FAIL because `contact_identity` still returns the confirmed merge link.

- [ ] **Step 7: Exclude confirmed split pairs from active contact identity detail**

In `backend/src/contact_identity.rs`, replace the SQL inside `contact_identity` with:

```rust
        let rows = sqlx::query(
            r#"
            SELECT
                merge.identity_candidate_id,
                merge.candidate_kind,
                merge.left_contact_id,
                merge.right_contact_id,
                merge.email_address,
                merge.evidence_summary,
                merge.confidence,
                merge.review_state,
                merge.generated_at,
                merge.reviewed_at,
                merge.updated_at
            FROM contact_identity_candidates merge
            WHERE (merge.left_contact_id = $1 OR merge.right_contact_id = $1)
              AND merge.candidate_kind = 'merge_contacts'
              AND merge.review_state = 'user_confirmed'
              AND NOT EXISTS (
                  SELECT 1
                  FROM contact_identity_candidates split
                  WHERE split.candidate_kind = 'split_contact'
                    AND split.review_state = 'user_confirmed'
                    AND split.left_contact_id = merge.left_contact_id
                    AND split.right_contact_id = merge.right_contact_id
              )
            ORDER BY merge.updated_at DESC, merge.identity_candidate_id
            "#,
        )
```

- [ ] **Step 8: Run contact identity store tests**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test contact_identity -- --nocapture --test-threads=1
```

Expected: PASS when `HERMES_TEST_DATABASE_URL` is set.

- [ ] **Step 9: Add API coverage for split candidates**

In `backend/tests/contact_identity_api.rs`, add a test named `identity_candidates_returns_split_candidate_for_confirmed_merge`. Use the existing router/test helper style and assert:

```rust
assert_eq!(split_item["candidate_kind"], "split_contact");
assert_eq!(split_item["review_state"], "suggested");
assert!(split_item["evidence_summary"].as_str().unwrap().contains("Previously confirmed merge"));
```

- [ ] **Step 10: Run contact identity API tests**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test contact_identity_api -- --nocapture --test-threads=1
```

Expected: PASS when `HERMES_TEST_DATABASE_URL` is set.

- [ ] **Step 11: Update frontend API types for split-aware identity**

In `frontend/src/lib/api.ts`, verify `ContactIdentityCandidate` already includes:

```ts
candidate_kind: 'merge_contacts' | 'attach_email_address' | 'split_contact';
review_state: 'suggested' | 'user_confirmed' | 'user_rejected';
```

If those exact unions are present, do not change them. If they differ, replace them with the snippet above.

- [ ] **Step 12: Add split controls to the Contacts identity surface**

In `frontend/src/routes/+page.svelte`, locate the Contacts identity review surface. For confirmed merge rows, render a reset/split action that sends `review_state: 'user_confirmed'` to the paired `split_contact` candidate when that candidate is available. Use the existing review helper used by identity candidate controls and generate a command id with this shape:

```ts
const commandId = `contact-identity-split-${Date.now()}-${candidate.identity_candidate_id}`;
```

The button label must be:

```svelte
Split
```

The disabled title for unavailable split candidates must be:

```svelte
title="Refresh identity candidates to create a split review for this confirmed link"
```

- [ ] **Step 13: Run frontend static validation**

Run:

```sh
pnpm --dir frontend check
pnpm --dir frontend build
```

Expected: both commands pass.

- [ ] **Step 14: Commit contact identity closure**

Run:

```sh
git add backend/src/contact_identity.rs backend/tests/contact_identity.rs backend/tests/contact_identity_api.rs frontend/src/lib/api.ts frontend/src/routes/+page.svelte
git commit -m "feat: close contact identity split review"
```

---

### Task 3: Document Processing Retry Command

**Files:**
- Modify: `backend/src/document_processing.rs`
- Modify: `backend/src/audit.rs`
- Modify: `backend/src/lib.rs`
- Modify: `backend/tests/document_processing.rs`
- Modify: `backend/tests/document_processing_api.rs`
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Add failing store coverage for retrying failed jobs**

Append this test to `backend/tests/document_processing.rs`:

```rust
#[tokio::test]
async fn document_processing_retry_failed_job_requeues_job_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live document processing retry test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = document_processing_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_{suffix}");

    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry doc",
            "# Retry\nbody",
        ))
        .await
        .expect("import document");
    let jobs = context
        .store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue");
    let job_id = jobs
        .iter()
        .find(|job| job.step == DocumentProcessingStep::ExtractText)
        .expect("extract job")
        .job_id
        .clone();

    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 1,
            last_error_summary = 'safe retry failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(&job_id)
    .execute(&context.pool)
    .await
    .expect("mark failed");

    let result = context
        .store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("doc-processing-retry-{suffix}"),
            job_id: job_id.clone(),
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("retry failed job");

    assert_eq!(result.job_id, job_id);
    assert_eq!(result.status, DocumentProcessingStatus::Queued);
    assert!(result.event_id.starts_with("document_processing_retry:"));

    let row: (String, i32, Option<String>) = sqlx::query_as(
        "SELECT status, attempts, last_error_summary FROM document_processing_jobs WHERE job_id = $1",
    )
    .bind(&job_id)
    .fetch_one(&context.pool)
    .await
    .expect("load retried job");
    assert_eq!(row.0, "queued");
    assert_eq!(row.1, 0);
    assert_eq!(row.2, None);
}
```

- [ ] **Step 2: Run the store retry test to verify RED**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test document_processing document_processing_retry_failed_job_requeues_job_against_postgres -- --nocapture --test-threads=1
```

Expected: FAIL with unresolved `DocumentProcessingRetryCommand` or `retry_failed_job`.

- [ ] **Step 3: Add retry command types and event constants**

In `backend/src/document_processing.rs`, add these items near the other public structs:

```rust
const DOCUMENT_PROCESSING_RETRY_EVENT_TYPE: &str = "document_processing.retry_requested";
const DOCUMENT_PROCESSING_RETRY_EVENT_PREFIX: &str = "document_processing_retry:";
const DOCUMENT_PROCESSING_RETRY_SOURCE_KIND: &str = "document_processing_retry";
const DOCUMENT_PROCESSING_RETRY_SOURCE_PROVIDER: &str = "local_api";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentProcessingRetryCommand {
    pub command_id: String,
    pub job_id: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingRetryCommandResult {
    pub job_id: String,
    pub status: DocumentProcessingStatus,
    pub event_id: String,
}
```

- [ ] **Step 4: Implement retry_failed_job**

In `backend/src/document_processing.rs`, add this public method inside `impl DocumentProcessingStore`:

```rust
    pub async fn retry_failed_job(
        &self,
        command: &DocumentProcessingRetryCommand,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let job_id = validate_non_empty("job_id", &command.job_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;
        let event_id = format!("{DOCUMENT_PROCESSING_RETRY_EVENT_PREFIX}{command_id}");
        let occurred_at = Utc::now();

        let mut transaction = self.pool.begin().await?;
        let current_status: String = sqlx::query_scalar(
            "SELECT status FROM document_processing_jobs WHERE job_id = $1",
        )
        .bind(&job_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(DocumentProcessingError::JobNotFound)?;

        if current_status != DocumentProcessingStatus::Failed.as_str() {
            return Err(DocumentProcessingError::RetryRequiresFailedJob);
        }

        let event = NewEventEnvelope::builder(
            event_id.clone(),
            DOCUMENT_PROCESSING_RETRY_EVENT_TYPE,
            occurred_at,
            json!({
                "kind": DOCUMENT_PROCESSING_RETRY_SOURCE_KIND,
                "provider": DOCUMENT_PROCESSING_RETRY_SOURCE_PROVIDER,
                "source_id": command_id,
            }),
            json!({
                "kind": "document_processing_retry",
            }),
        )
        .actor(json!({ "actor_id": actor_id }))
        .payload(json!({ "job_id": job_id }))
        .build()?;

        crate::event_log::EventStore::append_in_transaction(&mut transaction, &event).await?;

        sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = 'queued',
                attempts = 0,
                last_error_summary = NULL,
                started_at = NULL,
                finished_at = NULL,
                updated_at = now()
            WHERE job_id = $1
            "#,
        )
        .bind(&job_id)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(DocumentProcessingRetryCommandResult {
            job_id,
            status: DocumentProcessingStatus::Queued,
            event_id,
        })
    }
```

- [ ] **Step 5: Add retry errors**

In `backend/src/document_processing.rs`, keep the existing `JobNotFound` variant and add these variants to `DocumentProcessingError`:

```rust
#[error("document processing retry requires a failed job")]
RetryRequiresFailedJob,

#[error(transparent)]
EventEnvelope(#[from] crate::event_log::EventEnvelopeError),

#[error(transparent)]
EventStore(#[from] crate::event_log::EventStoreError),
```

- [ ] **Step 6: Run the store retry test to verify GREEN**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test document_processing document_processing_retry_failed_job_requeues_job_against_postgres -- --nocapture --test-threads=1
```

Expected: PASS when `HERMES_TEST_DATABASE_URL` is set.

- [ ] **Step 7: Add failing API coverage for protected retry**

Append this test to `backend/tests/document_processing_api.rs`:

```rust
#[tokio::test]
async fn post_document_processing_job_retry_requires_actor_and_requeues_failed_job() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live document processing retry API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let document_id = format!("doc_processing_retry_api_{:x}", unique_suffix());

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry api doc",
            "# Retry API\nbody",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let job_id = jobs[0].job_id.clone();
    sqlx::query(
        "UPDATE document_processing_jobs SET status = 'failed', attempts = 1, last_error_summary = 'safe failure', finished_at = now() WHERE job_id = $1",
    )
    .bind(&job_id)
    .execute(&pool)
    .await
    .expect("mark failed");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let command_id = format!("document-processing-retry-api-{:x}", unique_suffix());
    let missing_actor = app
        .clone()
        .oneshot(json_post_request_with_token(
            &format!("/api/v2/document-processing/jobs/{job_id}/retry"),
            serde_json::json!({ "command_id": command_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("missing actor response");
    assert_eq!(missing_actor.status(), StatusCode::BAD_REQUEST);

    let response = app
        .oneshot(json_post_request_with_actor(
            &format!("/api/v2/document-processing/jobs/{job_id}/retry"),
            serde_json::json!({ "command_id": command_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("retry response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["job_id"], Value::String(job_id.clone()));
    assert_eq!(body["status"], Value::String("queued".to_owned()));
    assert_eq!(
        body["event_id"],
        Value::String(format!("document_processing_retry:{command_id}"))
    );
}
```

Also add request helpers in the same file:

```rust
fn json_post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn json_post_request_with_actor(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}
```

- [ ] **Step 8: Run API retry test to verify RED**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test document_processing_api post_document_processing_job_retry_requires_actor_and_requeues_failed_job -- --nocapture --test-threads=1
```

Expected: FAIL because the retry route is not registered.

- [ ] **Step 9: Add audit helper for retry**

In `backend/src/audit.rs`, add:

```rust
    pub fn document_processing_job_retry(
        actor_id: impl Into<String>,
        job_id: impl Into<String>,
    ) -> Self {
        let job_id = job_id.into();
        Self {
            actor_kind: LOCAL_API_TOKEN_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "document_processing.job.retry".to_owned(),
            method: "POST".to_owned(),
            path_template: "/api/v2/document-processing/jobs/{job_id}/retry".to_owned(),
            target_kind: "document_processing_job".to_owned(),
            target_id: Some(job_id),
            metadata: json!({}),
        }
    }
```

- [ ] **Step 10: Add retry API route and request/response types**

In `backend/src/lib.rs`, add the route next to the existing document processing routes:

```rust
        .route(
            "/api/v2/document-processing/jobs/{job_id}/retry",
            post(post_document_processing_job_retry),
        )
```

Add these API structs near the other V2 API request/response structs:

```rust
#[derive(Debug, Deserialize)]
struct DocumentProcessingRetryApiRequest {
    command_id: String,
}

#[derive(Debug, Serialize)]
struct DocumentProcessingRetryApiResponse {
    job_id: String,
    status: String,
    event_id: String,
}

impl DocumentProcessingRetryApiRequest {
    fn into_command(
        self,
        job_id: String,
        actor_id: String,
    ) -> Result<DocumentProcessingRetryCommand, ApiError> {
        let command_id = validate_non_empty_document_id(&self.command_id)?;
        let job_id = validate_non_empty_document_id(&job_id)?;
        let actor_id = validate_non_empty_actor_id(&actor_id)?;
        Ok(DocumentProcessingRetryCommand {
            command_id,
            job_id,
            actor_id,
        })
    }
}

impl From<DocumentProcessingRetryCommandResult> for DocumentProcessingRetryApiResponse {
    fn from(result: DocumentProcessingRetryCommandResult) -> Self {
        Self {
            job_id: result.job_id,
            status: result.status.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}
```

Add the handler:

```rust
async fn post_document_processing_job_retry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(request): Json<DocumentProcessingRetryApiRequest>,
) -> Result<Json<DocumentProcessingRetryApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let command = request.into_command(job_id, actor.actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::document_processing_job_retry(
            &command.actor_id,
            &command.job_id,
        ))
        .await?;

    let result = document_processing_store(&state)?
        .retry_failed_job(&command)
        .await?;

    Ok(Json(result.into()))
}
```

- [ ] **Step 11: Map retry errors to stable API errors**

In the `ApiError` conversion for `DocumentProcessingError`, keep the existing `JobNotFound` mapping and add `RetryRequiresFailedJob` inside the `Self::DocumentProcessing(error)` match:

```rust
DocumentProcessingError::RetryRequiresFailedJob => {
    (StatusCode::BAD_REQUEST, "document processing retry requires a failed job")
}
```

- [ ] **Step 12: Run document processing API tests**

Run:

```sh
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test document_processing_api -- --nocapture --test-threads=1
```

Expected: PASS when `HERMES_TEST_DATABASE_URL` is set.

- [ ] **Step 13: Add frontend retry helper**

In `frontend/src/lib/api.ts`, add:

```ts
export type DocumentProcessingRetryRequest = {
	command_id: string;
};

export type DocumentProcessingRetryResponse = {
	job_id: string;
	status: DocumentProcessingStatus;
	event_id: string;
};

export async function retryDocumentProcessingJob(
	baseUrl: string,
	token: string,
	actorId: string,
	jobId: string,
	request: DocumentProcessingRetryRequest
): Promise<DocumentProcessingRetryResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/document-processing/jobs/${encodeURIComponent(jobId)}/retry`,
		request,
		'Document processing retry request failed'
	);
}
```

- [ ] **Step 14: Render retry button for failed document processing jobs**

In `frontend/src/routes/+page.svelte`, import `retryDocumentProcessingJob`. In the document processing surface, render a `Retry` button only for jobs where `job.status === 'failed'`. The button must call:

```ts
await retryDocumentProcessingJob(apiBaseUrl, apiToken, actorId, job.job_id, {
	command_id: `document-processing-retry-${Date.now()}-${job.job_id}`
});
```

After success, reload the document processing jobs and the selected document processing detail if one is selected.

- [ ] **Step 15: Run validation for document retry**

Run:

```sh
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
make db-up
set -a; . docker/.env; set +a
HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test document_processing --test document_processing_api -- --nocapture --test-threads=1
pnpm --dir frontend check
pnpm --dir frontend build
git diff --check
```

Expected: all commands pass when `HERMES_TEST_DATABASE_URL` is set.

- [ ] **Step 16: Commit document processing retry**

Run:

```sh
git add backend/src/document_processing.rs backend/src/audit.rs backend/src/lib.rs backend/tests/document_processing.rs backend/tests/document_processing_api.rs frontend/src/lib/api.ts frontend/src/routes/+page.svelte
git commit -m "feat: add document processing retry command"
```

---

### Task 4: V2 Workflow Live Smoke Gate

**Files:**
- Modify: `Makefile`
- Modify: `docs/roadmap/v2-closure-checklist.md`

- [ ] **Step 1: Add `backend-v2-workflow-smoke-dev` to Makefile PHONY**

In `Makefile`, add `backend-v2-workflow-smoke-dev` to the `.PHONY` list.

- [ ] **Step 2: Add help text**

In the `help` target, add:

```make
	@printf '%s\n' '  make backend-v2-workflow-smoke-dev Run V2 workflow smoke tests with dev PostgreSQL'
```

- [ ] **Step 3: Add the smoke target**

Add this Makefile target after `backend-graph-smoke-dev`:

```make
backend-v2-workflow-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) \
			--test projects \
			--test projects_api \
			--test project_link_reviews \
			--test task_candidates \
			--test task_candidates_api \
			--test contact_identity \
			--test contact_identity_api \
			--test document_processing \
			--test document_processing_api \
			-- --nocapture --test-threads=1
```

- [ ] **Step 4: Include V2 workflow smoke in full validation**

In the `validate:` dependency list, insert `backend-v2-workflow-smoke-dev` after `backend-graph-smoke-dev`.

- [ ] **Step 5: Run the new smoke target**

Run:

```sh
make backend-v2-workflow-smoke-dev
```

Expected: Docker Compose PostgreSQL starts, the listed tests pass, and PostgreSQL is stopped by the target cleanup.

- [ ] **Step 6: Mark the smoke gate in the V2 closure checklist**

In `docs/roadmap/v2-closure-checklist.md`, change:

```markdown
- [ ] `make validate` includes live PostgreSQL smoke coverage for V2 workflow APIs.
```

to:

```markdown
- [x] `make validate` includes live PostgreSQL smoke coverage for V2 workflow APIs.
```

- [ ] **Step 7: Validate Makefile and checklist references**

Run:

```sh
make -n backend-v2-workflow-smoke-dev >/tmp/hermes-v2-workflow-smoke.make
rg -n "backend-v2-workflow-smoke-dev|V2 workflow smoke" Makefile
rg -n "live PostgreSQL smoke coverage" docs/roadmap/v2-closure-checklist.md
git diff --check
```

Expected: all commands pass.

- [ ] **Step 8: Commit smoke gate**

Run:

```sh
git add Makefile docs/roadmap/v2-closure-checklist.md
git commit -m "build: add v2 workflow smoke gate"
```

---

### Task 5: Full V2 Release Validation And Checklist Closure

**Files:**
- Modify: `docs/roadmap/v2-closure-checklist.md`

- [ ] **Step 1: Run backend validation**

Run:

```sh
make backend-validate
```

Expected: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml` pass.

- [ ] **Step 2: Run frontend validation**

Run:

```sh
make frontend-check
make frontend-build
```

Expected: Svelte diagnostics report `0 errors and 0 warnings`, and Vite/SvelteKit build succeeds.

- [ ] **Step 3: Run full validation gate**

Run:

```sh
make validate
```

Expected: full Docker Compose config, backend validation, live PostgreSQL smoke targets, V2 workflow smoke target, backend HTTP smoke, frontend check and frontend build all pass.

- [ ] **Step 4: Run desktop browser smoke**

Run:

```sh
make dev
```

Open the printed frontend URL, usually:

```text
http://127.0.0.1:5174
```

Smoke the desktop shell only:

- Knowledge Graph tab loads summary, search/picker and neighborhood states.
- Projects tab loads project list/detail and project link review controls.
- Tasks tab loads task candidates and active tasks.
- Contacts tab shows identity review data and split action for confirmed links when available.
- Documents or processing surface shows processing job states and retry only for failed jobs.

Stop `make dev` with `Ctrl+C`.

- [ ] **Step 5: Update final V2 checklist gates**

In `docs/roadmap/v2-closure-checklist.md`, change these gates to checked only after the matching validation has passed:

```markdown
- [x] Contact identity supports explicit split review for confirmed merge links.
- [x] Document processing failed jobs can be retried through a protected event-backed command.
- [x] Backend README documents all V2 workflow APIs and dev commands.
- [x] Frontend README documents V2 desktop surfaces and validation commands.
- [x] Full `make validate` passes from a clean checkout with Docker available.
- [x] Desktop browser smoke validates graph, projects, tasks, contacts and document-processing surfaces.
```

- [ ] **Step 6: Verify no unchecked V2 acceptance gates remain**

Run:

```sh
rg -n "^- \\[ \\]" docs/roadmap/v2-closure-checklist.md
```

Expected: no output.

- [ ] **Step 7: Verify working tree and diff hygiene**

Run:

```sh
git status --short
git diff --check
```

Expected: `git status --short` shows only the intended checklist change, and `git diff --check` passes.

- [ ] **Step 8: Commit V2 closure**

Run:

```sh
git add docs/roadmap/v2-closure-checklist.md
git commit -m "docs: close v2 release checklist"
```

---

## Self-Review

Spec coverage:

- V2 graph core is covered by existing checked gates and carried into the full closure checklist.
- Project timelines and project link review are covered by existing checked gates and V2 workflow smoke.
- Task candidates are covered by existing checked gates and V2 workflow smoke.
- Contact merge/split is covered by Task 2 through explicit non-destructive split review for confirmed merge links.
- Document extraction/OCR state is covered by existing processing jobs/artifacts, and Task 3 adds protected retry for failed jobs.
- Full validation is covered by Task 4 and Task 5.

Placeholder scan:

- No deferred-work markers or undefined acceptance gates are used.

Type consistency:

- Contact identity uses existing `ContactIdentityCandidateKind::SplitContact` and existing `review_state` values.
- Document processing retry uses `DocumentProcessingStatus::Queued`, existing local API auth, canonical events and audit logging.
- Frontend type names mirror backend response names and existing `DocumentProcessingStatus`.
