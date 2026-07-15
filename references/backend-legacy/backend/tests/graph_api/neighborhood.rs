use crate::support::*;

#[tokio::test]
async fn graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence() {
    let Some(context) = live_graph_api_context("neighborhood").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Persona,
            format!("person:alex-neighborhood:{suffix}"),
            format!("Alex Neighborhood {suffix}"),
        ))
        .await
        .expect("person node");
    let email = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("alex-neighborhood-{suffix}@example.com"),
            format!("alex-neighborhood-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let edge = context
        .store
        .upsert_edge_with_evidence(
            &NewGraphEdge::new(
                person.node_id.clone(),
                email.node_id.clone(),
                RelationshipType::PersonaHasEmailAddress,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &[NewGraphEvidence::new(
                GraphEvidenceSourceKind::Persona,
                format!("person-source:{suffix}"),
            )
            .excerpt("confirmed by person record")
            .metadata(json!({"source": "graph_api_test"}))],
        )
        .await
        .expect("graph edge");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/graph/neighborhood?node_id={}&depth=1",
                person.node_id
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["selected_node"]["node_id"], json!(person.node_id));
    assert_eq!(body["selected_node"]["label"], json!(person.label));
    assert_eq!(
        body["edge_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT)
    );
    assert_eq!(body["truncated"], json!(false));
    assert_eq!(
        body["evidence_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
    );
    assert_eq!(body["evidence_truncated"], json!(false));

    let nodes = body["nodes"].as_array().expect("node array");
    assert_eq!(nodes.len(), 1);
    assert!(nodes.iter().all(|node| node["node_id"] != person.node_id));
    assert!(nodes.iter().any(|node| node["node_id"] == email.node_id));

    let edges = body["edges"].as_array().expect("edge array");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["edge_id"], json!(edge.edge_id));
    assert_eq!(edges[0]["source_node_id"], json!(person.node_id));
    assert_eq!(edges[0]["target_node_id"], json!(email.node_id));

    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0]["edge_id"], json!(edge.edge_id));
    assert_eq!(evidence[0]["source_kind"], json!("persona"));
    assert_eq!(
        evidence[0]["source_id"],
        json!(format!("person-source:{suffix}"))
    );
    assert_eq!(evidence[0]["excerpt"], json!("confirmed by person record"));
    assert_eq!(evidence[0]["metadata"], json!({"source": "graph_api_test"}));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_caps_depth_one_edges_nodes_and_evidence() {
    let Some(context) = live_graph_api_context("neighborhood cap").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Persona,
            format!("person:alex-neighborhood-cap:{suffix}"),
            format!("Alex Neighborhood Cap {suffix}"),
        ))
        .await
        .expect("person node");

    for index in 0..=EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT {
        let email = context
            .store
            .upsert_node(&NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                format!("alex-neighborhood-cap-{suffix}-{index:03}@example.com"),
                format!("alex-neighborhood-cap-{suffix}-{index:03}@example.com"),
            ))
            .await
            .expect("email node");
        context
            .store
            .upsert_edge_with_evidence(
                &NewGraphEdge::new(
                    person.node_id.clone(),
                    email.node_id,
                    RelationshipType::PersonaHasEmailAddress,
                    1.0,
                    GraphReviewState::SystemAccepted,
                ),
                &[NewGraphEvidence::new(
                    GraphEvidenceSourceKind::Persona,
                    format!("person-source:{suffix}:{index:03}"),
                )],
            )
            .await
            .expect("graph edge");
    }

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/graph/neighborhood?node_id={}", person.node_id),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body["nodes"].as_array().expect("node array");
    let edges = body["edges"].as_array().expect("edge array");
    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(
        body["edge_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT)
    );
    assert_eq!(body["truncated"], json!(true));
    assert_eq!(
        body["evidence_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
    );
    assert_eq!(body["evidence_truncated"], json!(false));
    assert_eq!(nodes.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);
    assert!(nodes.iter().all(|node| node["node_id"] != person.node_id));
    assert_eq!(edges.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);
    assert_eq!(evidence.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_caps_evidence_for_returned_edges() {
    let Some(context) = live_graph_api_context("neighborhood evidence cap").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Persona,
            format!("person:alex-neighborhood-evidence-cap:{suffix}"),
            format!("Alex Neighborhood Evidence Cap {suffix}"),
        ))
        .await
        .expect("person node");
    let email = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("alex-neighborhood-evidence-cap-{suffix}@example.com"),
            format!("alex-neighborhood-evidence-cap-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let evidence = (0..=EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
        .map(|index| {
            NewGraphEvidence::new(
                GraphEvidenceSourceKind::Persona,
                format!("person-source:{suffix}:{index:03}"),
            )
        })
        .collect::<Vec<_>>();
    context
        .store
        .upsert_edge_with_evidence(
            &NewGraphEdge::new(
                person.node_id.clone(),
                email.node_id,
                RelationshipType::PersonaHasEmailAddress,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &evidence,
        )
        .await
        .expect("graph edge with over-limit evidence");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/graph/neighborhood?node_id={}", person.node_id),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let edges = body["edges"].as_array().expect("edge array");
    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(body["truncated"], json!(false));
    assert_eq!(
        body["evidence_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT)
    );
    assert_eq!(body["evidence_truncated"], json!(true));
    assert_eq!(edges.len(), 1);
    assert_eq!(evidence.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT);

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_returns_not_found_when_node_id_is_missing() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/graph/neighborhood?depth=1",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "graph_node_not_found",
            "message": "graph node was not found"
        })
    );
}

#[tokio::test]
async fn graph_neighborhood_rejects_unsupported_depth() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/graph/neighborhood?node_id=graph:node:v1:persona:alex&depth=2",
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
            "message": "depth supports only 1"
        })
    );
}
