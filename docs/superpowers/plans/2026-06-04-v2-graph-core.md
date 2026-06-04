# V2 Graph Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first V2 graph core: a deterministic, read-only PostgreSQL graph projection from contacts, communication messages and documents, with protected graph read APIs and desktop dashboard wiring.

**Architecture:** PostgreSQL remains the only persistence system. `graph_nodes`, `graph_edges` and `graph_evidence` are rebuildable projections, not source-of-truth records. Rust store/projection modules own graph writes and query shaping; Axum handlers expose read-only local-token APIs; SvelteKit consumes the APIs without adding mobile scope.

**Tech Stack:** Rust 1.85, Axum 0.8, SQLx 0.8, PostgreSQL JSONB, SvelteKit, TypeScript, pnpm, Docker Compose, Make.

---

## Source Spec

- `docs/superpowers/specs/2026-06-04-v2-graph-core-design.md`

## File Map

- Create: `docs/adr/ADR-0045-graph-core-projection.md` - decision record for graph projection storage, provenance and non-goals.
- Modify: `docs/adr/README.md` - ADR index entry for ADR-0045.
- Create: `docs/roadmap/v2-graph-core-checklist.md` - acceptance checklist for the first V2 graph slice.
- Modify: `docs/roadmap/product-roadmap.md` - mark Version 2.0 first slice as graph core.
- Modify: `README.md` - link V2 graph checklist.
- Create: `backend/migrations/0010_create_graph_core.sql` - graph schema and constraints.
- Modify: `backend/src/lib.rs` - expose modules and register `/api/v2/graph/*` routes.
- Create: `backend/src/graph.rs` - graph domain types, deterministic IDs, store writes and read queries.
- Create: `backend/src/graph_projection.rs` - V1-to-graph projection from contacts, messages and documents.
- Create: `backend/tests/graph.rs` - graph store tests.
- Create: `backend/tests/graph_projection.rs` - idempotent graph projection tests.
- Create: `backend/tests/graph_api.rs` - protected read API tests.
- Modify: `Makefile` - add `backend-graph-smoke-dev` and wire it into `validate`.
- Modify: `backend/README.md` - document graph smoke command.
- Modify: `frontend/src/lib/api.ts` - add V2 graph API types and fetch helpers.
- Modify: `frontend/src/routes/+page.svelte` - replace mock graph card data with API-backed state and disabled future controls.
- Modify: `frontend/README.md` - document graph API env dependency.

## Assumptions

Assumption: The first implementation uses existing SQLx dynamic queries rather than compile-time checked `query!` macros.
Reason: The current backend already uses dynamic `sqlx::query` and `Row` extraction.
Risk: SQL errors are caught by live PostgreSQL tests rather than compile-time SQLx metadata.

Assumption: Graph API requests reuse the current local bearer token and `X-Hermes-Actor-Id` header.
Reason: ADR-0038, ADR-0039 and ADR-0040 are already implemented in `backend/src/lib.rs`.
Risk: Full audit coverage for read-only graph endpoints can be added later if ADR-0039 is expanded beyond current event API audit cases.

---

## Task 1: ADR And V2 Checklist

**Files:**
- Create: `docs/adr/ADR-0045-graph-core-projection.md`
- Modify: `docs/adr/README.md`
- Create: `docs/roadmap/v2-graph-core-checklist.md`
- Modify: `docs/roadmap/product-roadmap.md`
- Modify: `README.md`

- [ ] **Step 1: Create ADR-0045**

Create `docs/adr/ADR-0045-graph-core-projection.md` with this content:

```markdown
# ADR-0045 Graph Core Projection

Status: Proposed

## Context

Version 2 starts by turning the Knowledge Graph into a real backend projection. Hermes Hub already has local PostgreSQL storage for contacts, communication messages and documents. ADR-0008 requires relationships to be durable records with provenance and confidence. ADR-0023 requires derived state to be rebuildable. ADR-0019 forbids ambiguous automatic identity merges. ADR-0031 keeps the UI desktop/laptop only.

## Decision

Use PostgreSQL relational graph tables for the first V2 graph core:

- `graph_nodes`
- `graph_edges`
- `graph_evidence`

The graph tables are a rebuildable projection, not source of truth. Source records remain in `contacts`, `communication_messages` and `documents`.

Initial node kinds:

- `person`
- `email_address`
- `message`
- `document`

Initial relationship types:

- `person_has_email_address`
- `person_sent_message`
- `person_received_message`
- `email_address_sent_message`
- `email_address_received_message`

System-created edges require evidence. The first projection only uses exact email matching to connect messages to contacts. When no exact contact exists, the graph uses an `email_address` node instead of inventing a person.

Read APIs are local-only, read-only and protected by the existing bearer token plus `X-Hermes-Actor-Id`.

## Non-Goals

- Separate graph database.
- GraphQL.
- Fuzzy person merge.
- Graph editing.
- OCR and entity extraction.
- Task candidate extraction.
- Mobile graph UI.

## Consequences

Positive:

- Graph data stays inspectable and rebuildable in PostgreSQL.
- Provenance is queryable without unpacking arbitrary edge JSON.
- The first V2 slice avoids false person merges.
- Existing Docker, SQLx and live PostgreSQL smoke tests remain enough for validation.

Negative:

- Graph traversal depth is intentionally limited in the first slice.
- Richer identity resolution requires a later reviewed merge/split workflow.
- Document-person and document-project edges wait for a later extraction engine.
```

- [ ] **Step 2: Link ADR-0045 in the ADR index**

In `docs/adr/README.md`, add this line after ADR-0044:

```markdown
- [ADR-0045 Graph Core Projection](ADR-0045-graph-core-projection.md)
```

- [ ] **Step 3: Create V2 graph checklist**

