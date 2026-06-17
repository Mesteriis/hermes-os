use axum::http::StatusCode;
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, build_persons_app_with_database, get_request_with_token,
    json_body, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn person_dossier_get_persists_snapshot_and_review_state_against_postgres() {
    let Some(database_url) = super::support::live_database_url("dossier snapshot API").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("dossier-snapshot-{suffix}@example.com"))
        .await
        .expect("upsert dossier persona");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/persons/{}/dossier",
                urlencoding_percent_encode(&person.person_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let snapshot_id = body["dossier_snapshot_id"]
        .as_str()
        .expect("dossier snapshot id")
        .to_owned();
    assert_eq!(body["review_state"], "suggested");
    assert_eq!(body["person"]["person_id"], person.person_id);

    let row = sqlx::query(
        r#"
        SELECT persona_id, review_state, dossier
        FROM persona_dossier_snapshots
        WHERE dossier_snapshot_id = $1
        "#,
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("stored dossier snapshot");
    assert_eq!(
        row.try_get::<String, _>("persona_id").expect("persona id"),
        person.person_id
    );
    assert_eq!(
        row.try_get::<String, _>("review_state")
            .expect("review state"),
        "suggested"
    );
    let stored_dossier = row.try_get::<Value, _>("dossier").expect("dossier json");
    assert_eq!(stored_dossier["person"]["person_id"], person.person_id);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/persons/{}/dossier/review",
                urlencoding_percent_encode(&person.person_id)
            ),
            json!({ "review_state": "user_confirmed" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier review response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["dossier_snapshot_id"], snapshot_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert!(body["reviewed_at"].is_string());
}

#[tokio::test]
async fn person_detail_not_found_returns_404() {
    let Some(database_url) = super::support::live_database_url("person detail").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/persons/person:nonexistent-{suffix}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn person_owner_get_and_put_uses_owner_persona_against_postgres() {
    let Some(database_url) = super::support::live_database_url("owner persona API").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE persons SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonProjectionStore::new(pool);
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("owner-api-{suffix}@example.com"))
        .await
        .expect("upsert owner candidate");
    let other = store
        .upsert_email_person(&format!("not-owner-api-{suffix}@example.com"))
        .await
        .expect("upsert non-owner candidate");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/persons/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("initial owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body["owner_persona"].is_null());

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            "/api/v1/persons/owner",
            json!({ "person_id": owner.person_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("set owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["person_id"], owner.person_id);
    assert_eq!(body["owner_persona"]["is_self"], true);
    assert_eq!(body["owner_persona"]["persona_type"], "human");

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["person_id"], owner.person_id);
    assert_ne!(body["owner_persona"]["person_id"], other.person_id);
}
