use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::persons::PersonProjectionStore;
use hermes_hub_backend::documents::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::graph::{GraphNodeKind, node_id};
use hermes_hub_backend::graph_projection::GraphProjectionService;
use hermes_hub_backend::messages::{MessageProjectionStore, project_raw_email_message};
use hermes_hub_backend::project_link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewState, ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use hermes_hub_backend::projects::{NewProject, ProjectStore, project_graph_node_id};
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn graph_projection_is_idempotent_for_v1_sources_against_postgres() {
    let Some(context) = live_projection_context("graph projection idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    seed_person_message_and_document(&context, suffix).await;

    let first = context
        .graph_projection
        .project_from_v1()
        .await
        .expect("first graph projection");
    let counts_after_first = graph_counts_for_suffix(&context.pool, suffix).await;
    let second = context
        .graph_projection
        .project_from_v1()
        .await
        .expect("second graph projection");
    let counts_after_second = graph_counts_for_suffix(&context.pool, suffix).await;

    assert_eq!(first.nodes_upserted, second.nodes_upserted);
    assert_eq!(first.edges_upserted, second.edges_upserted);
    assert_eq!(first.evidence_upserted, second.evidence_upserted);
    assert_eq!(counts_after_first, counts_after_second);

    let person_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM graph_nodes WHERE node_kind = 'person' AND stable_key LIKE $1",
    )
    .bind(format!("person:v1:email:%unknown-{suffix}%"))
    .fetch_one(&context.pool)
    .await
    .expect("unknown sender person count");
    assert_eq!(person_count, 0);

    assert_unknown_email_endpoint_projected(
        &context.pool,
        &format!("unknown-{suffix}@example.com"),
        &format!("provider-graph-projection-{suffix}"),
        "email_address_sent_message",
    )
    .await;
    assert_unknown_email_endpoint_projected(
        &context.pool,
        &format!("unknown-recipient-{suffix}@example.com"),
        &format!("provider-graph-projection-{suffix}"),
        "email_address_received_message",
    )
    .await;
    assert_known_person_endpoint_projected(&context.pool, suffix).await;
    assert_document_projected(&context.pool, suffix).await;
}

#[tokio::test]
async fn graph_projection_replaces_stale_unknown_message_edges_against_postgres() {
    let Some(context) =
        live_projection_context("graph projection stale message edge replacement").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let sender_email = format!("identity-upgrade-{suffix}@example.com");
    let provider_record_id = format!("provider-graph-identity-upgrade-{suffix}");
    let subject = format!("Graph identity upgrade subject {suffix}");
    let projected = seed_message(
        &context,
        suffix,
        &sender_email,
        &[format!("recipient-upgrade-{suffix}@example.com")],
        &provider_record_id,
        &subject,
    )
    .await;

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("first graph projection before person exists");
    assert_message_edge_with_evidence(
        &context.pool,
        "email_address",
        &sender_email,
        &provider_record_id,
        "email_address_sent_message",
        &projected,
    )
    .await;

    context
        .person_store
        .upsert_email_person(&sender_email)
        .await
        .expect("upsert exact sender person");
    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("second graph projection after person exists");

    assert_message_edge_with_evidence(
        &context.pool,
        "person",
        &sender_email,
        &provider_record_id,
        "person_sent_message",
        &projected,
    )
    .await;
    assert_message_edge_count(
        &context.pool,
        "email_address",
        &sender_email,
        &provider_record_id,
        "email_address_sent_message",
        0,
    )
    .await;
}

#[tokio::test]
async fn graph_projection_links_projects_to_keyword_messages_documents_and_people_against_postgres()
{
    let Some(context) = live_projection_context("project graph projection").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphProject{suffix}");
    let project_id = format!("project:v1:graph:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Project {suffix}"),
                "Product Development",
                "Graph project projection test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(55),
        )
        .await
        .expect("upsert graph project");
    let owner = context
        .person_store
        .upsert_email_person(&format!("graph-project-owner-{suffix}@example.com"))
        .await
        .expect("upsert graph project owner");
    let projected = seed_message(
        &context,
        suffix,
        &owner.email_address,
        &[format!("graph-project-reviewer-{suffix}@example.com")],
        &format!("provider-graph-project-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;
    let document_id = format!("doc_graph_project_{suffix}");
    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            format!("{keyword} notes.md"),
            "# Notes\n\nProject graph content.",
        ))
        .await
        .expect("import graph project document");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("first graph projection");
    let counts_after_first = project_graph_counts(&context.pool, &project_id).await;
    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("second graph projection");
    let counts_after_second = project_graph_counts(&context.pool, &project_id).await;
    assert_eq!(counts_after_first, counts_after_second);

    let project_node_id = project_graph_node_id(&project_id);
    let owner_node_id = node_id(GraphNodeKind::Person, &owner.person_id);
    let reviewer_node_id = node_id(
        GraphNodeKind::EmailAddress,
        &format!("graph-project-reviewer-{suffix}@example.com"),
    );
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Message, &projected.message_id),
            relationship_type: "project_has_message",
            source_kind: "message",
            source_id: &projected.message_id,
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Document, &document_id),
            relationship_type: "project_has_document",
            source_kind: "document",
            source_id: &document_id,
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &owner_node_id,
            relationship_type: "project_involves_person",
            source_kind: "message",
            source_id: &projected.message_id,
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &reviewer_node_id,
            relationship_type: "project_involves_email_address",
            source_kind: "message",
            source_id: &projected.message_id,
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