Create `docs/roadmap/v2-graph-core-checklist.md` with this content:

```markdown
# V2 Graph Core Checklist

## Release Goal

The first Version 2 slice is complete when Hermes Hub builds a deterministic, read-only Knowledge Graph projection from existing contacts, communication messages and documents, exposes protected read APIs, and renders graph-backed desktop dashboard data.

## In Scope

- PostgreSQL graph projection tables.
- Graph node, edge and evidence store.
- Deterministic graph IDs.
- Idempotent projection from `contacts`, `communication_messages` and `documents`.
- Exact-email identity linking only.
- Read-only graph summary, neighborhood and search APIs.
- Desktop dashboard graph summary and read-only explorer entry point.
- Live PostgreSQL graph smoke validation.

## Out of Scope

- Fuzzy identity merge.
- Contact merge/split UI.
- OCR.
- Entity extraction from document text.
- Task candidate extraction.
- AI summaries.
- Graph editing.
- Mobile graph UI.

## Acceptance Gate Status

- [ ] `backend/migrations/0010_create_graph_core.sql` creates graph tables and constraints.
- [ ] Graph node upserts are idempotent.
- [ ] Graph edge upserts are idempotent.
- [ ] System-created graph edges require evidence.
- [ ] V1 graph projection from contacts, messages and documents is idempotent.
- [ ] Exact email rules do not create fuzzy person merges.
- [ ] `GET /api/v2/graph/summary` has auth and response coverage.
- [ ] `GET /api/v2/graph/neighborhood` has auth, not-found, unsupported-depth and happy-path coverage.
- [ ] `GET /api/v2/graph/search` has auth, empty-query and happy-path coverage.
- [ ] `make backend-graph-smoke-dev` passes against live PostgreSQL.
- [ ] `make validate` includes the graph smoke target.
- [ ] `pnpm --dir frontend check` passes after graph UI wiring.
- [ ] `pnpm --dir frontend build` passes after graph UI wiring.
```

- [ ] **Step 4: Update roadmap and README links**

In `docs/roadmap/product-roadmap.md`, under `## Version 2.0 - Knowledge Graph and Documents`, add this first item under `Key functions:`:

```markdown
- first graph core projection from contacts, messages and documents
```

In `README.md`, add this link near the existing roadmap links:

```markdown
- [V2 Graph Core Checklist](docs/roadmap/v2-graph-core-checklist.md)
```

- [ ] **Step 5: Validate and commit docs**

Run:

```bash
git diff --check
git status --short
```

Expected:

- `git diff --check` exits 0.
- `git status --short` shows only the docs from this task.

Commit:

```bash
git add README.md docs/adr/README.md docs/adr/ADR-0045-graph-core-projection.md docs/roadmap/product-roadmap.md docs/roadmap/v2-graph-core-checklist.md
git commit -m "docs: define v2 graph core projection"
```

---

## Task 2: Graph Schema And Store

**Files:**
- Create: `backend/migrations/0010_create_graph_core.sql`
- Modify: `backend/src/lib.rs`
- Create: `backend/src/graph.rs`
- Create: `backend/tests/graph.rs`

- [ ] **Step 1: Write graph store tests first**

Create `backend/tests/graph.rs` with these tests:

```rust
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_hub_backend::graph::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore,
    GraphStoreError, NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType,
};
use hermes_hub_backend::storage::Database;
use serde_json::json;

#[tokio::test]
async fn graph_store_upserts_node_idempotently_against_postgres() {
    let Some(store) = live_graph_store("node idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    let node = NewGraphNode::new(
        GraphNodeKind::Person,
        format!("contact-{suffix}"),
        format!("Alex {suffix}"),
    )
    .properties(json!({"email_address": format!("alex-{suffix}@example.com")}));

    let first = store.upsert_node(&node).await.expect("first node upsert");
    let second = store.upsert_node(&node).await.expect("second node upsert");

    assert_eq!(first.node_id, second.node_id);
    assert_eq!(first.node_kind, GraphNodeKind::Person);
    assert_eq!(first.stable_key, format!("contact-{suffix}"));
}

#[tokio::test]
async fn graph_store_upserts_edge_with_evidence_idempotently_against_postgres() {
    let Some(store) = live_graph_store("edge idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("contact-{suffix}"),
            format!("Person {suffix}"),
        ))
        .await
        .expect("person node");
    let email = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("person-{suffix}@example.com"),
            format!("person-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let edge = NewGraphEdge::new(
        person.node_id.clone(),
        email.node_id.clone(),
        RelationshipType::PersonHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );
    let evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Contact,
        format!("contact-{suffix}"),
    );

    let first = store
        .upsert_edge_with_evidence(&edge, &[evidence.clone()])
        .await
        .expect("first edge");
    let second = store
        .upsert_edge_with_evidence(&edge, &[evidence])
        .await
        .expect("second edge");

    assert_eq!(first.edge_id, second.edge_id);
    assert_eq!(first.relationship_type, RelationshipType::PersonHasEmailAddress);
    assert_eq!(first.review_state, GraphReviewState::SystemAccepted);
}

#[tokio::test]
async fn graph_store_rejects_system_edge_without_evidence_against_postgres() {
    let Some(store) = live_graph_store("evidence requirement").await else {
        return;
    };
    let suffix = unique_suffix();
    let left = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("left-{suffix}"),
            "Left",
        ))
        .await
        .expect("left node");
    let right = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("right-{suffix}@example.com"),
            "right@example.com",
        ))
        .await
        .expect("right node");
    let edge = NewGraphEdge::new(
        left.node_id,
        right.node_id,
        RelationshipType::PersonHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );

    let error = store
        .upsert_edge_with_evidence(&edge, &[])
        .await
        .expect_err("system edge without evidence must fail");

    assert!(matches!(error, GraphStoreError::SystemEdgeRequiresEvidence));
}

#[tokio::test]
async fn graph_store_rejects_invalid_confidence_before_database_write() {
    let store = disconnected_graph_store();
    let edge = NewGraphEdge::new(
        "graph:node:v1:person:left".to_owned(),
        "graph:node:v1:email:right@example.com".to_owned(),
        RelationshipType::PersonHasEmailAddress,
        1.1,
        GraphReviewState::SystemAccepted,
    );
    let evidence = NewGraphEvidence::new(GraphEvidenceSourceKind::Contact, "contact-id");

    let error = store
        .upsert_edge_with_evidence(&edge, &[evidence])
        .await
        .expect_err("invalid confidence must fail");

    assert!(matches!(error, GraphStoreError::InvalidConfidence(_)));
}

async fn live_graph_store(test_name: &str) -> Option<GraphStore> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live graph {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(GraphStore::new(database.pool().expect("configured pool").clone()))
}

fn disconnected_graph_store() -> GraphStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://hermes:unused@127.0.0.1:1/hermes_hub")
        .expect("create lazy test pool");
    GraphStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

- [ ] **Step 2: Run graph tests and verify the expected compile failure**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test graph -- --nocapture --test-threads=1
```

