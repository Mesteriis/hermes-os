# Contact Identity Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit review for possible contact identity merges without mutating contacts automatically.

**Architecture:** Generate conservative identity candidates from existing contacts and store user decisions as canonical `contact_identity.review_state_changed` events. Keep contacts immutable for this slice; confirmed decisions live in a replayable identity review model and are exposed through protected read APIs and a compact desktop review surface.

**Tech Stack:** Rust 1.85/edition 2024, Axum, SQLx/PostgreSQL migrations, existing event log/audit patterns, SvelteKit 2, Svelte 5 runes, TypeScript, pnpm, Make.

---

## Source Spec

- `docs/superpowers/specs/2026-06-05-v2-memory-workflow-completion-design.md`

## Relevant ADRs

- `docs/adr/ADR-0001-event-sourcing-as-system-spine.md`
- `docs/adr/ADR-0015-command-query-separation.md`
- `docs/adr/ADR-0019-contact-identity-resolution.md`
- `docs/adr/ADR-0023-rebuildable-projections.md`
- `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md`
- `docs/adr/ADR-0038-local-event-api-capability-token.md`
- `docs/adr/ADR-0040-local-api-actor-identity.md`

## File Map

- Create: `backend/migrations/0016_create_contact_identity_reviews.sql`
- Create: `backend/src/contact_identity.rs`
- Modify: `backend/src/audit.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/contact_identity.rs`
- Create: `backend/tests/contact_identity_api.rs`
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

## Data Contracts

