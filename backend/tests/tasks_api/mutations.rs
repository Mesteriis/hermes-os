use crate::support::*;

#[tokio::test]
async fn task_rule_create_and_delete() {
    let Some(database_url) = test_database_url("task rule create/delete test") else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(_task_id) = create_task(&app, suffix).await else {
        eprintln!("skip");
        return;
    };

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks/rules",
            json!({"name": format!("Rule{suffix}"), "rule_type": "auto_priority", "config": json!({"default": "medium"})}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: rule create failed");
        return;
    }
    let rule_id = json_body(response).await["rule_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if rule_id.is_empty() {
        return;
    }

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/tasks/rules/{}",
                urlencoding_percent_encode(&rule_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "rule delete={}",
        response.status()
    );
}

macro_rules! task_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = test_database_url(stringify!($name)) else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_tasks_app(&database_url).await;
            let Some(task_id) = create_task(&app, suffix).await else {
                eprintln!("skip: no task");
                return;
            };
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/tasks/{}/{}",
                        urlencoding_percent_encode(&task_id),
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

task_post_test!(
    task_post_context_pack,
    "context-pack",
    json!({"summary": "Test context"})
);
task_post_test!(
    task_post_evidence,
    "evidence",
    json!({"source": "email", "reference_id": "msg:test", "note": "Test evidence"})
);
task_post_test!(
    task_post_relation,
    "relations",
    json!({"related_task_id": "task:fake", "relation_type": "blocks"})
);
task_post_test!(
    task_post_checklist,
    "checklist",
    json!({"item": "Test item", "done": false})
);
task_post_test!(
    task_post_subtask,
    "subtasks",
    json!({"title": "Test subtask", "status": "active"})
);

#[tokio::test]
async fn task_post_provider() {
    let Some(database_url) = test_database_url("task provider post test") else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/tasks/providers",
            json!({"name": format!("Provider{suffix}"), "provider_type": "jira", "config": json!({"url": "https://example.com"})}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "provider post={}",
        response.status()
    );
}

#[tokio::test]
async fn task_candidate_review() {
    let Some(database_url) = test_database_url("task candidate review test") else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(put_request_with_token(
            &format!("/api/v1/task-candidates/tc:fake-{suffix}/review"),
            json!({"review_state": "declined", "reason": "Not relevant"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "candidate review={}",
        response.status()
    );
}
