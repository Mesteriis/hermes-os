use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Utc};
use hermes_hub_backend::domains::calendar::brain::CalendarBrainService;
use hermes_hub_backend::domains::calendar::core::{
    ContextPackInput, EventAgendaStore, EventChecklistStore, EventContextPackStore,
    EventParticipantStore, EventRelationStore,
};
use hermes_hub_backend::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarEventListQuery, CalendarEventStore,
    CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};
use hermes_hub_backend::domains::calendar::health::CalendarWatchtowerService;
use hermes_hub_backend::domains::calendar::intelligence::CalendarIntelligenceService;
use hermes_hub_backend::domains::calendar::meetings::{MeetingNoteStore, MeetingOutcomeStore};
use hermes_hub_backend::domains::calendar::rules::CalendarRuleStore;
use hermes_hub_backend::domains::calendar::scheduling::{DeadlineStore, FocusBlockStore};
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn live_pool() -> Option<PgPool> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

fn disconnected_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://hermes:unused@127.0.0.1:1/hermes_hub")
        .expect("create lazy test pool")
}

// ── Calendar Account CRUD ──────────────────────────────────────────────────

#[tokio::test]
async fn calendar_account_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = CalendarAccountStore::new(pool);
    let suffix = unique_suffix();

    let acct = store
        .create("local", &format!("Test Account {suffix}"), None)
        .await
        .expect("create account");
    assert_eq!(acct.provider, "local");
    assert!(acct.account_id.starts_with("cal:v1:"));

    let fetched = store
        .get(&acct.account_id)
        .await
        .expect("get account")
        .expect("account exists");
    assert_eq!(fetched.account_name, acct.account_name);

    let update = CalendarAccountUpdate {
        account_name: Some(format!("Updated {suffix}")),
        ..Default::default()
    };
    let updated = store
        .update(&acct.account_id, &update)
        .await
        .expect("update account");
    assert_eq!(updated.account_name, format!("Updated {suffix}"));

    let list = store.list(Some("local")).await.expect("list accounts");
    assert!(list.iter().any(|a| a.account_id == acct.account_id));

    store
        .delete(&acct.account_id)
        .await
        .expect("delete account");
    assert!(
        store
            .get(&acct.account_id)
            .await
            .expect("get deleted")
            .is_none()
    );
}

// ── Calendar Source CRUD ───────────────────────────────────────────────────

#[tokio::test]
async fn calendar_source_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let src_store = CalendarSourceStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Src Test {suffix}"), None)
        .await
        .expect("create account");
    let src = src_store
        .create(
            &acct.account_id,
            "Work Calendar",
            Some("gcal-123"),
            Some("#4285f4"),
            Some("Europe/Madrid"),
        )
        .await
        .expect("create source");
    assert!(src.source_id.starts_with("src:v1:"));
    assert_eq!(src.name, "Work Calendar");
    assert_eq!(src.color.as_deref(), Some("#4285f4"));

    let list = src_store
        .list_by_account(&acct.account_id)
        .await
        .expect("list sources");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Work Calendar");
}

// ── Calendar Event CRUD ────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Event Test {suffix}"), None)
        .await
        .expect("create account");
    let now = Utc::now();
    let req = NewCalendarEvent {
        title: format!("Test Event {suffix}"),
        description: Some("Test description".into()),
        start_at: now,
        end_at: now + Duration::hours(1),
        account_id: Some(acct.account_id.clone()),
        event_type: Some("meeting".into()),
        ..Default::default()
    };

    let event = event_store.create(&req).await.expect("create event");
    assert!(event.event_id.starts_with("evt:v1:"));
    assert_eq!(event.title, format!("Test Event {suffix}"));
    assert_eq!(event.status, "scheduled");

    let fetched = event_store
        .get(&event.event_id)
        .await
        .expect("get event")
        .expect("event exists");
    assert_eq!(fetched.event_type.as_deref(), Some("meeting"));

    let update = CalendarEventUpdate {
        title: Some(format!("Updated {suffix}")),
        ..Default::default()
    };
    let updated = event_store
        .update(&event.event_id, &update)
        .await
        .expect("update event");
    assert_eq!(updated.title, format!("Updated {suffix}"));

    // List by time range
    let list = event_store
        .list(&CalendarEventListQuery {
            from: Some(now - Duration::hours(1)),
            to: Some(now + Duration::hours(2)),
            limit: Some(50),
            ..Default::default()
        })
        .await
        .expect("list events");
    assert!(list.iter().any(|e| e.event_id == event.event_id));

    event_store
        .delete(&event.event_id)
        .await
        .expect("delete event");
    assert!(
        event_store
            .get(&event.event_id)
            .await
            .expect("get deleted")
            .is_none()
    );
}