#[tokio::test]
async fn graph_projection_omits_rejected_project_link_against_postgres() {
    let Some(context) = live_projection_context("project graph projection rejected link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphReject{suffix}");
    let project_id = format!("project:v1:graph-reject:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Reject Project {suffix}"),
                "Product Development",
                "Graph project rejected link test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(50),
        )
        .await
        .expect("upsert graph reject project");

    let projected = seed_message(
        &context,
        suffix,
        &format!("owner-reject-{suffix}@example.com"),
        &[format!("reviewer-reject-{suffix}@example.com")],
        &format!("provider-graph-reject-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;

    context
        .project_link_review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("graph-reject-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: projected.message_id.clone(),
            review_state: ProjectLinkReviewState::UserRejected,
            actor_id: format!("reviewer-actor-{suffix}"),
        })
        .await
        .expect("set rejected link review");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("project projection for rejected link");

    let project_node_id = project_graph_node_id(&project_id);
    let message_node_id = node_id(GraphNodeKind::Message, &projected.message_id);
    let link_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'project_has_message'
        "#,
    )
    .bind(&project_node_id)
    .bind(&message_node_id)
    .fetch_one(&context.pool)
    .await
    .expect("rejected project link count");
    assert_eq!(link_count, 0);

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

#[tokio::test]
async fn graph_projection_marks_confirmed_project_link_user_confirmed_against_postgres() {
    let Some(context) = live_projection_context("project graph projection confirmed link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphConfirm{suffix}");
    let project_id = format!("project:v1:graph-confirm:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Confirm Project {suffix}"),
                "Product Development",
                "Graph project confirmed link test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(50),
        )
        .await
        .expect("upsert graph confirm project");

    let projected = seed_message(
        &context,
        suffix,
        &format!("owner-confirm-{suffix}@example.com"),
        &[format!("reviewer-confirm-{suffix}@example.com")],
        &format!("provider-graph-confirm-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;

    context
        .project_link_review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("graph-confirm-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: projected.message_id.clone(),
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: format!("reviewer-actor-{suffix}"),
        })
        .await
        .expect("set confirmed link review");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("project projection for confirmed link");

    let project_node_id = project_graph_node_id(&project_id);
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Message, &projected.message_id),
            relationship_type: "project_has_message",
            source_kind: "message",
            source_id: &projected.message_id,
            review_state: "user_confirmed",
            confidence: 1.0,
        },
    )
    .await;

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

struct LiveProjectionContext {
    pool: PgPool,
    person_store: PersonProjectionStore,
    communication_store: CommunicationIngestionStore,
    message_store: MessageProjectionStore,
    document_store: DocumentImportStore,
    project_store: ProjectStore,
    graph_projection: GraphProjectionService,
    project_link_review_store: ProjectLinkReviewStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GraphCounts {
    nodes: i64,
    edges: i64,
    evidence: i64,
}

struct ProjectedMessageFixture {
    message_id: String,
    raw_record_id: String,
    provider_record_id: String,
    subject: String,
}

async fn live_projection_context(test_name: &str) -> Option<LiveProjectionContext> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(LiveProjectionContext {
        pool: pool.clone(),
        person_store: PersonProjectionStore::new(pool.clone()),
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        document_store: DocumentImportStore::new(pool.clone()),
        project_store: ProjectStore::new(pool.clone()),
        graph_projection: GraphProjectionService::new(pool.clone()),
        project_link_review_store: ProjectLinkReviewStore::new(pool),
    })
}

