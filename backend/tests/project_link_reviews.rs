use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::event_log::{EventStore, NewEventEnvelope};
use hermes_hub_backend::messages::{MessageProjectionStore, project_raw_email_message};
use hermes_hub_backend::project_link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewCommandResult, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use hermes_hub_backend::projects::{NewProject, ProjectStore};
use hermes_hub_backend::storage::Database;

const PROJECT_LINK_REVIEW_EVENT_TYPE: &str = "project.link_review_state_changed";

#[tokio::test]
async fn project_link_review_command_appends_event_and_updates_review_against_postgres() {
    let Some(context) = live_review_context("project link review command").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectLinkReview{suffix}");
    let project_id = format!("project:v1:review:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Link Review {suffix}"),
                "Product Development",
                "Project link review event test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(63),
        )
        .await
        .expect("upsert review project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-{suffix}"),
        &format!("{keyword} kickoff"),
        "Review review candidate",
    )
    .await;
    let command_id = format!("link-review-confirm-{suffix}");
    let command = ProjectLinkReviewCommand {
        command_id: command_id.clone(),
        project_id: project_id.clone(),
        target_kind: ProjectLinkTargetKind::Message,
        target_id: message_id.clone(),
        review_state: ProjectLinkReviewState::UserConfirmed,
        actor_id: "reviewer".to_owned(),
    };
    let result = context
        .review_store
        .set_review_state(&command)
        .await
        .expect("set review state");

    assert_eq!(
        result,
        ProjectLinkReviewCommandResult {
            project_id,
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id,
            review_state: ProjectLinkReviewState::UserConfirmed,
            event_id: format!("project_link_review:{command_id}"),
        }
    );

    let review = context
        .review_store
        .explicit_review(&result.project_id, result.target_kind, &result.target_id)
        .await
        .expect("load review row")
        .expect("review exists");
    assert_eq!(review.review_state, ProjectLinkReviewState::UserConfirmed);
    assert_eq!(review.event_id, result.event_id);
}

#[tokio::test]
async fn project_link_review_reset_removes_explicit_decision_against_postgres() {
    let Some(context) = live_review_context("project link review reset").await else {
        return;
    };
    let suffix = unique_suffix();
    let project_id = format!("project:v1:review-reset:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Review Reset {suffix}"),
                "Product Development",
                "Reset path test",
                "Alex Morgan",
                vec![format!("Reset{suffix}")],
            )
            .progress(63),
        )
        .await
        .expect("upsert review project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("owner-{suffix}@example.com"),
        &[format!("reviewer-{suffix}@example.com")],
        &format!("provider-link-review-reset-{suffix}"),
        &format!("Reset keyword {suffix}"),
        "Review reset body",
    )
    .await;

    context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("link-review-reject-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::UserRejected,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("reject link first");

    let result = context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("link-review-suggested-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::Suggested,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("reset link");
    assert_eq!(result.review_state, ProjectLinkReviewState::Suggested);

    let review = context
        .review_store
        .explicit_review(&project_id, ProjectLinkTargetKind::Message, &message_id)
        .await
        .expect("load review after reset");
    assert_eq!(review, None);
}

#[tokio::test]
async fn project_link_review_projection_rebuilds_review_state_from_event_against_postgres() {
    let Some(context) = live_review_context("project link review projection replay").await else {
        return;
    };
    let suffix = unique_suffix();
    let project_id = format!("project:v1:review-replay:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Review Replay {suffix}"),
                "Product Development",
                "Replay review projection test",
                "Alex Morgan",
                vec![format!("Replay{suffix}")],
            )
            .progress(63),
        )
        .await
        .expect("upsert review project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("replay-owner-{suffix}@example.com"),
        &[format!("replay-reviewer-{suffix}@example.com")],
        &format!("provider-link-review-replay-{suffix}"),
        &format!("Replay keyword {suffix}"),
        "Replay body",
    )
    .await;
    let actor_id = "projection-reviewer";
    let confirm_event_id = format!("evt_link_review_replay_confirm_{suffix}");
    let confirm = build_review_event(
        &project_id,
        ProjectLinkTargetKind::Message,
        &message_id,
        ProjectLinkReviewState::UserConfirmed,
        actor_id,
        &confirm_event_id,
    );
    let reject_event_id = format!("evt_link_review_replay_reject_{suffix}");
    let reject = build_review_event(
        &project_id,
        ProjectLinkTargetKind::Message,
        &message_id,
        ProjectLinkReviewState::UserRejected,
        actor_id,
        &reject_event_id,
    );
    let _ = context
        .event_store
        .append(&confirm)
        .await
        .expect("append confirm event");
    let _ = context
        .event_store
        .append(&reject)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    let reject_event = context
        .event_store
        .get_by_id(&reject_event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    context
        .review_store
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm");
    context
        .review_store
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject");

    let review = context
        .review_store
        .explicit_review(&project_id, ProjectLinkTargetKind::Message, &message_id)
        .await
        .expect("load replayed review")
        .expect("replayed review exists");
    assert_eq!(review.review_state, ProjectLinkReviewState::UserRejected);
    assert_eq!(review.event_id, reject_event_id);
}

struct LiveReviewContext {
    project_store: ProjectStore,
    communication_store: CommunicationIngestionStore,
    message_store: MessageProjectionStore,
    review_store: ProjectLinkReviewStore,
    event_store: EventStore,
}

async fn live_review_context(_test_name: &str) -> Option<LiveReviewContext> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live project link review test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let _ = database_url;

    Some(LiveReviewContext {
        project_store: ProjectStore::new(pool.clone()),
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        review_store: ProjectLinkReviewStore::new(pool.clone()),
        event_store: EventStore::new(pool),
    })
}

async fn seed_message(
    context: &LiveReviewContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_link_review_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Link Review Gmail",
            format!("link-review-{suffix}@example.com"),
        ))
        .await
        .expect("store link review provider account");

    let raw_record_id = format!("raw_link_review_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:link-review:{suffix}:{provider_record_id}"),
                format!("batch-link-review-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"project_link_review_test"})),
        )
        .await
        .expect("record link review raw message");

    project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project link review message")
        .message_id
}

fn build_review_event(
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    review_state: ProjectLinkReviewState,
    actor_id: &str,
    event_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        event_id,
        PROJECT_LINK_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "project_link_review",
            "provider": "local_api",
            "source_id": event_id,
        }),
        json!({
            "kind": "project_link_review",
            "project_id": project_id,
        }),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "project_id": project_id,
        "target_kind": target_kind.as_str(),
        "target_id": target_id,
        "review_state": review_state.as_str(),
    }))
    .build()
    .expect("valid review event")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