Expected:

- Fails because `hermes_hub_backend::graph` does not exist yet.

- [ ] **Step 3: Add graph migration**

Create `backend/migrations/0010_create_graph_core.sql` with this content:

```sql
CREATE TABLE IF NOT EXISTS graph_nodes (
    node_id TEXT PRIMARY KEY,
    node_kind TEXT NOT NULL,
    stable_key TEXT NOT NULL,
    label TEXT NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_nodes_kind CHECK (node_kind IN ('person', 'email_address', 'message', 'document')),
    CONSTRAINT graph_nodes_stable_key_not_empty CHECK (length(trim(stable_key)) > 0),
    CONSTRAINT graph_nodes_label_not_empty CHECK (length(trim(label)) > 0),
    CONSTRAINT graph_nodes_properties_is_object CHECK (jsonb_typeof(properties) = 'object'),
    UNIQUE (node_kind, stable_key)
);

CREATE TABLE IF NOT EXISTS graph_edges (
    edge_id TEXT PRIMARY KEY,
    source_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
    target_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL,
    confidence NUMERIC(5,4) NOT NULL,
    review_state TEXT NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_edges_relationship_type CHECK (
        relationship_type IN (
            'person_has_email_address',
            'person_sent_message',
            'person_received_message',
            'email_address_sent_message',
            'email_address_received_message'
        )
    ),
    CONSTRAINT graph_edges_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT graph_edges_review_state CHECK (
        review_state IN ('system_accepted', 'suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT graph_edges_properties_is_object CHECK (jsonb_typeof(properties) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS graph_edges_active_unique
ON graph_edges (source_node_id, target_node_id, relationship_type)
WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS graph_evidence (
    evidence_id TEXT PRIMARY KEY,
    edge_id TEXT NOT NULL REFERENCES graph_edges(edge_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    excerpt TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_evidence_source_kind CHECK (source_kind IN ('contact', 'message', 'document', 'raw_record')),
    CONSTRAINT graph_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT graph_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (edge_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS graph_nodes_label_idx ON graph_nodes (label);
CREATE INDEX IF NOT EXISTS graph_edges_source_idx ON graph_edges (source_node_id);
CREATE INDEX IF NOT EXISTS graph_edges_target_idx ON graph_edges (target_node_id);
CREATE INDEX IF NOT EXISTS graph_evidence_edge_idx ON graph_evidence (edge_id);
```

- [ ] **Step 4: Expose the graph module**

In `backend/src/lib.rs`, add this module declaration with the existing module list:

```rust
pub mod graph;
```

- [ ] **Step 5: Implement graph store types and deterministic IDs**

Create `backend/src/graph.rs` with public types and helpers matching this contract:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone)]
pub struct GraphStore {
    pool: PgPool,
}

impl GraphStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_node(
        &self,
        node: &NewGraphNode,
    ) -> Result<GraphNode, GraphStoreError> {
        node.validate()?;
        let node_id = node_id(node.node_kind, &node.stable_key);
        let row = sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_kind, stable_key, label, properties)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = EXCLUDED.properties,
                updated_at = now()
            RETURNING node_id, node_kind, stable_key, label, properties, created_at, updated_at
            "#,
        )
        .bind(&node_id)
        .bind(node.node_kind.as_str())
        .bind(&node.stable_key)
        .bind(&node.label)
        .bind(&node.properties)
        .fetch_one(&self.pool)
        .await?;

        row_to_node(row)
    }

    pub async fn upsert_edge_with_evidence(
        &self,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphStoreError> {
        edge.validate()?;
        if edge.review_state == GraphReviewState::SystemAccepted && evidence.is_empty() {
            return Err(GraphStoreError::SystemEdgeRequiresEvidence);
        }
        for item in evidence {
            item.validate()?;
        }

        let edge_id = edge_id(
            &edge.source_node_id,
            edge.relationship_type,
            &edge.target_node_id,
        );
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO graph_edges (
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence,
                review_state,
                properties,
                valid_from,
                valid_to
            )
            VALUES ($1, $2, $3, $4, CAST($5 AS NUMERIC(5,4)), $6, $7, $8, $9)
            ON CONFLICT (source_node_id, target_node_id, relationship_type) WHERE valid_to IS NULL
            DO UPDATE SET
                confidence = EXCLUDED.confidence,
                review_state = EXCLUDED.review_state,
                properties = EXCLUDED.properties,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                updated_at = now()
            RETURNING
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence::float8 AS confidence,
                review_state,
                properties,
                valid_from,
                valid_to,
                created_at,
                updated_at
            "#,
        )
        .bind(&edge_id)
        .bind(&edge.source_node_id)
        .bind(&edge.target_node_id)
        .bind(edge.relationship_type.as_str())
        .bind(edge.confidence)
        .bind(edge.review_state.as_str())
        .bind(&edge.properties)
        .bind(edge.valid_from)
        .bind(edge.valid_to)
        .fetch_one(&mut *transaction)
        .await?;

        for item in evidence {
            let evidence_id = evidence_id(&edge_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO graph_evidence (evidence_id, edge_id, source_kind, source_id, excerpt, metadata)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (edge_id, source_kind, source_id)
                DO UPDATE SET
                    excerpt = EXCLUDED.excerpt,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&edge_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(&item.excerpt)
            .bind(&item.metadata)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        row_to_edge(row)
    }
}
```

The same file must define:

```rust
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    Person,
    EmailAddress,
    Message,
    Document,
}

