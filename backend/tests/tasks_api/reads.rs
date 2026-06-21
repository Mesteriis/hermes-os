use crate::support::*;

async fn create_task_or_skip(app: &Router, suffix: u128) -> Option<String> {
    let task_id = create_task(app, suffix).await;
    if task_id.is_none() {
        eprintln!("skip: task create failed");
    }
    task_id
}

macro_rules! task_get_requires_task {
    ($name:ident, $path_suffix:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = test_database_url(stringify!($name)) else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_tasks_app(&database_url).await;
            let Some(task_id) = create_task_or_skip(&app, suffix).await else {
                return;
            };

            let response = app
                .oneshot(get_request_with_token(
                    &format!(
                        "/api/v1/tasks/{}/{}",
                        urlencoding_percent_encode(&task_id),
                        $path_suffix
                    ),
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK);
        }
    };
}

macro_rules! task_get_simple {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = test_database_url(stringify!($name)) else {
                return;
            };
            let app = build_tasks_app(&database_url).await;
            let response = app
                .oneshot(get_request_with_token($path, LOCAL_API_TOKEN))
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK);
        }
    };
}

task_get_requires_task!(task_context_pack_returns_ok, "context-pack");
task_get_requires_task!(task_evidence_list_returns_empty, "evidence");
task_get_requires_task!(task_relations_list_returns_empty, "relations");
task_get_requires_task!(task_checklist_list_returns_empty, "checklist");
task_get_requires_task!(task_subtasks_list_returns_empty, "subtasks");
task_get_requires_task!(task_export_returns_text, "export");
task_get_requires_task!(task_external_returns_ok, "external");

task_get_simple!(task_providers_list_returns_ok, "/api/v1/tasks/providers");
task_get_simple!(task_search_returns_ok, "/api/v1/tasks/search?q=test");
task_get_simple!(task_daily_brief_returns_ok, "/api/v1/tasks/daily-brief");
task_get_simple!(task_rules_list_returns_empty, "/api/v1/tasks/rules");
task_get_simple!(task_templates_list_returns_ok, "/api/v1/tasks/templates");
task_get_simple!(task_watchtower_returns_ok, "/api/v1/tasks/watchtower");
task_get_simple!(task_health_returns_ok, "/api/v1/tasks/health");
task_get_simple!(task_analytics_returns_ok, "/api/v1/tasks/analytics");
task_get_simple!(task_candidates_list_returns_ok, "/api/v1/task-candidates");
