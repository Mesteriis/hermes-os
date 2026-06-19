use serde_json::json;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, delete_request_with_token, post_request_with_token,
    put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

macro_rules! person_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = super::support::live_database_url(stringify!($name)).await
            else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_persons_app(&database_url).await;
            let pid = format!("person:nonexistent-{suffix}");
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/persons/{}/{}",
                        urlencoding_percent_encode(&pid),
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

person_post_test!(
    person_post_fingerprint,
    "fingerprint",
    json!({"fingerprint_data": "test-fingerprint-data"})
);
person_post_test!(person_post_favorite, "favorite", json!({}));
person_post_test!(
    person_post_investigate,
    "investigate",
    json!({"query": "background check"})
);
person_post_test!(
    person_post_fact,
    "facts",
    json!({"fact_type": "preference", "value": "Likes coffee", "confidence": 0.9})
);
person_post_test!(
    person_post_memory_card,
    "memory-cards",
    json!({"title": "Memory card", "content": "Test memory content"})
);
person_post_test!(
    person_post_preference,
    "preferences",
    json!({"key": "communication_style", "value": "direct"})
);
person_post_test!(
    person_post_timeline,
    "timeline",
    json!({"event_type": "meeting", "description": "Test meeting", "occurred_at": "2027-01-01T00:00:00Z"})
);

#[tokio::test]
async fn person_put_notes() {
    let Some(database_url) = super::support::live_database_url("person put notes").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .oneshot(put_request_with_token(
            &format!("/api/v1/persons/{}/notes", urlencoding_percent_encode(&pid)),
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
async fn person_roles_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("person roles post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{}/roles", urlencoding_percent_encode(&pid)),
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
                "/api/v1/persons/{}/roles/colleague",
                urlencoding_percent_encode(&pid)
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
async fn person_persona_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("person persona post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/personas",
                urlencoding_percent_encode(&pid)
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
                "/api/v1/persons/{}/personas/pers:fake",
                urlencoding_percent_encode(&pid)
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
async fn person_watchlist_toggle() {
    let Some(database_url) = super::support::live_database_url("person watchlist toggle").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/watchlist",
                urlencoding_percent_encode(&pid)
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