impl GraphNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::EmailAddress => "email_address",
            Self::Message => "message",
            Self::Document => "document",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    PersonHasEmailAddress,
    PersonSentMessage,
    PersonReceivedMessage,
    EmailAddressSentMessage,
    EmailAddressReceivedMessage,
}

impl RelationshipType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersonHasEmailAddress => "person_has_email_address",
            Self::PersonSentMessage => "person_sent_message",
            Self::PersonReceivedMessage => "person_received_message",
            Self::EmailAddressSentMessage => "email_address_sent_message",
            Self::EmailAddressReceivedMessage => "email_address_received_message",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphReviewState {
    SystemAccepted,
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl GraphReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SystemAccepted => "system_accepted",
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEvidenceSourceKind {
    Contact,
    Message,
    Document,
    RawRecord,
}

impl GraphEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contact => "contact",
            Self::Message => "message",
            Self::Document => "document",
            Self::RawRecord => "raw_record",
        }
    }
}
```

Add these constructor and validation types in `backend/src/graph.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewGraphNode {
    pub node_kind: GraphNodeKind,
    pub stable_key: String,
    pub label: String,
    pub properties: Value,
}

impl NewGraphNode {
    pub fn new(
        node_kind: GraphNodeKind,
        stable_key: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            node_kind,
            stable_key: stable_key.into(),
            label: label.into(),
            properties: json!({}),
        }
    }

    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = properties;
        self
    }

    fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("stable_key", &self.stable_key)?;
        validate_non_empty("label", &self.label)?;
        validate_json_object("node properties", &self.properties)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewGraphEdge {
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: RelationshipType,
    pub confidence: f64,
    pub review_state: GraphReviewState,
    pub properties: Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

impl NewGraphEdge {
    pub fn new(
        source_node_id: String,
        target_node_id: String,
        relationship_type: RelationshipType,
        confidence: f64,
        review_state: GraphReviewState,
    ) -> Self {
        Self {
            source_node_id,
            target_node_id,
            relationship_type,
            confidence,
            review_state,
            properties: json!({}),
            valid_from: None,
            valid_to: None,
        }
    }

    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = properties;
        self
    }

    fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("source_node_id", &self.source_node_id)?;
        validate_non_empty("target_node_id", &self.target_node_id)?;
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(GraphStoreError::InvalidConfidence(self.confidence));
        }
        validate_json_object("edge properties", &self.properties)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewGraphEvidence {
    pub source_kind: GraphEvidenceSourceKind,
    pub source_id: String,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

impl NewGraphEvidence {
    pub fn new(source_kind: GraphEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            excerpt: None,
            metadata: json!({}),
        }
    }

    pub fn excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_json_object("evidence metadata", &self.metadata)
    }
}
```

Add row models and errors:

```rust
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNode {
    pub node_id: String,
    pub node_kind: GraphNodeKind,
    pub stable_key: String,
    pub label: String,
    pub properties: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEdge {
    pub edge_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: RelationshipType,
    pub confidence: f64,
    pub review_state: GraphReviewState,
    pub properties: Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum GraphStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("graph edge confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("system-created graph edges require evidence")]
    SystemEdgeRequiresEvidence,

    #[error("unknown graph node kind stored in database: {0}")]
    UnknownNodeKind(String),

    #[error("unknown graph relationship type stored in database: {0}")]
    UnknownRelationshipType(String),

    #[error("unknown graph review state stored in database: {0}")]
    UnknownReviewState(String),
}
```

Add deterministic ID helpers:

```rust
pub fn node_id(kind: GraphNodeKind, stable_key: &str) -> String {
    format!("graph:node:v1:{}:{stable_key}", kind.as_str())
}

pub fn edge_id(
    source_node_id: &str,
    relationship_type: RelationshipType,
    target_node_id: &str,
) -> String {
    format!(
        "graph:edge:v1:{}:{}:{}:{}:{}:{}",
        source_node_id.len(),
        source_node_id,
        relationship_type.as_str().len(),
        relationship_type.as_str(),
        target_node_id.len(),
        target_node_id
    )
}

pub fn evidence_id(
    edge_id: &str,
    source_kind: GraphEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "graph:evidence:v1:{}:{}:{}:{}:{}:{}",
        edge_id.len(),
        edge_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}
```

Implement `row_to_node`, `row_to_edge`, enum parsers and validation helpers in the same file using `row.try_get`, `trim().is_empty()` checks and `Value::is_object()`.

- [ ] **Step 6: Run graph tests against live PostgreSQL**

Run:

```bash
make db-up
set -a; . docker/.env; set +a; HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test graph -- --nocapture --test-threads=1
```

Expected:

- All `backend/tests/graph.rs` tests pass.

- [ ] **Step 7: Commit graph schema and store**

Run:

```bash
cargo fmt --manifest-path backend/Cargo.toml
git diff --check
```

Expected:

- Both commands exit 0.

Commit:

```bash
git add backend/migrations/0010_create_graph_core.sql backend/src/lib.rs backend/src/graph.rs backend/tests/graph.rs
git commit -m "feat: add graph core store"
```

---

## Task 3: V1 Graph Projection

**Files:**
- Modify: `backend/src/lib.rs`
- Create: `backend/src/graph_projection.rs`
- Create: `backend/tests/graph_projection.rs`

- [ ] **Step 1: Write projection tests first**

Create `backend/tests/graph_projection.rs` with live PostgreSQL tests that:

1. Insert a contact through `ContactProjectionStore`.
2. Insert a raw email and project it through `MessageProjectionStore`.
3. Import a document through `DocumentImportStore`.
4. Run `GraphProjectionService::project_from_v1`.
5. Run it again and verify counts are unchanged.
6. Verify unknown sender/recipient endpoints use `email_address` nodes and not invented `person` nodes.

Use this test shape:

```rust
#[tokio::test]
async fn graph_projection_is_idempotent_for_v1_sources_against_postgres() {
    let Some(context) = live_projection_context("graph projection idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    seed_contact_message_and_document(&context, suffix).await;

    let first = context.graph_projection.project_from_v1().await.expect("first graph projection");
    let second = context.graph_projection.project_from_v1().await.expect("second graph projection");

    assert_eq!(first.nodes_upserted, second.nodes_upserted);
    assert_eq!(first.edges_upserted, second.edges_upserted);
    assert_eq!(first.evidence_upserted, second.evidence_upserted);

    let person_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM graph_nodes WHERE node_kind = 'person' AND stable_key LIKE $1",
    )
    .bind(format!("contact:v1:email:%unknown-{suffix}%"))
    .fetch_one(&context.pool)
    .await
    .expect("unknown sender person count");
    assert_eq!(person_count, 0);
}
```

The helper functions in this test file must follow the existing live-test pattern from `backend/tests/messages.rs`, including `HERMES_TEST_DATABASE_URL` skip behavior and `unique_suffix()`.

- [ ] **Step 2: Run projection tests and verify the expected compile failure**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test graph_projection -- --nocapture --test-threads=1
```

Expected:

- Fails because `GraphProjectionService` does not exist yet.

- [ ] **Step 3: Expose the graph projection module**

In `backend/src/lib.rs`, add:

```rust
pub mod graph_projection;
```

- [ ] **Step 4: Implement projection service**

Create `backend/src/graph_projection.rs` with:

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::graph::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore,
    GraphStoreError, NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType,
};

#[derive(Clone)]
pub struct GraphProjectionService {
    pool: PgPool,
    graph: GraphStore,
}

impl GraphProjectionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            graph: GraphStore::new(pool.clone()),
            pool,
        }
    }

    pub async fn project_from_v1(&self) -> Result<GraphProjectionReport, GraphProjectionError> {
        let mut report = GraphProjectionReport::default();

        for contact in self.list_contacts().await? {
            self.project_contact(&contact, &mut report).await?;
        }
        for message in self.list_messages().await? {
            self.project_message(&message, &mut report).await?;
        }
        for document in self.list_documents().await? {
            self.project_document(&document, &mut report).await?;
        }

        Ok(report)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphProjectionReport {
    pub nodes_upserted: usize,
    pub edges_upserted: usize,
    pub evidence_upserted: usize,
}

#[derive(Debug, Error)]
pub enum GraphProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),

    #[error("message recipients must be a JSON array of strings")]
    InvalidRecipients,
}
```

Add private source row structs:

```rust
struct ContactRow {
    contact_id: String,
    display_name: String,
    email_address: String,
}