Backend domain values:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactIdentityCandidateKind {
    MergeContacts,
    AttachEmailAddress,
    SplitContact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactIdentityReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

pub struct ContactIdentityReviewCommand {
    pub command_id: String,
    pub identity_candidate_id: String,
    pub review_state: ContactIdentityReviewState,
    pub actor_id: String,
}
```

Frontend request shape:

```ts
export type ContactIdentityReviewRequest = {
	command_id: string;
	review_state: 'suggested' | 'user_confirmed' | 'user_rejected';
};
```

## Assumptions

Assumption: This slice does not collapse rows in `contacts`.
Reason: Existing contacts are deterministic email-derived records, and `ADR-0019` disallows ambiguous automatic identity collapse.
Risk: Confirmed identity links improve review/read behavior first; destructive merge/split editing remains a later command.

Assumption: The first candidate generator emits `merge_contacts` candidates only.
Reason: Existing schema has contacts with one primary email each and no independent email-address table outside graph projections.
Risk: `attach_email_address` and `split_contact` are represented in schema/API enums but are not generated until the contact model grows.

---

## Task 1: Schema

**Files:**
- Create: `backend/migrations/0016_create_contact_identity_reviews.sql`

- [ ] **Step 1: Add migration**

Create `backend/migrations/0016_create_contact_identity_reviews.sql`:

```sql
CREATE TABLE IF NOT EXISTS contact_identity_candidates (
    identity_candidate_id TEXT PRIMARY KEY,
    candidate_kind TEXT NOT NULL,
    left_contact_id TEXT NOT NULL REFERENCES contacts(contact_id) ON DELETE CASCADE,
    right_contact_id TEXT REFERENCES contacts(contact_id) ON DELETE CASCADE,
    email_address TEXT,
    evidence_summary TEXT NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    event_id TEXT,
    actor_id TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT contact_identity_candidate_kind_check
        CHECK (candidate_kind IN ('merge_contacts', 'attach_email_address', 'split_contact')),
    CONSTRAINT contact_identity_review_state_check
        CHECK (review_state IN ('suggested', 'user_confirmed', 'user_rejected')),
    CONSTRAINT contact_identity_confidence_check
        CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT contact_identity_candidate_id_not_empty
        CHECK (length(trim(identity_candidate_id)) > 0),
    CONSTRAINT contact_identity_left_contact_not_empty
        CHECK (length(trim(left_contact_id)) > 0),
    CONSTRAINT contact_identity_evidence_not_empty
        CHECK (length(trim(evidence_summary)) > 0),
    CONSTRAINT contact_identity_merge_has_right_contact
        CHECK (candidate_kind <> 'merge_contacts' OR right_contact_id IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS contact_identity_merge_pair_idx
    ON contact_identity_candidates (
        candidate_kind,
        LEAST(left_contact_id, COALESCE(right_contact_id, left_contact_id)),
        GREATEST(left_contact_id, COALESCE(right_contact_id, left_contact_id))
    )
    WHERE candidate_kind = 'merge_contacts';

CREATE INDEX IF NOT EXISTS contact_identity_review_state_idx
    ON contact_identity_candidates (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS contact_identity_left_contact_idx
    ON contact_identity_candidates (left_contact_id);

CREATE INDEX IF NOT EXISTS contact_identity_right_contact_idx
    ON contact_identity_candidates (right_contact_id);
```

- [ ] **Step 2: Run migration test command to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test contact_identity -- --nocapture --test-threads=1
```

Expected: FAIL because `backend/tests/contact_identity.rs` does not exist yet.

- [ ] **Step 3: Commit schema**

Run:

```sh
git add backend/migrations/0016_create_contact_identity_reviews.sql
git commit -m "feat: add contact identity review schema"
```

## Task 2: Store And Candidate Generation

**Files:**
- Create: `backend/src/contact_identity.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/contact_identity.rs`

- [ ] **Step 1: Write failing store tests**

Create `backend/tests/contact_identity.rs` with tests named:

```rust
#[tokio::test]
async fn contact_identity_refresh_creates_conservative_merge_candidate_against_postgres() {}

#[tokio::test]
async fn contact_identity_confirm_records_review_without_mutating_contacts_against_postgres() {}

#[tokio::test]
async fn contact_identity_reject_suppresses_candidate_against_postgres() {}

#[tokio::test]
async fn contact_identity_review_event_rebuilds_state_against_postgres() {}
```

Seed contacts through `ContactProjectionStore::upsert_email_contact`, then update `contacts.display_name` in the test setup with SQL so two distinct contacts share a normalized display name. Assert that the candidate exists but contact rows remain distinct after confirmation.

- [ ] **Step 2: Run tests to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test contact_identity -- --nocapture --test-threads=1
```

Expected: FAIL with unresolved import `hermes_hub_backend::contact_identity`.

- [ ] **Step 3: Add module export**

Add this line near existing public modules in `backend/src/lib.rs`:

```rust
pub mod contact_identity;
```

- [ ] **Step 4: Create contact identity store**

Create `backend/src/contact_identity.rs` with these public items:

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::event_log::{EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope};

const CONTACT_IDENTITY_REVIEW_EVENT_TYPE: &str = "contact_identity.review_state_changed";
const CONTACT_IDENTITY_REVIEW_SOURCE_KIND: &str = "contact_identity_review";
const CONTACT_IDENTITY_REVIEW_SOURCE_PROVIDER: &str = "local_api";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ContactIdentityCandidateKind {
    MergeContacts,
    AttachEmailAddress,
    SplitContact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ContactIdentityReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactIdentityReviewCommand {
    pub command_id: String,
    pub identity_candidate_id: String,
    pub review_state: ContactIdentityReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ContactIdentityCandidate {
    pub identity_candidate_id: String,
    pub candidate_kind: String,
    pub left_contact_id: String,
    pub right_contact_id: Option<String>,
    pub email_address: Option<String>,
    pub evidence_summary: String,
    pub confidence: f64,
    pub review_state: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ContactIdentityStore {
    pool: PgPool,
}

impl ContactIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

- [ ] **Step 5: Implement deterministic refresh**

Add:

```rust
pub async fn refresh_candidates(&self, limit: i64) -> Result<usize, ContactIdentityError>
```

Behavior:

- read contact pairs with the same normalized `display_name`;
- require different `contact_id` values;
- generate only `merge_contacts`;
- ignore display names that contain `@` to avoid merging default email-as-name contacts;
- set confidence to `0.72`;
- evidence summary is `Same normalized display name: {display_name}`;
- deterministic ID is `identity_candidate:v1:merge_contacts:{left_contact_id}:{right_contact_id}` with pair order normalized lexicographically;
- do not overwrite `user_confirmed` or `user_rejected` review state.

- [ ] **Step 6: Implement review command**

Add:

```rust
pub async fn set_review_state(
    &self,
    command: &ContactIdentityReviewCommand,
) -> Result<ContactIdentityReviewResult, ContactIdentityError>
```

It must:

- validate non-empty `command_id`, `identity_candidate_id` and `actor_id`;
- ensure candidate exists;
- append event ID `contact_identity_review:{command_id}`;
- update candidate review state transactionally;
- never update or delete rows in `contacts`.

- [ ] **Step 7: Implement replay and reads**

Add:

```rust
pub async fn apply_review_event(&self, event: &EventEnvelope) -> Result<(), ContactIdentityError>
pub async fn list_candidates(&self, limit: Option<i64>) -> Result<Vec<ContactIdentityCandidate>, ContactIdentityError>
pub async fn contact_identity(&self, contact_id: &str) -> Result<ContactIdentityDetail, ContactIdentityError>
```

`contact_identity` must return confirmed candidates where `contact_id` is either side of a merge candidate.

- [ ] **Step 8: Run store tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test contact_identity -- --nocapture --test-threads=1
```

Expected: PASS or live-test skip when `HERMES_TEST_DATABASE_URL` is unset.

- [ ] **Step 9: Commit store**

Run:

```sh
git add backend/src/contact_identity.rs backend/src/lib.rs backend/tests/contact_identity.rs
git commit -m "feat: add contact identity review store"
```

## Task 3: Protected API

**Files:**
- Modify: `backend/src/audit.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/contact_identity_api.rs`

- [ ] **Step 1: Write failing API tests**

Create `backend/tests/contact_identity_api.rs` with tests named:

```rust
#[tokio::test]
async fn identity_candidates_reject_missing_local_api_token() {}

#[tokio::test]
async fn identity_candidates_returns_safe_candidate_payload() {}

#[tokio::test]
async fn put_identity_candidate_review_requires_actor_and_confirms_candidate() {}

#[tokio::test]
async fn contact_identity_returns_confirmed_links_for_contact() {}
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test contact_identity_api -- --nocapture --test-threads=1
```

Expected: FAIL because `/api/v2/identity-candidates` is not routed.

- [ ] **Step 3: Add audit helper**

In `backend/src/audit.rs`, add:

```rust
pub fn contact_identity_review_set(
    actor_id: impl Into<String>,
    identity_candidate_id: impl Into<String>,
) -> Self {
    let identity_candidate_id = identity_candidate_id.into();
    Self {
        actor_kind: LOCAL_API_TOKEN_ACTOR_KIND.to_owned(),
        actor_id: actor_id.into(),
        operation: "contact_identity.review.set".to_owned(),
        method: "PUT".to_owned(),
        path_template: "/api/v2/identity-candidates/{identity_candidate_id}/review".to_owned(),
        target_kind: "contact_identity_candidate".to_owned(),
        target_id: Some(identity_candidate_id),
        metadata: json!({}),
    }
}
```

- [ ] **Step 4: Add routes**

In `build_router_with_database`, add:

```rust
.route("/api/v2/identity-candidates", get(get_identity_candidates))
.route(
    "/api/v2/identity-candidates/{identity_candidate_id}/review",
    put(put_identity_candidate_review),
)
.route("/api/v2/contacts/{contact_id}/identity", get(get_contact_identity))
```

- [ ] **Step 5: Add handlers**

Add handlers that:

- verify local API token for all reads;
- verify actor ID for review command;
- parse `limit` with the existing query parsing style;
- call `ContactIdentityStore`;
- return stable error codes.

Use these error codes:

```text
invalid_contact_identity_review
contact_identity_candidate_not_found
contact_identity_store_error
```

- [ ] **Step 6: Run API tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test contact_identity_api -- --nocapture --test-threads=1
```

Expected: PASS or live-test skip when `HERMES_TEST_DATABASE_URL` is unset.

- [ ] **Step 7: Commit API**

Run:

```sh
git add backend/src/audit.rs backend/src/lib.rs backend/tests/contact_identity_api.rs
git commit -m "feat: expose contact identity review APIs"
```

## Task 4: Frontend Review Surface

**Files:**
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Add frontend API types**

In `frontend/src/lib/api.ts`, add:

```ts
export type ContactIdentityReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export type ContactIdentityCandidate = {
	identity_candidate_id: string;
	candidate_kind: 'merge_contacts' | 'attach_email_address' | 'split_contact';
	left_contact_id: string;
	right_contact_id: string | null;
	email_address: string | null;
	evidence_summary: string;
	confidence: number;
	review_state: ContactIdentityReviewState;
	generated_at: string;
	reviewed_at: string | null;
	updated_at: string;
};

export type ContactIdentityCandidateListResponse = {
	items: ContactIdentityCandidate[];
};
```

- [ ] **Step 2: Add frontend API functions**

Add:

```ts
export async function fetchIdentityCandidates(apiBaseUrl: string, token: string, actorId: string, limit = 50) {
	return fetchJson<ContactIdentityCandidateListResponse>(
		`${apiBaseUrl}/api/v2/identity-candidates?limit=${limit}`,
		token,
		actorId
	);
}

export async function reviewIdentityCandidate(
	apiBaseUrl: string,
	token: string,
	actorId: string,
	identityCandidateId: string,
	reviewState: ContactIdentityReviewState
) {
	return fetchJson(
		`${apiBaseUrl}/api/v2/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
		token,
		actorId,
		{
			method: 'PUT',
			body: JSON.stringify({
				command_id: `contact-identity-review-${crypto.randomUUID()}`,
				review_state: reviewState
			})
		}
	);
}
```

- [ ] **Step 3: Add Svelte state**

In `frontend/src/routes/+page.svelte`, add:

```svelte
let identityCandidates = $state<ContactIdentityCandidate[]>([]);
let identityCandidatesError = $state('');
let isIdentityCandidatesLoading = $state(false);

const suggestedIdentityCandidates = $derived(
	identityCandidates.filter((item) => item.review_state === 'suggested')
);
```

- [ ] **Step 4: Load identity candidates**

Add:

```svelte
async function loadIdentityCandidates() {
	isIdentityCandidatesLoading = true;
	try {
		const response = await fetchIdentityCandidates(apiBaseUrl, apiToken, actorId, 50);
		identityCandidates = response.items;
		identityCandidatesError = '';
	} catch (error) {
		identityCandidatesError =
			error instanceof Error ? error.message : 'Unknown identity candidate error';
	} finally {
		isIdentityCandidatesLoading = false;
	}
}
```

Call it from `onMount`.

- [ ] **Step 5: Add review action**

Add:

```svelte
async function setIdentityCandidateReview(
	candidate: ContactIdentityCandidate,
	reviewState: ContactIdentityReviewState
) {
	try {
		await reviewIdentityCandidate(
			apiBaseUrl,
			apiToken,
			actorId,
			candidate.identity_candidate_id,
			reviewState
		);
		await loadIdentityCandidates();
	} catch (error) {
		identityCandidatesError =
			error instanceof Error ? error.message : 'Unknown identity review error';
	}
}
```

- [ ] **Step 6: Render compact review queue**

Add a compact identity review panel to the Contacts view or Knowledge Graph detail column. It must show:

- candidate kind;
- left/right contact IDs;
- evidence summary;
- confidence;
- confirm/reject buttons;
- explicit copy that suggestions are not applied until confirmed.

Keep existing desktop panel styling and do not add mobile-specific work.

- [ ] **Step 7: Run frontend validation**

Run:

```sh
pnpm --dir frontend check
pnpm --dir frontend build
```

Expected: PASS.

- [ ] **Step 8: Commit frontend**

Run:

```sh
git add frontend/src/lib/api.ts frontend/src/routes/+page.svelte
git commit -m "feat: render contact identity review"
```

## Task 5: Final Validation

**Files:**
- No new files.

- [ ] **Step 1: Run targeted backend tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test contact_identity --test contact_identity_api -- --nocapture --test-threads=1
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

- [ ] Candidate generation is conservative.
- [ ] Confirming identity candidates does not mutate `contacts`.
- [ ] Review decisions append canonical events.
- [ ] Rejected candidates stay rejected until source evidence changes or the user resets state.
- [ ] Protected commands require bearer token and actor ID.
- [ ] UI states make it clear that suggestions are inactive until confirmed.

## Closure Status

Closed on 2026-06-05 as part of the V2 workflow slices.

Implemented:
- conservative contact identity candidate generation;
- canonical `contact_identity.review_state_changed` event recording and replay;
- review state storage that does not mutate canonical contacts;
- protected local APIs for identity candidates, review commands and contact identity detail;
- compact desktop review surface for identity suggestions.

Validated:
- `cargo test --manifest-path backend/Cargo.toml --test contact_identity --test contact_identity_api -- --nocapture`;
- `make backend-validate`;
- `pnpm --dir frontend check`;
- `pnpm --dir frontend build`.

Not run:
- full `make validate`, because that also runs Docker-backed smoke checks outside this slice;
- browser screenshot smoke, because Playwright was not available in `frontend/node_modules` in this session.