async fn seed_person_message_and_document(context: &LiveProjectionContext, suffix: u128) {
    context
        .person_store
        .upsert_email_person(&format!(" Known-{suffix}@Example.com "))
        .await
        .expect("upsert known person");

    seed_message(
        context,
        suffix,
        &format!("Unknown-{suffix}@Example.com"),
        &[
            format!("known-{suffix}@example.com"),
            format!("unknown-recipient-{suffix}@example.com"),
        ],
        &format!("provider-graph-projection-{suffix}"),
        &format!("Graph projection subject {suffix}"),
    )
    .await;

    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_graph_projection_{suffix}"),
            format!("graph-projection-{suffix}.md"),
            "# Graph Projection\n\nDocument body.",
        ))
        .await
        .expect("import graph projection document");
}

async fn seed_message(
    context: &LiveProjectionContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
) -> ProjectedMessageFixture {
    let account_id = format!("acct_graph_projection_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Graph Projection Gmail",
            format!("graph-projection-{suffix}@example.com"),
        ))
        .await
        .expect("store graph projection provider account");

    let raw_record_id = format!("raw_graph_projection_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:graph-projection-{suffix}:{provider_record_id}"),
                format!("batch_graph_projection_{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": "Graph projection body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"graph_projection_test"})),
        )
        .await
        .expect("record graph projection raw message");

    let projected = project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project raw graph projection message");

    ProjectedMessageFixture {
        message_id: projected.message_id,
        raw_record_id: projected.raw_record_id,
        provider_record_id: projected.provider_record_id,
        subject: projected.subject,
    }
}

async fn graph_counts_for_suffix(pool: &PgPool, suffix: u128) -> GraphCounts {
    let pattern = format!("%{suffix}%");
    let nodes =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_nodes WHERE stable_key LIKE $1")
            .bind(&pattern)
            .fetch_one(pool)
            .await
            .expect("graph node count for suffix");
    let edges = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        WHERE source.stable_key LIKE $1 OR target.stable_key LIKE $1
        "#,
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await
    .expect("graph edge count for suffix");
    let evidence = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE source_id LIKE $1 OR metadata::text LIKE $1
        "#,
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await
    .expect("graph evidence count for suffix");

    GraphCounts {
        nodes,
        edges,
        evidence,
    }
}

async fn project_graph_counts(pool: &PgPool, project_id: &str) -> GraphCounts {
    let project_node_id = project_graph_node_id(project_id);
    let nodes = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM graph_nodes WHERE node_id = $1 AND node_kind = 'project'",
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph node count");
    let edges = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges
        WHERE source_node_id = $1
          AND relationship_type IN (
              'project_has_message',
              'project_has_document',
              'project_involves_person',
              'project_involves_email_address'
          )
        "#,
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph edge count");
    let evidence = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence evidence
        JOIN graph_edges edge ON edge.edge_id = evidence.edge_id
        WHERE edge.source_node_id = $1
        "#,
    )
    .bind(&project_node_id)
    .fetch_one(pool)
    .await
    .expect("project graph evidence count");

    GraphCounts {
        nodes,
        edges,
        evidence,
    }
}

struct ExpectedProjectEdge<'a> {
    source_node_id: &'a str,
    target_node_id: &'a str,
    relationship_type: &'a str,
    source_kind: &'a str,
    source_id: &'a str,
    review_state: &'a str,
    confidence: f64,
}

async fn assert_project_edge_with_evidence(pool: &PgPool, expected: ExpectedProjectEdge<'_>) {
    let row = sqlx::query(
        r#"
        SELECT
            edge.review_state,
            edge.confidence::float8 AS confidence,
            evidence.source_kind,
            evidence.source_id
        FROM graph_edges edge
        JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
        WHERE edge.source_node_id = $1
          AND edge.target_node_id = $2
          AND edge.relationship_type = $3
          AND evidence.source_kind = $4
          AND evidence.source_id = $5
        "#,
    )
    .bind(expected.source_node_id)
    .bind(expected.target_node_id)
    .bind(expected.relationship_type)
    .bind(expected.source_kind)
    .bind(expected.source_id)
    .fetch_one(pool)
    .await
    .expect("project edge with evidence");

    let review_state: String = row.try_get("review_state").expect("review state");
    let confidence: f64 = row.try_get("confidence").expect("confidence");
    let stored_source_kind: String = row.try_get("source_kind").expect("source kind");
    let stored_source_id: String = row.try_get("source_id").expect("source id");
    assert_eq!(review_state, expected.review_state);
    assert!((confidence - expected.confidence).abs() < f64::EPSILON);
    assert_eq!(stored_source_kind, expected.source_kind);
    assert_eq!(stored_source_id, expected.source_id);
}

