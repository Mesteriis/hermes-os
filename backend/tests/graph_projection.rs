use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::contacts::ContactProjectionStore;
use hermes_hub_backend::documents::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::graph_projection::GraphProjectionService;
use hermes_hub_backend::messages::{MessageProjectionStore, project_raw_email_message};
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn graph_projection_is_idempotent_for_v1_sources_against_postgres() {
    let Some(context) = live_projection_context("graph projection idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    seed_contact_message_and_document(&context, suffix).await;

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
    .bind(format!("contact:v1:email:%unknown-{suffix}%"))
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
    assert_known_contact_endpoint_projected(&context.pool, suffix).await;
    assert_document_projected(&context.pool, suffix).await;
}

struct LiveProjectionContext {
    pool: PgPool,
    contact_store: ContactProjectionStore,
    communication_store: CommunicationIngestionStore,
    message_store: MessageProjectionStore,
    document_store: DocumentImportStore,
    graph_projection: GraphProjectionService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GraphCounts {
    nodes: i64,
    edges: i64,
    evidence: i64,
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
        contact_store: ContactProjectionStore::new(pool.clone()),
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        document_store: DocumentImportStore::new(pool.clone()),
        graph_projection: GraphProjectionService::new(pool),
    })
}

async fn seed_contact_message_and_document(context: &LiveProjectionContext, suffix: u128) {
    context
        .contact_store
        .upsert_email_contact(&format!(" Known-{suffix}@Example.com "))
        .await
        .expect("upsert known contact");

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

    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_graph_projection_{suffix}"),
                &account_id,
                "email_message",
                format!("provider-graph-projection-{suffix}"),
                format!("sha256:graph-projection-{suffix}"),
                format!("batch_graph_projection_{suffix}"),
                json!({
                    "subject": format!("Graph projection subject {suffix}"),
                    "from": format!("Unknown-{suffix}@Example.com"),
                    "to": [
                        format!("known-{suffix}@example.com"),
                        format!("unknown-recipient-{suffix}@example.com")
                    ],
                    "body_text": "Graph projection body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"graph_projection_test"})),
        )
        .await
        .expect("record graph projection raw message");

    project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project raw graph projection message");

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

async fn assert_unknown_email_endpoint_projected(
    pool: &PgPool,
    email_address: &str,
    provider_record_id: &str,
    relationship_type: &str,
) {
    let edge_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        WHERE source.node_kind = 'email_address'
          AND source.stable_key = $1
          AND target.node_kind = 'message'
          AND target.properties->>'provider_record_id' = $2
          AND edge.relationship_type = $3
        "#,
    )
    .bind(email_address)
    .bind(provider_record_id)
    .bind(relationship_type)
    .fetch_one(pool)
    .await
    .expect("unknown email endpoint edge count");
    assert_eq!(edge_count, 1);

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

async fn assert_known_contact_endpoint_projected(pool: &PgPool, suffix: u128) {
    let edge_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges edge
        JOIN graph_nodes source ON source.node_id = edge.source_node_id
        JOIN graph_nodes target ON target.node_id = edge.target_node_id
        WHERE source.node_kind = 'person'
          AND source.properties->>'email_address' = $1
          AND target.node_kind = 'message'
          AND target.properties->>'provider_record_id' = $2
          AND edge.relationship_type = 'person_received_message'
        "#,
    )
    .bind(format!("known-{suffix}@example.com"))
    .bind(format!("provider-graph-projection-{suffix}"))
    .fetch_one(pool)
    .await
    .expect("known contact received message edge count");
    assert_eq!(edge_count, 1);
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
