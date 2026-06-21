use crate::support::*;

#[tokio::test]
async fn graph_summary_returns_empty_state_for_empty_database() {
    let Some(context) = live_graph_api_context("empty summary").await else {
        return;
    };

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/graph/summary",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["node_counts"], json!([]));
    assert_eq!(body["edge_counts"], json!([]));
    assert_eq!(body["evidence_count"], json!(0));
    assert_eq!(body["latest_projection_at"], Value::Null);
    assert_eq!(body["is_empty"], json!(true));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_nodes_returns_connected_picker_nodes_first() {
    let Some(context) = live_graph_api_context("node picker").await else {
        return;
    };
    let suffix = unique_suffix();
    let connected_person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:connected-picker:{suffix}"),
            format!("Connected Picker {suffix}"),
        ))
        .await
        .expect("connected person node");
    let connected_email = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("connected-picker-{suffix}@example.com"),
            format!("connected-picker-{suffix}@example.com"),
        ))
        .await
        .expect("connected email node");
    let disconnected = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:disconnected-picker:{suffix}"),
            format!("Disconnected Picker {suffix}"),
        ))
        .await
        .expect("disconnected node");
    context
        .store
        .upsert_edge_with_evidence(
            &NewGraphEdge::new(
                connected_person.node_id.clone(),
                connected_email.node_id.clone(),
                RelationshipType::PersonHasEmailAddress,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &[NewGraphEvidence::new(
                GraphEvidenceSourceKind::Person,
                format!("person-source:{suffix}"),
            )],
        )
        .await
        .expect("connected picker edge");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/graph/nodes?limit=2",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body.as_array().expect("node array");
    let node_ids = nodes
        .iter()
        .map(|node| node["node_id"].as_str().expect("node id"))
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 2);
    assert!(node_ids.contains(&connected_person.node_id.as_str()));
    assert!(node_ids.contains(&connected_email.node_id.as_str()));
    assert!(!node_ids.contains(&disconnected.node_id.as_str()));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_search_returns_matching_nodes() {
    let Some(context) = live_graph_api_context("search").await else {
        return;
    };
    let suffix = unique_suffix();
    let alex = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:alex:{suffix}"),
            format!("Alex Morgan {suffix}"),
        ))
        .await
        .expect("alex node");
    context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person:blair:{suffix}"),
            format!("Blair Morgan {suffix}"),
        ))
        .await
        .expect("blair node");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/graph/search?q=alex",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body.as_array().expect("node array");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["node_id"], json!(alex.node_id));
    assert_eq!(nodes[0]["label"], json!(alex.label));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_search_rejects_empty_query() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/graph/search?q=",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_graph_query",
            "message": "q must not be empty"
        })
    );
}