struct MessageRow {
    message_id: String,
    raw_record_id: String,
    account_id: String,
    provider_record_id: String,
    subject: String,
    sender: String,
    recipients: Vec<String>,
    body_text: String,
    occurred_at: Option<DateTime<Utc>>,
}

struct DocumentRow {
    document_id: String,
    document_kind: String,
    title: String,
    source_fingerprint: String,
    imported_at: DateTime<Utc>,
}
```

Implement list queries exactly against the existing V1 tables:

```rust
async fn list_contacts(&self) -> Result<Vec<ContactRow>, GraphProjectionError> {
    let rows = sqlx::query(
        "SELECT contact_id, display_name, email_address FROM contacts ORDER BY contact_id",
    )
    .fetch_all(&self.pool)
    .await?;
    rows.into_iter().map(row_to_contact).collect()
}

async fn list_messages(&self) -> Result<Vec<MessageRow>, GraphProjectionError> {
    let rows = sqlx::query(
        r#"
        SELECT
            message_id,
            raw_record_id,
            account_id,
            provider_record_id,
            subject,
            sender,
            recipients,
            body_text,
            occurred_at
        FROM communication_messages
        ORDER BY message_id
        "#,
    )
    .fetch_all(&self.pool)
    .await?;
    rows.into_iter().map(row_to_message).collect()
}

