use std::env;

use serde_json::json;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, delete_request_with_token, get_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn calendar_accounts_crud_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar accounts CRUD test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let acct_name = format!("API Cal Acct {suffix}");

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": &acct_name, "email": format!("cal-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let created = json_body(response).await;
    let account_id = created["account_id"]
        .as_str()
        .expect("account_id")
        .to_owned();
    assert_eq!(created["provider"], json!("google"));

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            json!({"account_name": format!("Updated {acct_name}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated = json_body(response).await;
    assert_eq!(
        updated["account_name"],
        json!(format!("Updated {acct_name}"))
    );

    let response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let deleted = json_body(response).await;
    assert_eq!(deleted["deleted"], json!(true));
}

#[tokio::test]
async fn calendar_accounts_list_returns_items() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar accounts list test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("List Acct {suffix}"), "email": format!("list-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert!(!body["items"].as_array().expect("items").is_empty());
}
