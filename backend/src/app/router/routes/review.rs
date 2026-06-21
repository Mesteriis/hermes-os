use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/review/items", get(get_v1_review_items))
        .route("/api/v1/review/items", post(post_v1_review_items))
        .route(
            "/api/v1/review/items/{review_item_id}/approve",
            post(post_v1_review_item_approve),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/dismiss",
            post(post_v1_review_item_dismiss),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/take",
            post(post_v1_review_item_take),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/archive",
            post(post_v1_review_item_archive),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/promote",
            post(post_v1_review_item_promote),
        )
        .route("/api/v1/obligations", get(get_v1_obligations))
        .route(
            "/api/v1/obligations/{obligation_id}/review",
            put(put_v1_obligation_review),
        )
        .route("/api/v1/decisions", get(get_v1_decisions))
        .route(
            "/api/v1/decisions/{decision_id}/review",
            put(put_v1_decision_review),
        )
        .route("/api/v1/relationships", get(get_v1_relationships))
        .route(
            "/api/v1/relationships/{relationship_id}/review",
            put(put_v1_relationship_review),
        )
        .route("/api/v1/contradictions", get(get_v1_contradictions))
        .route(
            "/api/v1/contradictions/{observation_id}/review",
            put(put_v1_contradiction_review),
        )
}