// ── Event Reschedule and Status ────────────────────────────────────────────

#[tokio::test]
async fn event_reschedule_and_status_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Status Test {suffix}"), None)
        .await
        .expect("create account");
    let now = Utc::now();
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Reschedule {suffix}"),
            start_at: now,
            end_at: now + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let new_start = now + Duration::hours(2);
    let rescheduled = event_store
        .reschedule(&event.event_id, new_start, new_start + Duration::hours(1))
        .await
        .expect("reschedule");
    assert_eq!(rescheduled.status, "rescheduled");

    event_store
        .set_status(&event.event_id, "cancelled")
        .await
        .expect("cancel");
    let cancelled = event_store
        .get(&event.event_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(cancelled.status, "cancelled");
}

// ── Event Participants ─────────────────────────────────────────────────────

#[tokio::test]
async fn event_participants_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let part_store = EventParticipantStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Part Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Participants {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let p = part_store
        .add(
            &event.event_id,
            &format!("john-{suffix}@test.com"),
            Some("John"),
            Some("required"),
            None,
            None,
        )
        .await
        .expect("add participant");
    assert_eq!(p.role, "required");
    assert_eq!(p.email, format!("john-{suffix}@test.com"));

    let list = part_store
        .list(&event.event_id)
        .await
        .expect("list participants");
    assert_eq!(list.len(), 1);
}

// ── Event Relations ────────────────────────────────────────────────────────

#[tokio::test]
async fn event_relations_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let rel_store = EventRelationStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Rel Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Relations {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let rel = rel_store
        .link(&event.event_id, "project", "proj-1", "related_to")
        .await
        .expect("link");
    assert_eq!(rel.entity_type, "project");

    let list = rel_store
        .list(&event.event_id)
        .await
        .expect("list relations");
    assert_eq!(list.len(), 1);
}

// ── Event Context Pack ─────────────────────────────────────────────────────

#[tokio::test]
async fn event_context_pack_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let ctx_store = EventContextPackStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Ctx Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Context {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let input = ContextPackInput {
        summary: Some("Test summary".into()),
        documents: json!([]),
        tasks: json!([]),
        open_questions: json!(["Q1"]),
        risks: json!(["No agenda"]),
        suggested_agenda: json!([]),
        suggested_actions: json!([]),
        ..Default::default()
    };
    let pack = ctx_store
        .upsert(&event.event_id, &input)
        .await
        .expect("upsert context pack");
    assert_eq!(pack.summary.as_deref(), Some("Test summary"));

    let fetched = ctx_store
        .get(&event.event_id)
        .await
        .expect("get pack")
        .expect("pack exists");
    assert_eq!(fetched.summary.as_deref(), Some("Test summary"));
}

// ── Event Agenda and Checklist ─────────────────────────────────────────────

#[tokio::test]
async fn event_agenda_and_checklist_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let agenda_store = EventAgendaStore::new(pool.clone());
    let cl_store = EventChecklistStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Agenda Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Agenda {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let agenda = agenda_store
        .set(&event.event_id, json!(["Item 1", "Item 2"]), "manual")
        .await
        .expect("set agenda");
    assert_eq!(agenda.source, "manual");

    let cl = cl_store
        .set(
            &event.event_id,
            json!([{"text": "Prepare docs", "done": false}]),
            "manual",
        )
        .await
        .expect("set checklist");
    assert_eq!(cl.source, "manual");

    let fetched_agenda = agenda_store
        .get(&event.event_id)
        .await
        .expect("get agenda")
        .expect("exists");
    let items = fetched_agenda.items.as_array().expect("array");
    assert_eq!(items.len(), 2);
}