async fn list_documents(&self) -> Result<Vec<DocumentRow>, GraphProjectionError> {
    let rows = sqlx::query(
        r#"
        SELECT document_id, document_kind, title, source_fingerprint, imported_at
        FROM documents
        ORDER BY document_id
        "#,
    )
    .fetch_all(&self.pool)
    .await?;
    rows.into_iter().map(row_to_document).collect()
}
```

Implement projection rules:

```rust
async fn project_contact(
    &self,
    contact: &ContactRow,
    report: &mut GraphProjectionReport,
) -> Result<(), GraphProjectionError> {
    let normalized_email = normalize_email_address(&contact.email_address);
    let person = self.graph.upsert_node(
        &NewGraphNode::new(GraphNodeKind::Person, &contact.contact_id, &contact.display_name)
            .properties(json!({ "email_address": normalized_email })),
    ).await?;
    report.nodes_upserted += 1;

    let email = self.graph.upsert_node(
        &NewGraphNode::new(GraphNodeKind::EmailAddress, &normalized_email, &normalized_email),
    ).await?;
    report.nodes_upserted += 1;

    self.graph.upsert_edge_with_evidence(
        &NewGraphEdge::new(
            person.node_id,
            email.node_id,
            RelationshipType::PersonHasEmailAddress,
            1.0,
            GraphReviewState::SystemAccepted,
        ),
        &[NewGraphEvidence::new(GraphEvidenceSourceKind::Contact, &contact.contact_id)],
    ).await?;
    report.edges_upserted += 1;
    report.evidence_upserted += 1;

    Ok(())
}
```

Message projection must:

- Create a `message` node with `account_id`, `provider_record_id`, `raw_record_id` and `occurred_at` properties.
- Resolve sender by exact normalized email against `contacts.email_address`.
- Resolve each recipient by exact normalized email.
- Use `person_sent_message` or `person_received_message` when a contact is found.
- Use `email_address_sent_message` or `email_address_received_message` when no contact is found.
- Attach evidence with `source_kind = message`, `source_id = communication_messages.message_id`, `excerpt = subject` and metadata containing `raw_record_id` and `provider_record_id`.

Document projection must create only a `document` node with `document_kind`, `source_fingerprint` and `imported_at` properties.

- [ ] **Step 5: Run projection tests against live PostgreSQL**

Run:

```bash
make db-up
set -a; . docker/.env; set +a; HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test graph_projection -- --nocapture --test-threads=1
```

Expected:

- All `backend/tests/graph_projection.rs` tests pass.

- [ ] **Step 6: Commit graph projection**

Run:

```bash
cargo fmt --manifest-path backend/Cargo.toml
git diff --check
```

Expected:

- Both commands exit 0.

Commit:

```bash
git add backend/src/lib.rs backend/src/graph_projection.rs backend/tests/graph_projection.rs
git commit -m "feat: project v1 data into graph"
```

---

## Task 4: Graph Read APIs

**Files:**
- Modify: `backend/src/graph.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/graph_api.rs`

- [ ] **Step 1: Write API tests first**

Create `backend/tests/graph_api.rs` with tests for:

- Missing token on `/api/v2/graph/summary` returns `401`.
- Missing actor on `/api/v2/graph/summary` returns `400`.
- Empty database graph summary returns `is_empty = true`.
- `/api/v2/graph/search?q=alex` returns matching nodes.
- `/api/v2/graph/search?q=` returns `400`.
- `/api/v2/graph/neighborhood?node_id=<id>&depth=1` returns selected node, neighboring nodes, edges and evidence.
- Missing `node_id` returns `404`.
- `depth=2` returns `400`.

Reuse the `tower::ServiceExt` request helpers from `backend/tests/v1_api.rs`. Use `GraphStore` directly to seed happy-path API data.

- [ ] **Step 2: Run API tests and verify the expected route failure**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test graph_api -- --nocapture --test-threads=1
```

Expected:

- Fails because `/api/v2/graph/*` routes do not exist yet.

- [ ] **Step 3: Add read query methods to `GraphStore`**

In `backend/src/graph.rs`, add response models:

```rust
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphCount {
    pub key: String,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphSummary {
    pub node_counts: Vec<GraphCount>,
    pub edge_counts: Vec<GraphCount>,
    pub evidence_count: i64,
    pub latest_projection_at: Option<DateTime<Utc>>,
    pub is_empty: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEvidenceSummary {
    pub edge_id: String,
    pub source_kind: GraphEvidenceSourceKind,
    pub source_id: String,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNeighborhood {
    pub selected_node: GraphNode,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub evidence: Vec<GraphEvidenceSummary>,
}
```

Add methods:

```rust
pub async fn summary(&self) -> Result<GraphSummary, GraphStoreError>;
pub async fn search_nodes(&self, query: &str, limit: i64) -> Result<Vec<GraphNode>, GraphStoreError>;
pub async fn neighborhood(&self, node_id: &str) -> Result<Option<GraphNeighborhood>, GraphStoreError>;
```

Method rules:

- `summary` groups `graph_nodes` by `node_kind`, groups `graph_edges` by `relationship_type`, counts `graph_evidence`, sets `latest_projection_at` to greatest `updated_at` from nodes/edges and sets `is_empty` from total node count.
- `search_nodes` uses `label ILIKE $1 OR stable_key ILIKE $1`, orders by `node_kind, label`, and applies caller-provided limit.
- `neighborhood` returns `None` when selected node is absent; otherwise it returns the selected node, all depth-1 neighbors, all touching active edges and evidence for those returned edges.

- [ ] **Step 4: Register API routes**

In `backend/src/lib.rs`, add route registrations:

```rust
.route("/api/v2/graph/summary", get(get_graph_summary))
.route("/api/v2/graph/neighborhood", get(get_graph_neighborhood))
.route("/api/v2/graph/search", get(get_graph_search))
```

Add query structs:

```rust
#[derive(Deserialize)]
struct GraphNeighborhoodQuery {
    node_id: String,
    depth: Option<u8>,
}

#[derive(Deserialize)]
struct GraphSearchQuery {
    q: String,
    limit: Option<i64>,
}
```

Add handler logic:

