use serde_json::json;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_personas_app, delete_request_with_token, post_request_with_token,
    put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

macro_rules! persona_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = super::support::live_database_url(stringify!($name)).await
            else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_personas_app(&database_url).await;
            let persona_id = format!("person:nonexistent-{suffix}");
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/personas/{}/{}",
                        urlencoding_percent_encode(&persona_id),
                        $path_suffix
                    ),
                    $body,
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

persona_post_test!(
    persona_post_fingerprint,
    "fingerprint",
    json!({"fingerprint_data": "test-fingerprint-data"})
);
persona_post_test!(persona_post_favorite, "favorite", json!({}));
persona_post_test!(
    persona_post_investigate,
    "investigate",
    json!({"query": "background check"})
);
persona_post_test!(
    persona_post_fact,
    "facts",
    json!({"fact_type": "preference", "value": "Likes coffee", "confidence": 0.9})
);
persona_post_test!(
    persona_post_memory_card,
    "memory-cards",
    json!({"title": "Memory card", "content": "Test memory content"})
);
persona_post_test!(
    persona_post_preference,
    "preferences",
    json!({"key": "communication_style", "value": "direct"})
);
persona_post_test!(
    persona_post_timeline,
    "timeline",
    json!({"event_type": "meeting", "description": "Test meeting", "occurred_at": "2027-01-01T00:00:00Z"})
);

#[tokio::test]
async fn persona_put_notes() {
    let Some(database_url) = super::support::live_database_url("persona put notes").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_personas_app(&database_url).await;
    let persona_id = format!("person:nonexistent-{suffix}");
    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}/notes",
                urlencoding_percent_encode(&persona_id)
            ),
            json!({"notes": "Test notes content"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "notes status={}",
        response.status()
    );
}

#[tokio::test]
async fn persona_roles_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("persona roles post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_personas_app(&database_url).await;
    let persona_id = format!("person:nonexistent-{suffix}");
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/personas/{}/roles",
                urlencoding_percent_encode(&persona_id)
            ),
            json!({"role": "colleague", "organization": "TestCo"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "role post={}",
        response.status()
    );

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/personas/{}/roles/colleague",
                urlencoding_percent_encode(&persona_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "role delete={}",
        response.status()
    );
}

#[tokio::test]
async fn persona_interaction_contexts_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("persona interaction contexts post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_personas_app(&database_url).await;
    let persona_id = format!("person:nonexistent-{suffix}");
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/personas/{}/interaction-contexts",
                urlencoding_percent_encode(&persona_id)
            ),
            json!({"name": "Work Persona", "description": "Professional context"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "persona post={}",
        response.status()
    );

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/personas/{}/interaction-contexts/pers:fake",
                urlencoding_percent_encode(&persona_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "persona delete={}",
        response.status()
    );
}

#[tokio::test]
async fn persona_watchlist_toggle() {
    let Some(database_url) = super::support::live_database_url("persona watchlist toggle").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_personas_app(&database_url).await;
    let persona_id = format!("person:nonexistent-{suffix}");
    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/personas/{}/watchlist",
                urlencoding_percent_encode(&persona_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "watchlist toggle={}",
        response.status()
    );
}
