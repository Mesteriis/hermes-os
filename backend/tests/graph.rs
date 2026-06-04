use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_hub_backend::graph::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, GraphStoreError,
    NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType,
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
        .upsert_edge_with_evidence(&edge, std::slice::from_ref(&evidence))
        .await
        .expect("first edge");
    let second = store
        .upsert_edge_with_evidence(&edge, &[evidence])
        .await
        .expect("second edge");

    assert_eq!(first.edge_id, second.edge_id);
    assert_eq!(
        first.relationship_type,
        RelationshipType::PersonHasEmailAddress
    );
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
    Some(GraphStore::new(
        database.pool().expect("configured pool").clone(),
    ))
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
