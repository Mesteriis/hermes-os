use super::support::{
    assert_document_projected, assert_known_person_endpoint_projected, assert_message_edge_count,
    assert_message_edge_with_evidence, assert_unknown_email_endpoint_projected,
    graph_counts_for_suffix, live_projection_context, seed_message,
    seed_person_message_and_document, unique_suffix,
};

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
