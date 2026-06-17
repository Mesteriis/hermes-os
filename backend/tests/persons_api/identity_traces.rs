use axum::http::StatusCode;
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, get_request_with_token, json_body, post_request_with_token,
    put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn identity_traces_create_list_and_attach_unattached_trace() {
    let Some(database_url) = super::support::live_database_url("identity traces API").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;

    let create = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/identity-traces",
            json!({
                "identity_type": "message_participant",
                "identity_value": format!("message:v1:{suffix}:api-unattached"),
                "source": "communication_projection"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create trace response");
    assert_eq!(create.status(), StatusCode::OK);
    let create_body = json_body(create).await;
    assert_eq!(create_body["person_id"], Value::Null);
    assert_eq!(create_body["identity_type"], "message_participant");
    assert_eq!(create_body["source"], "communication_projection");
    let identity_id = create_body["id"].as_str().expect("identity id").to_owned();

    let list = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/identity-traces?status=unattached",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("list traces response");
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = json_body(list).await;
    let items = list_body["items"].as_array().expect("items");
    assert!(items.iter().any(|item| item["id"] == identity_id
        && item["person_id"] == Value::Null
        && item["identity_type"] == "message_participant"));

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool);
    let person = person_store
        .upsert_email_person(&format!("identity-trace-api-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let attach = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/identity-traces/{}/assignment",
                urlencoding_percent_encode(&identity_id)
            ),
            json!({ "person_id": person.person_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("attach trace response");
    assert_eq!(attach.status(), StatusCode::OK);
    let attach_body = json_body(attach).await;
    assert_eq!(attach_body["id"], identity_id);
    assert_eq!(attach_body["person_id"], person.person_id);
    assert_eq!(attach_body["status"], "active");
}
