use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Utc};
use hermes_hub_backend::tasks::{NewTask, TaskStore, TaskUpdate, TaskListQuery};
use hermes_hub_backend::task_core::{
    TaskContextPackStore, TaskEvidenceStore, TaskRelationStore,
    TaskChecklistStore, TaskSubtaskStore, TaskProviderStore,
};
use hermes_hub_backend::task_intelligence::TaskIntelligenceService;
use hermes_hub_backend::task_brain::TaskBrainService;
use hermes_hub_backend::task_health::TaskWatchtowerService;
use hermes_hub_backend::task_rules::{TaskRuleStore, TaskTemplateStore};
use hermes_hub_backend::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

fn unique_suffix() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
}

async fn live_pool() -> Option<PgPool> {
    let Some(url) = env::var("HERMES_TEST_DATABASE_URL").ok() else { eprintln!("skip: no DB"); return None; };
    let db = Database::connect(Some(&url)).await.expect("connect");
    Some(db.pool().expect("pool").clone())
}

fn disconnected_pool() -> PgPool {
    PgPoolOptions::new().connect_lazy("postgres://x:x@127.0.0.1:1/db").expect("lazy")
}

// ── Task CRUD ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_crud_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    let task = store.create(&NewTask {
        title: format!("Test {suffix}"), description: Some("desc".into()),
        source_type: Some("manual".into()), hermes_status: Some("new".into()),
        priority_score: Some(0.8), ..Default::default()
    }).await.expect("create");
    assert!(task.task_id.starts_with("task:v1:"));
    assert_eq!(task.hermes_status, "new");

    let fetched = store.get(&task.task_id).await.expect("get").expect("exists");
    assert_eq!(fetched.priority_score, Some(0.8));

    let updated = store.update(&task.task_id, &TaskUpdate {
        hermes_status: Some("in_progress".into()), priority_score: Some(0.9), ..Default::default()
    }).await.expect("update");
    assert_eq!(updated.hermes_status, "in_progress");

    store.set_status(&task.task_id, "done").await.expect("set status");
    let done = store.get(&task.task_id).await.expect("get").expect("exists");
    assert_eq!(done.hermes_status, "done");
    assert!(done.completed_at.is_some());

    store.archive(&task.task_id).await.expect("archive");
    let archived = store.get(&task.task_id).await.expect("get").expect("exists");
    assert_eq!(archived.hermes_status, "archived");
}

// ── Task List ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_list_filtering_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    store.create(&NewTask { title: format!("Active {suffix}"), source_type: Some("manual".into()), ..Default::default() }).await.expect("create");
    store.create(&NewTask { title: format!("Done {suffix}"), source_type: Some("manual".into()), hermes_status: Some("done".into()), ..Default::default() }).await.expect("create");

    let all = store.list(&TaskListQuery { limit: Some(100), ..Default::default() }).await.expect("list");
    assert!(all.len() >= 2);

    let active = store.list(&TaskListQuery { status: Some("new".into()), limit: Some(100), ..Default::default() }).await.expect("list");
    assert!(active.iter().any(|t| t.title.contains(&format!("Active {suffix}"))));
}

// ── Context Pack ──────────────────────────────────────────────────────────

#[tokio::test]
async fn task_context_pack_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskStore::new(pool.clone());
    let ctx = TaskContextPackStore::new(pool);
    let suffix = unique_suffix();
    let task = store.create(&NewTask { title: format!("Ctx {suffix}"), source_type: Some("manual".into()), ..Default::default() }).await.expect("create");

    let pack = ctx.upsert(&task.task_id, Some("summary"), json!(["Q1"]), json!(["blocker"]), json!(["risk"]), Some("next step")).await.expect("upsert");
    assert_eq!(pack.summary.as_deref(), Some("summary"));

    let fetched = ctx.get(&task.task_id).await.expect("get").expect("exists");
    assert_eq!(fetched.summary.as_deref(), Some("summary"));
}

// ── Evidence ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_evidence_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskStore::new(pool.clone());
    let ev = TaskEvidenceStore::new(pool);
    let suffix = unique_suffix();
    let task = store.create(&NewTask { title: format!("Ev {suffix}"), source_type: Some("email".into()), ..Default::default() }).await.expect("create");

    let evidence = ev.add(&task.task_id, "email", "msg-1", Some("Please do this"), Some(0.9)).await.expect("add");
    assert_eq!(evidence.source_type, "email");
    assert_eq!(evidence.confidence, 0.9);

    let list = ev.list(&task.task_id).await.expect("list");
    assert_eq!(list.len(), 1);
}

// ── Relations ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_relations_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskStore::new(pool.clone());
    let rel = TaskRelationStore::new(pool);
    let suffix = unique_suffix();
    let task = store.create(&NewTask { title: format!("Rel {suffix}"), source_type: Some("manual".into()), ..Default::default() }).await.expect("create");

    rel.link(&task.task_id, "person", "p1", "blocks").await.expect("link");
    let list = rel.list(&task.task_id).await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].relation_type, "blocks");
}

// ── Checklist ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_checklist_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskStore::new(pool.clone());
    let cl = TaskChecklistStore::new(pool);
    let suffix = unique_suffix();
    let task = store.create(&NewTask { title: format!("Cl {suffix}"), source_type: Some("manual".into()), ..Default::default() }).await.expect("create");

    cl.set(&task.task_id, json!([{"text":"Step 1","done":false}]), "manual").await.expect("set");
    let fetched = cl.get(&task.task_id).await.expect("get").expect("exists");
    assert_eq!(fetched.source, "manual");
}

