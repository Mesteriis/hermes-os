use axum::http::StatusCode;
use tower::ServiceExt;

use super::support::{LOCAL_API_TOKEN, build_personas_app, get_request_with_token, unique_suffix};

#[tokio::test]
async fn person_search_returns_ok() {
    let Some(database_url) = super::support::live_database_url("person search").await else {
        return;
    };
    let app = build_personas_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/personas/search?q=alex",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

macro_rules! person_endpoint_test {
    ($name:ident, $path_suffix:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = super::support::live_database_url(stringify!($name)).await
            else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_personas_app(&database_url).await;
            let path = format!(
                "/api/v1/personas/person:nonexistent-{}{}",
                suffix, $path_suffix
            );
            let response = app
                .oneshot(get_request_with_token(&path, LOCAL_API_TOKEN))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "status={}",
                response.status()
            );
        }
    };
}

person_endpoint_test!(persona_identities_list, "/identities");
person_endpoint_test!(persona_roles_list, "/roles");
person_endpoint_test!(persona_interaction_contexts_list, "/interaction-contexts");
person_endpoint_test!(persona_facts_list, "/facts");
person_endpoint_test!(persona_memory_cards_list, "/memory-cards");
person_endpoint_test!(persona_preferences_list, "/preferences");
person_endpoint_test!(person_timeline_list, "/timeline");
person_endpoint_test!(persona_snapshots_list, "/snapshots");
person_endpoint_test!(person_history_diff, "/history-diff");
person_endpoint_test!(person_enrichment_list, "/enrichment");
person_endpoint_test!(persona_expertise_list, "/expertise");
person_endpoint_test!(persona_promises_list, "/promises");
person_endpoint_test!(persona_risks_list, "/risks");
person_endpoint_test!(person_health_get, "/health");
person_endpoint_test!(person_dossier_get, "/dossier");
person_endpoint_test!(person_meeting_prep_get, "/meeting-prep");
person_endpoint_test!(person_analytics_get, "/analytics");
person_endpoint_test!(person_export_get, "/export");
person_endpoint_test!(persona_identity_detail, "/identity");

#[tokio::test]
async fn personas_health_returns_ok() {
    let Some(database_url) = super::support::live_database_url("persons health").await else {
        return;
    };
    let app = build_personas_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/personas/health",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn personas_watchlist_returns_ok() {
    let Some(database_url) = super::support::live_database_url("persons watchlist").await else {
        return;
    };
    let app = build_personas_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/personas/watchlist",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn identity_candidates_list_returns_ok() {
    let Some(database_url) = super::support::live_database_url("identity candidates").await else {
        return;
    };
    let app = build_personas_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/identity-candidates",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn persona_expertise_search() {
    let Some(database_url) = super::support::live_database_url("person expertise search").await
    else {
        return;
    };
    let app = build_personas_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/personas/search/expertise?q=rust",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "expertise search={}",
        response.status()
    );
}