// ── Meeting Notes and Outcomes ─────────────────────────────────────────────

#[tokio::test]
async fn meeting_notes_and_outcomes_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let note_store = MeetingNoteStore::new(pool.clone());
    let outcome_store = MeetingOutcomeStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Meeting Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Meeting {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let note = note_store
        .create(
            &event.event_id,
            "# Meeting Notes\n\nDiscussed scope.",
            Some("markdown"),
            Some("manual"),
        )
        .await
        .expect("create note");
    assert!(note.content.contains("Meeting Notes"));

    let outcome = outcome_store
        .add(
            &event.event_id,
            "decision",
            "Use Rust",
            Some("Decided to use Rust for backend"),
            None,
            None,
        )
        .await
        .expect("add outcome");
    assert_eq!(outcome.outcome_type, "decision");

    let notes = note_store.list(&event.event_id).await.expect("list notes");
    assert_eq!(notes.len(), 1);

    let outcomes = outcome_store
        .list(&event.event_id)
        .await
        .expect("list outcomes");
    assert_eq!(outcomes.len(), 1);
}

// ── Deadlines and Focus Blocks ─────────────────────────────────────────────

#[tokio::test]
async fn deadlines_and_focus_blocks_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let deadline_store = DeadlineStore::new(pool.clone());
    let fb_store = FocusBlockStore::new(pool);
    let suffix = unique_suffix();
    let now = Utc::now();

    let d = deadline_store
        .create(
            &format!("Deadline {suffix}"),
            now + Duration::days(7),
            Some("high"),
            None,
            None,
        )
        .await
        .expect("create deadline");
    assert_eq!(d.severity, "high");
    assert_eq!(d.status, "active");

    let deadlines = deadline_store.list(None, 50).await.expect("list deadlines");
    assert!(
        deadlines
            .iter()
            .any(|dl| dl.title == format!("Deadline {suffix}"))
    );

    let fb = fb_store
        .create(
            &format!("Focus {suffix}"),
            now,
            now + Duration::hours(2),
            Some("Deep work"),
            None,
            Some("high"),
        )
        .await
        .expect("create focus block");
    assert_eq!(fb.protection_level, "high");

    let blocks = fb_store
        .list(None, None, 50)
        .await
        .expect("list focus blocks");
    assert!(blocks.iter().any(|b| b.title == format!("Focus {suffix}")));
}

// ── Calendar Rules ─────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_rules_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = CalendarRuleStore::new(pool);
    let suffix = unique_suffix();

    let rule = store
        .create(
            &format!("Rule {suffix}"),
            Some("Auto-prepare meetings with clients"),
            json!({"trigger": "event_type=meeting", "action": "generate_brief"}),
            Some("suggest_only"),
        )
        .await
        .expect("create rule");
    assert!(rule.rule_id.starts_with("rule:v1:"));
    assert_eq!(rule.approval_mode, "suggest_only");

    let list = store.list().await.expect("list rules");
    assert!(list.iter().any(|r| r.rule_id == rule.rule_id));

    store.delete(&rule.rule_id).await.expect("delete rule");
    let list2 = store.list().await.expect("list after delete");
    assert!(!list2.iter().any(|r| r.rule_id == rule.rule_id));
}

// ── Intelligence: Classify ─────────────────────────────────────────────────

#[test]
fn intelligence_classify_event() {
    assert_eq!(
        CalendarIntelligenceService::classify_event("Weekly sync meeting", 3, 60),
        "meeting"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Tax deadline AEAT", 1, 0),
        "deadline"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Focus: deep work", 1, 120),
        "focus"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Flight to Madrid", 1, 180),
        "travel"
    ); // long duration -> meeting
    assert_eq!(
        CalendarIntelligenceService::classify_event("Coffee", 1, 15),
        "personal"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Sprint planning", 4, 90),
        "planning"
    );
}

// ── Intelligence: Importance ───────────────────────────────────────────────

