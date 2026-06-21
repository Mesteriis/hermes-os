use axum::http::StatusCode;
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, build_persons_app_with_database, get_request_with_token,
    json_body, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn persons_list_returns_ok() {
    let Some(database_url) = super::support::live_database_url("persons list").await else {
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token("/api/v1/persons", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn personas_routes_return_persona_native_schema_against_postgres() {
    let Some(database_url) = super::support::live_database_url("personas route").await else {
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
        .upsert_email_person(&format!("persona-native-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    store
        .set_owner_persona(&owner.person_id)
        .await
        .expect("set owner persona");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/personas?limit=20",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("personas list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items array");
    assert!(
        items
            .iter()
            .any(|item| item["persona_id"] == owner.person_id && item["is_self"] == true),
        "personas list should include owner Persona: {body}"
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona detail response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.person_id);
    assert_eq!(body["persona_type"], "human");
    assert_eq!(body["is_self"], true);
    assert_eq!(body["identity"]["display_name"], owner.display_name);
    assert_eq!(body["identity"]["email_address"], owner.email_address);
    assert_eq!(body["communication"]["primary_email"], owner.email_address);
    assert_eq!(body["compatibility"]["legacy_person_id"], owner.person_id);
    assert_eq!(body["compatibility"]["legacy_route"], "/api/v1/persons");
}

#[tokio::test]
async fn personas_put_updates_compatibility_projection_against_postgres() {
    let Some(database_url) = super::support::live_database_url("personas write route").await else {
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
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("persona-native-write-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    let previous_owner = store
        .upsert_email_person(&format!("persona-native-write-prev-{suffix}@example.com"))
        .await
        .expect("upsert previous owner persona");
    store
        .set_owner_persona(&previous_owner.person_id)
        .await
        .expect("set previous owner persona");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            json!({
                "identity": {
                    "display_name": "Owner Persona"
                },
                "is_self": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona update response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.person_id);
    assert_eq!(body["identity"]["display_name"], "Owner Persona");
    assert_eq!(body["is_self"], true);

    let row = sqlx::query(
        r#"
        SELECT display_name, is_self
        FROM persons
        WHERE person_id = $1
        "#,
    )
    .bind(&owner.person_id)
    .fetch_one(&pool)
    .await
    .expect("updated persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "Owner Persona"
    );
    assert!(row.try_get::<bool, _>("is_self").unwrap());

    let persona_update_observation: (String, String) = sqlx::query_as(
        "SELECT link.observation_id, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'persona'
           AND link.entity_id = $1
           AND link.relationship_kind = 'persona_update'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&owner.person_id)
    .fetch_one(&pool)
    .await
    .expect("persona update observation link");
    assert!(!persona_update_observation.0.is_empty());
    assert_eq!(persona_update_observation.1, "PERSON_MUTATION");

    let previous_is_self: bool =
        sqlx::query_scalar("SELECT is_self FROM persons WHERE person_id = $1")
            .bind(&previous_owner.person_id)
            .fetch_one(&pool)
            .await
            .expect("previous owner row");
    assert!(!previous_is_self);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            json!({ "is_self": false }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona unset owner response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