// ── Subtasks ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_subtasks_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskStore::new(pool.clone());
    let sub = TaskSubtaskStore::new(pool);
    let suffix = unique_suffix();
    let parent = store.create(&NewTask { title: format!("Parent {suffix}"), source_type: Some("manual".into()), ..Default::default() }).await.expect("create");
    let child = store.create(&NewTask { title: format!("Child {suffix}"), source_type: Some("manual".into()), ..Default::default() }).await.expect("create");

    sub.add(&parent.task_id, &child.task_id, 0).await.expect("add");
    let list = sub.list(&parent.task_id).await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].child_task_id, child.task_id);
}

// ── Providers ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_providers_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let store = TaskProviderStore::new(pool);
    let suffix = unique_suffix();
    store.create("jira", &format!("Jira {suffix}")).await.expect("create");
    let list = store.list().await.expect("list");
    assert!(list.iter().any(|p| p.provider == "jira"));
}

// ── Rules and Templates ───────────────────────────────────────────────────

#[tokio::test]
async fn task_rules_and_templates_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let rules = TaskRuleStore::new(pool.clone());
    let tmpl = TaskTemplateStore::new(pool);
    let suffix = unique_suffix();

    let rule = rules.create(&format!("Rule {suffix}"), None, json!({"action":"auto_prioritize"}), Some("suggest_only")).await.expect("create");
    assert!(rule.rule_id.starts_with("taskrule:v1:"));

    rules.delete(&rule.rule_id).await.expect("delete");

    let templates = tmpl.list().await.expect("list");
    assert!(templates.iter().any(|t| t.template_id == "bug"));
}

// ── Intelligence ──────────────────────────────────────────────────────────

#[test]
fn task_intelligence_priority() {
    let now = Utc::now();
    let high = TaskIntelligenceService::calculate_priority(Some(now + Duration::hours(2)), true, true, true, true, false, false);
    let low = TaskIntelligenceService::calculate_priority(Some(now + Duration::days(30)), false, false, false, false, false, false);
    assert!(high > low);
    assert!(high > 0.5);
}

#[test]
fn task_intelligence_risk() {
    let high = TaskIntelligenceService::calculate_risk(true, true, true, true, true, "urgent fix");
    let low = TaskIntelligenceService::calculate_risk(false, false, false, false, false, "update docs");
    assert!(high > low);
    assert!(high > 0.5);
}

#[test]
fn task_intelligence_readiness() {
    let full = TaskIntelligenceService::calculate_readiness(true, true, true, true, true, true);
    assert!((full - 1.0).abs() < 0.01);
    let none = TaskIntelligenceService::calculate_readiness(false, false, false, false, false, false);
    assert!((none - 0.0).abs() < 0.01);
}

#[test]
fn task_intelligence_next_action() {
    assert!(TaskIntelligenceService::suggest_next_action("new", false, false, None).contains("Review"));
    assert!(TaskIntelligenceService::suggest_next_action("waiting", false, false, Some("John")).contains("Follow"));
    assert!(TaskIntelligenceService::suggest_next_action("done", false, false, None).contains("Archive"));
}

// ── Health ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_health_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let overdue = TaskWatchtowerService::overdue(&pool).await.expect("overdue");
    assert!(overdue.is_object());

    let stale = TaskWatchtowerService::stale_tasks(&pool, 14).await.expect("stale");
    assert!(stale.is_object());

    let no_ctx = TaskWatchtowerService::without_context(&pool).await.expect("no ctx");
    assert!(no_ctx.is_object());

    let wl = TaskWatchtowerService::workload(&pool).await.expect("workload");
    assert!(wl.is_object());
}

// ── Brain ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_brain_against_postgres() {
    let Some(pool) = live_pool().await else { return; };
    let brief = TaskBrainService::daily_brief(&pool).await.expect("brief");
    assert!(brief.is_object());

    let search = TaskBrainService::search_tasks(&pool, "test").await.expect("search");
    assert!(search.is_object());
}

// ── Sync Export ───────────────────────────────────────────────────────────

#[test]
fn task_export_markdown() {
    let md = hermes_hub_backend::task_sync::export_task_md("Test", Some("desc"), "in_progress", Some("because"), Some("done"));
    assert!(md.contains("# Test"));
    assert!(md.contains("in_progress"));
    assert!(md.contains("because"));
}

#[test]
fn task_export_json() {
    let json = hermes_hub_backend::task_sync::export_task_json("Test", Some("desc"), "done", Some(0.8), Some("2026-01-01"));
    assert_eq!(json["title"], "Test");
    assert_eq!(json["priority"], 0.8);
}

// ── Disconnected pool smoke ───────────────────────────────────────────────

#[tokio::test]
async fn disconnected_task_stores() {
    let pool = disconnected_pool();
    let _ = TaskStore::new(pool.clone());
    let _ = TaskContextPackStore::new(pool.clone());
    let _ = TaskEvidenceStore::new(pool.clone());
    let _ = TaskRelationStore::new(pool.clone());
    let _ = TaskChecklistStore::new(pool.clone());
    let _ = TaskSubtaskStore::new(pool);
}