#[tokio::test]
async fn intelligence_calculate_importance() {
    let base = CalendarIntelligenceService::calculate_importance("Coffee", 1, false, false);
    assert!(base > 0.0 && base < 1.0);

    let urgent = CalendarIntelligenceService::calculate_importance(
        "URGENT: client escalation",
        3,
        true,
        true,
    );
    assert!(urgent > base);
    assert!(urgent <= 1.0);
}

// ── Intelligence: Readiness ────────────────────────────────────────────────

#[tokio::test]
async fn intelligence_calculate_readiness() {
    let full = CalendarIntelligenceService::calculate_readiness(true, true, true, true, true);
    assert_eq!(full, 1.0);

    let none = CalendarIntelligenceService::calculate_readiness(false, false, false, false, false);
    assert_eq!(none, 0.0);

    let partial = CalendarIntelligenceService::calculate_readiness(true, false, true, false, true);
    assert!(partial > 0.0 && partial < 1.0);
}

// ── Intelligence: Risks ────────────────────────────────────────────────────

#[tokio::test]
async fn intelligence_detect_risks() {
    let none = CalendarIntelligenceService::detect_risks(true, true, true, true, false);
    assert!(none.is_empty());

    let missing = CalendarIntelligenceService::detect_risks(false, false, false, false, true);
    assert_eq!(missing.len(), 5); // all risks triggered
    assert!(missing.contains(&"No agenda prepared".to_string()));
}

// ── Health: Weekly Brief (doesn't require data) ────────────────────────────

#[tokio::test]
async fn health_services_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };

    let brief = CalendarWatchtowerService::weekly_brief(&pool).await;
    assert!(brief.is_ok());

    let prep = CalendarWatchtowerService::events_needing_preparation(&pool).await;
    assert!(prep.is_ok());

    let no_outcomes = CalendarWatchtowerService::events_without_outcomes(&pool).await;
    assert!(no_outcomes.is_ok());

    let load = CalendarWatchtowerService::meeting_load_analysis(&pool).await;
    assert!(load.is_ok());
}

// ── Brain: Search and Weekly Overview ──────────────────────────────────────

#[tokio::test]
async fn brain_services_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };

    let overview = CalendarBrainService::answer(&pool, "show me this week").await;
    assert!(overview.is_ok());

    let search = CalendarBrainService::search_events(&pool, "meeting").await;
    assert!(search.is_ok());
}

// ── Sync: ICS Export (doesn't require database) ────────────────────────────

#[test]
fn sync_ics_export() {
    let ics = hermes_hub_backend::domains::calendar::sync::export_event_ics(
        "Test Meeting",
        Some("Description"),
        Some("Office"),
        "20260101T100000",
        "20260101T110000",
        Some("Europe/Madrid"),
    );
    assert!(ics.contains("BEGIN:VCALENDAR"));
    assert!(ics.contains("SUMMARY:Test Meeting"));
    assert!(ics.contains("DTSTART"));
    assert!(ics.contains("DTEND"));
}

#[test]
fn sync_markdown_export() {
    let md = hermes_hub_backend::domains::calendar::sync::export_event_md(
        "Test Meeting",
        Some("Description"),
        Some("Office"),
        "2026-01-01T10:00:00+01:00",
        "2026-01-01T11:00:00+01:00",
        &["John".into(), "Jane".into()],
    );
    assert!(md.contains("# Test Meeting"));
    assert!(md.contains("John"));
    assert!(md.contains("Jane"));
    assert!(md.contains("Office"));
}

// ── Disconnected pool tests (compile-time verification) ────────────────────

#[tokio::test]
async fn disconnected_pool_creates_store() {
    let pool = disconnected_pool();
    let _store = CalendarEventStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_core_stores() {
    let pool = disconnected_pool();
    let _p = EventParticipantStore::new(pool.clone());
    let _r = EventRelationStore::new(pool.clone());
    let _c = EventContextPackStore::new(pool.clone());
    let _a = EventAgendaStore::new(pool.clone());
    let _cl = EventChecklistStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_scheduling_stores() {
    let pool = disconnected_pool();
    let _d = DeadlineStore::new(pool.clone());
    let _f = FocusBlockStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_rules_store() {
    let _store = CalendarRuleStore::new(disconnected_pool());
}