async fn cleanup_project_graph_fixture(pool: &PgPool, project_id: &str) {
    sqlx::query("DELETE FROM graph_nodes WHERE node_id = $1")
        .bind(project_graph_node_id(project_id))
        .execute(pool)
        .await
        .expect("cleanup project graph node");
    sqlx::query("DELETE FROM projects WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .expect("cleanup graph project");
}

async fn assert_unknown_email_endpoint_projected(
    pool: &PgPool,
    email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
) {
    let message = message_fixture_by_provider_record_id(pool, provider_record_id).await;
    assert_message_edge_with_evidence(
        pool,
        "email_address",
        email_address,
        provider_record_id,
        relationship_type,
        &message,
    )
    .await;

    let person_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_nodes
        WHERE node_kind = 'person'
          AND (stable_key LIKE $1 OR properties->>'email_address' = $2)
        "#,
    )
    .bind(format!("%{email_address}%"))
    .bind(email_address)
    .fetch_one(pool)
    .await
    .expect("unknown email person node count");
    assert_eq!(person_count, 0);
}

async fn assert_message_edge_with_evidence(
    pool: &PgPool,
    source_node_kind: &str,
    source_email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
    message: &ProjectedMessageFixture,
) {
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
        JOIN communication_messages message ON message.message_id = evidence.source_id
        WHERE source.node_kind = $1
          AND (
              source.stable_key = $2
              OR source.properties->>'email_address' = $2
          )
          AND target.node_kind = 'message'
          AND target.properties->>'provider_record_id' = $3
          AND edge.relationship_type = $4
          AND evidence.source_kind = 'message'
          AND evidence.source_id = $5
          AND evidence.excerpt = $6
          AND evidence.metadata->>'raw_record_id' = $7
          AND evidence.metadata->>'provider_record_id' = $8
        "#,
    )
    .bind(source_node_kind)
    .bind(source_email_address)
    .bind(provider_record_id)
    .bind(relationship_type)
    .bind(&message.message_id)
    .bind(&message.subject)
    .bind(&message.raw_record_id)
    .bind(&message.provider_record_id)
    .fetch_one(pool)
    .await
    .expect("message edge evidence count");
    assert_eq!(evidence_count, 1);
}

async fn assert_message_edge_count(
    pool: &PgPool,
    source_node_kind: &str,
    source_email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
    expected_count: i64,
) {
    let edge_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        WHERE source.node_kind = $1
          AND (
              source.stable_key = $2
              OR source.properties->>'email_address' = $2
          )
          AND target.node_kind = 'message'
          AND target.properties->>'provider_record_id' = $3
          AND edge.relationship_type = $4
        "#,
    )
    .bind(source_node_kind)
    .bind(source_email_address)
    .bind(provider_record_id)
    .bind(relationship_type)
    .fetch_one(pool)
    .await
    .expect("message edge count");
    assert_eq!(edge_count, expected_count);
}

async fn assert_known_person_endpoint_projected(pool: &PgPool, suffix: u128) {
    let provider_record_id = format!("provider-graph-projection-{suffix}");
    let message = message_fixture_by_provider_record_id(pool, &provider_record_id).await;
    assert_message_edge_with_evidence(
        pool,
        "person",
        &format!("known-{suffix}@example.com"),
        &provider_record_id,
        "person_received_message",
        &message,
    )
    .await;
}

async fn message_fixture_by_provider_record_id(
    pool: &PgPool,
    provider_record_id: &str,
) -> ProjectedMessageFixture {
    let row = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT message_id, raw_record_id, provider_record_id, subject
        FROM communication_messages
        WHERE provider_record_id = $1
        "#,
    )
    .bind(provider_record_id)
    .fetch_one(pool)
    .await
    .expect("communication message fixture");

    ProjectedMessageFixture {
        message_id: row.0,
        raw_record_id: row.1,
        provider_record_id: row.2,
        subject: row.3,
    }
}

async fn assert_document_projected(pool: &PgPool, suffix: u128) {
    let document_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_nodes
        WHERE node_kind = 'document'
          AND stable_key = $1
          AND label = $2
          AND properties->>'document_kind' = 'markdown'
          AND properties->>'source_fingerprint' LIKE 'local-v1:markdown:%'
          AND properties ? 'imported_at'
        "#,
    )
    .bind(format!("doc_graph_projection_{suffix}"))
    .bind(format!("graph-projection-{suffix}.md"))
    .fetch_one(pool)
    .await
    .expect("projected document node count");
    assert_eq!(document_count, 1);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