```rust
async fn get_graph_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::graph::GraphSummary>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    Ok(Json(graph_store(&state)?.summary().await?))
}

async fn get_graph_neighborhood(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GraphNeighborhoodQuery>,
) -> Result<Json<crate::graph::GraphNeighborhood>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    if query.depth.unwrap_or(1) != 1 {
        return Err(ApiError::InvalidGraphQuery("depth supports only 1"));
    }
    let Some(neighborhood) = graph_store(&state)?.neighborhood(&query.node_id).await? else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(neighborhood))
}

async fn get_graph_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GraphSearchQuery>,
) -> Result<Json<Vec<crate::graph::GraphNode>>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let search = query.q.trim();
    if search.is_empty() {
        return Err(ApiError::InvalidGraphQuery("q must not be empty"));
    }
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(graph_store(&state)?.search_nodes(search, limit).await?))
}
```

Add helper and errors:

```rust
fn graph_store(state: &AppState) -> Result<crate::graph::GraphStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::graph::GraphStore::new(pool.clone()))
}
```

Extend `ApiError`:

```rust
Graph(crate::graph::GraphStoreError),
InvalidGraphQuery(&'static str),
```

Map `InvalidGraphQuery` to `400` with:

```rust
("invalid_graph_query", message.to_owned())
```

Map `Graph` to `500`, log the internal error, and return:

```rust
("graph_store_error", "graph store operation failed".to_owned())
```

Add:

```rust
impl From<crate::graph::GraphStoreError> for ApiError {
    fn from(error: crate::graph::GraphStoreError) -> Self {
        Self::Graph(error)
    }
}
```

- [ ] **Step 5: Run API tests**

Run:

```bash
make db-up
set -a; . docker/.env; set +a; HERMES_TEST_DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}" cargo test --manifest-path backend/Cargo.toml --test graph_api -- --nocapture --test-threads=1
```

Expected:

- All `backend/tests/graph_api.rs` tests pass.

- [ ] **Step 6: Commit graph APIs**

Run:

```bash
cargo fmt --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
git diff --check
```

Expected:

- All commands exit 0.

Commit:

```bash
git add backend/src/graph.rs backend/src/lib.rs backend/tests/graph_api.rs
git commit -m "feat: expose graph read APIs"
```

---

## Task 5: Graph Smoke Target And Backend Validation

**Files:**
- Modify: `Makefile`
- Modify: `backend/README.md`

- [ ] **Step 1: Add Makefile graph smoke target**

In `Makefile`, add `backend-graph-smoke-dev` to `.PHONY`.

Add this command near the existing backend smoke targets:

```make
backend-graph-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test graph --test graph_projection --test graph_api -- --nocapture --test-threads=1
```

Add `backend-graph-smoke-dev` to the `validate:` dependency list after `backend-documents-smoke-dev`.

- [ ] **Step 2: Document graph smoke command**

In `backend/README.md`, add:

````markdown
Graph core smoke:

```bash
make backend-graph-smoke-dev
```

This starts the local PostgreSQL container, runs graph store, projection and read API tests with `HERMES_TEST_DATABASE_URL`, then stops PostgreSQL.
````

- [ ] **Step 3: Run backend graph smoke**

Run:

```bash
make backend-graph-smoke-dev
```

Expected:

- Graph store, graph projection and graph API tests pass against live PostgreSQL.

- [ ] **Step 4: Run backend validation**

Run:

```bash
make backend-validate
```

Expected:

- Rust format check, clippy and backend tests pass.

- [ ] **Step 5: Commit smoke target**

Run:

```bash
git diff --check
```

Expected:

- Exits 0.

Commit:

```bash
git add Makefile backend/README.md
git commit -m "build: add graph smoke validation"
```

---

## Task 6: Frontend Graph API Wiring

**Files:**
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `frontend/README.md`

- [ ] **Step 1: Add TypeScript graph API types**

In `frontend/src/lib/api.ts`, add:

```ts
export type GraphNodeKind = 'person' | 'email_address' | 'message' | 'document';

export type GraphRelationshipType =
	| 'person_has_email_address'
	| 'person_sent_message'
	| 'person_received_message'
	| 'email_address_sent_message'
	| 'email_address_received_message';

export type GraphReviewState =
	| 'system_accepted'
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type GraphNode = {
	node_id: string;
	node_kind: GraphNodeKind;
	stable_key: string;
	label: string;
	properties: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type GraphEdge = {
	edge_id: string;
	source_node_id: string;
	target_node_id: string;
	relationship_type: GraphRelationshipType;
	confidence: number;
	review_state: GraphReviewState;
	properties: Record<string, unknown>;
	valid_from: string | null;
	valid_to: string | null;
	created_at: string;
	updated_at: string;
};

export type GraphCount = {
	key: string;
	count: number;
};

export type GraphSummary = {
	node_counts: GraphCount[];
	edge_counts: GraphCount[];
	evidence_count: number;
	latest_projection_at: string | null;
	is_empty: boolean;
};

export type GraphEvidenceSummary = {
	edge_id: string;
	source_kind: 'contact' | 'message' | 'document' | 'raw_record';
	source_id: string;
	excerpt: string | null;
	metadata: Record<string, unknown>;
};

export type GraphNeighborhood = {
	selected_node: GraphNode;
	nodes: GraphNode[];
	edges: GraphEdge[];
	evidence: GraphEvidenceSummary[];
};
```

Add a reusable GET helper:

```ts
async function getJson<TResponse>(
	baseUrl: string,
	token: string,
	actorId: string,
	path: string
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		headers: {
			Authorization: `Bearer ${token}`,
			'X-Hermes-Actor-Id': actorId
		}
	});

	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as { message?: string } | null;
		throw new Error(error?.message ?? `GET ${path} failed: ${response.status}`);
	}

	return (await response.json()) as TResponse;
}
```

Refactor `fetchV1Status` to call `getJson<V1Status>`.

Add graph fetchers:

