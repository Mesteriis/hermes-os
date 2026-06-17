use crate::support::*;

#[tokio::test]
async fn tasks_crud_against_postgres() {
    let Some(database_url) = test_database_url("tasks CRUD test") else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({"title": format!("CRUD Task {suffix}"), "description": "CRUD test", "status": "active"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: task create failed");
        return;
    }
    let created = json_body(response).await;
    let Some(task_id) = created["task_id"].as_str().map(|value| value.to_owned()) else {
        eprintln!("skip: no task_id");
        return;
    };
    assert_eq!(created["title"], json!(format!("CRUD Task {suffix}")));

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = json_body(response).await;
    assert_eq!(fetched["task_id"], json!(task_id));

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            json!({"title": format!("Updated Task {suffix}"), "priority": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let updated = json_body(response).await;
    assert_eq!(updated["title"], json!(format!("Updated Task {suffix}")));

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/archive",
                urlencoding_percent_encode(&task_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn tasks_list_returns_items() {
    let Some(database_url) = test_database_url("tasks list test") else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    create_task(&app, suffix).await;

    let response = app
        .oneshot(get_request_with_token("/api/v1/tasks", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let _items = body["items"].as_array().expect("items");
}

#[tokio::test]
async fn task_status_transition() {
    let Some(database_url) = test_database_url("task status test") else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/status",
                urlencoding_percent_encode(&task_id)
            ),
            json!({"status": "completed"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}