```ts
export async function fetchGraphSummary(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<GraphSummary> {
	return getJson(baseUrl, token, actorId, '/api/v2/graph/summary');
}

export async function searchGraphNodes(
	baseUrl: string,
	token: string,
	actorId: string,
	query: string,
	limit = 20
): Promise<GraphNode[]> {
	const params = new URLSearchParams({ q: query, limit: String(limit) });
	return getJson(baseUrl, token, actorId, `/api/v2/graph/search?${params.toString()}`);
}

export async function fetchGraphNeighborhood(
	baseUrl: string,
	token: string,
	actorId: string,
	nodeId: string
): Promise<GraphNeighborhood> {
	const params = new URLSearchParams({ node_id: nodeId, depth: '1' });
	return getJson(baseUrl, token, actorId, `/api/v2/graph/neighborhood?${params.toString()}`);
}
```

- [ ] **Step 2: Wire dashboard graph state**

In `frontend/src/routes/+page.svelte`:

- Import `fetchGraphSummary`, `searchGraphNodes`, `fetchGraphNeighborhood` and their types.
- Add state for `graphSummary`, `graphError`, `graphSearchQuery`, `graphSearchResults`, `selectedGraphNeighborhood` and `isGraphExplorerOpen`.
- On mount, call `fetchGraphSummary(apiBaseUrl, localApiToken, actorId)`.
- In the Knowledge Graph card, render real node and edge totals from the summary.
- If `graphSummary?.is_empty` is true, render the existing visual graph area as an empty state with zero counts and disabled merge/edit controls.
- Make `Explore Graph` open a desktop drawer or panel that supports search and selecting a node.
- Keep merge/split, OCR links, task candidates and graph editing controls visibly disabled.

Use this state shape:

```ts
let graphSummary: GraphSummary | null = null;
let graphError: string | null = null;
let graphSearchQuery = '';
let graphSearchResults: GraphNode[] = [];
let selectedGraphNeighborhood: GraphNeighborhood | null = null;
let isGraphExplorerOpen = false;
let isGraphLoading = false;
```

Use this loader:

```ts
async function loadGraphSummary() {
	isGraphLoading = true;
	graphError = null;
	try {
		graphSummary = await fetchGraphSummary(apiBaseUrl, localApiToken, actorId);
	} catch (error) {
		graphError = error instanceof Error ? error.message : 'Graph summary request failed';
	} finally {
		isGraphLoading = false;
	}
}
```

Use this search action:

```ts
async function runGraphSearch() {
	const query = graphSearchQuery.trim();
	if (!query) {
		graphSearchResults = [];
		return;
	}
	graphSearchResults = await searchGraphNodes(apiBaseUrl, localApiToken, actorId, query, 20);
}
```

Use this selection action:

```ts
async function selectGraphNode(node: GraphNode) {
	selectedGraphNeighborhood = await fetchGraphNeighborhood(
		apiBaseUrl,
		localApiToken,
		actorId,
		node.node_id
	);
}
```

- [ ] **Step 3: Update frontend docs**

In `frontend/README.md`, add:

```markdown
The Knowledge Graph dashboard reads `/api/v2/graph/summary` and the graph explorer reads `/api/v2/graph/search` plus `/api/v2/graph/neighborhood`. Local frontend runs must provide `VITE_HERMES_API_BASE_URL`, `VITE_HERMES_LOCAL_API_TOKEN` and `VITE_HERMES_ACTOR_ID`; `make dev` exports these from `docker/.env`.
```

- [ ] **Step 4: Run frontend validation**

Run:

```bash
pnpm --dir frontend check
pnpm --dir frontend build
```

Expected:

- Both commands pass.

- [ ] **Step 5: Commit frontend graph wiring**

Run:

```bash
git diff --check
```

Expected:

- Exits 0.

Commit:

```bash
git add frontend/src/lib/api.ts frontend/src/routes/+page.svelte frontend/README.md
git commit -m "feat: wire dashboard to graph APIs"
```

---

## Task 7: Final V2 Graph Core Validation

**Files:**
- Modify: `docs/roadmap/v2-graph-core-checklist.md`

- [ ] **Step 1: Run full validation gate**

Run:

```bash
make validate
```

Expected:

- Compose config renders.
- Backend format, clippy and tests pass.
- All live PostgreSQL smoke targets pass, including `backend-graph-smoke-dev`.
- Frontend check and build pass.

- [ ] **Step 2: Update V2 graph checklist**

In `docs/roadmap/v2-graph-core-checklist.md`, mark completed acceptance gates as checked only when the matching validation evidence exists.

- [ ] **Step 3: Run final focused checks**

Run:

```bash
git diff --check
pnpm --dir frontend check
pnpm --dir frontend build
make backend-graph-smoke-dev
```

Expected:

- All commands exit 0.

- [ ] **Step 4: Commit checklist update**

Commit:

```bash
git add docs/roadmap/v2-graph-core-checklist.md
git commit -m "docs: mark v2 graph core gates"
```

---

## Plan Self-Review

Spec coverage:

- ADR-0045 is Task 1.
- Graph tables and constraints are Task 2.
- Idempotent nodes, edges and evidence requirement are Task 2.
- Projection from contacts, messages and documents is Task 3.
- Exact email identity rules are Task 3.
- Read APIs for summary, neighborhood and search are Task 4.
- Smoke target and `make validate` wiring are Task 5.
- Frontend dashboard graph wiring and disabled future controls are Task 6.
- Final acceptance checklist update is Task 7.

Red-flag scan:

- The plan contains no deferred implementation markers.
- Each code step names exact files and commands.
- Later tasks use types introduced in earlier tasks.

Execution options:

1. **Subagent-Driven (recommended)** - dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** - execute tasks in this session using `superpowers:executing-plans`, batch execution with checkpoints.
