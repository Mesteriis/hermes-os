use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};

use crate::domains::decisions::{
    DecisionEntityKind, DecisionReviewState, DecisionStore, NewDecision, NewDecisionEvidence,
    NewDecisionImpactedEntity,
};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewState,
    RelationshipStore,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};
use crate::workflows::review_mirror::sync_relationship_review_state_in_transaction;

use super::errors::ProjectLinkReviewError;
use super::models::{ProjectLinkReviewState, ProjectLinkTargetKind, ReviewEventApplication};
use super::store::ProjectLinkReviewStore;

impl ProjectLinkReviewStore {
    pub(crate) async fn apply_review_event_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        application: ReviewEventApplication<'_>,
    ) -> Result<(), ProjectLinkReviewError> {
        match application.review_state {
            ProjectLinkReviewState::Suggested => {
                sqlx::query(
                    r#"
                    DELETE FROM project_link_reviews
                    WHERE project_id = $1
                      AND target_kind = $2
                      AND target_id = $3
                    "#,
                )
                .bind(application.project_id)
                .bind(application.target_kind.as_str())
                .bind(application.target_id)
                .execute(&mut **transaction)
                .await?;

                Self::project_review_relationship_in_transaction(transaction, &application).await?;
            }
            ProjectLinkReviewState::UserConfirmed | ProjectLinkReviewState::UserRejected => {
                sqlx::query(
                    r#"
                    INSERT INTO project_link_reviews (
                        project_id,
                        target_kind,
                        target_id,
                        review_state,
                        event_id,
                        actor_id,
                        reviewed_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (project_id, target_kind, target_id)
                    DO UPDATE SET
                        review_state = EXCLUDED.review_state,
                        event_id = EXCLUDED.event_id,
                        actor_id = EXCLUDED.actor_id,
                        reviewed_at = EXCLUDED.reviewed_at,
                        updated_at = now()
                    "#,
                )
                .bind(application.project_id)
                .bind(application.target_kind.as_str())
                .bind(application.target_id)
                .bind(application.review_state.as_str())
                .bind(application.event_id)
                .bind(application.actor_id)
                .bind(application.reviewed_at)
                .execute(&mut **transaction)
                .await?;

                Self::project_review_decision_in_transaction(transaction, &application).await?;
                Self::project_review_relationship_in_transaction(transaction, &application).await?;
            }
        }

        Ok(())
    }

    async fn project_review_decision_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        application: &ReviewEventApplication<'_>,
    ) -> Result<(), ProjectLinkReviewError> {
        let target_label = application.target_kind.as_str();
        let review_action = match application.review_state {
            ProjectLinkReviewState::UserConfirmed => "confirmed",
            ProjectLinkReviewState::UserRejected => "rejected",
            ProjectLinkReviewState::Suggested => return Ok(()),
        };
        let review_action_title = match application.review_state {
            ProjectLinkReviewState::UserConfirmed => "Confirm",
            ProjectLinkReviewState::UserRejected => "Reject",
            ProjectLinkReviewState::Suggested => return Ok(()),
        };
        let title = format!(
            "{review_action_title} {target_label} link for project {}",
            application.project_id
        );
        let rationale =
            format!("User {review_action} a {target_label} link candidate for this project.");
        let metadata = project_link_review_decision_metadata(application);
        let quote = format!("User {review_action} {target_label} link to project.");
        let observation = capture_project_link_review_observation(
            transaction,
            application,
            "decision",
            "project_link_review",
            &quote,
            &metadata,
        )
        .await?;
        let decision = NewDecision::new(
            title,
            rationale.clone(),
            1.0,
            DecisionReviewState::UserConfirmed,
        )
        .decided_at(application.reviewed_at)
        .metadata(metadata.clone());
        let evidence = NewDecisionEvidence::observation(observation.observation_id)
            .quote(quote)
            .confidence(1.0)
            .metadata(metadata.clone());
        let target_entity_kind = match application.target_kind {
            ProjectLinkTargetKind::Message => DecisionEntityKind::Communication,
            ProjectLinkTargetKind::Document => DecisionEntityKind::Document,
        };
        let impacted_entities = [
            NewDecisionImpactedEntity::new(DecisionEntityKind::Project, application.project_id)
                .impact_type("project_link_review")
                .metadata(metadata.clone()),
            NewDecisionImpactedEntity::new(target_entity_kind, application.target_id)
                .impact_type("project_link_review_target")
                .metadata(metadata),
        ];

        DecisionStore::upsert_with_evidence_in_transaction(
            transaction,
            &decision,
            &[evidence],
            &impacted_entities,
        )
        .await?;

        Ok(())
    }

    async fn project_review_relationship_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        application: &ReviewEventApplication<'_>,
    ) -> Result<(), ProjectLinkReviewError> {
        let target_label = application.target_kind.as_str();
        let review_action = match application.review_state {
            ProjectLinkReviewState::UserConfirmed => "confirmed",
            ProjectLinkReviewState::UserRejected => "rejected",
            ProjectLinkReviewState::Suggested => "reset",
        };
        let (target_entity_kind, relationship_type) = match application.target_kind {
            ProjectLinkTargetKind::Message => {
                (RelationshipEntityKind::Communication, "project_has_message")
            }
            ProjectLinkTargetKind::Document => {
                (RelationshipEntityKind::Document, "project_has_document")
            }
        };
        let review_state = match application.review_state {
            ProjectLinkReviewState::UserConfirmed => RelationshipReviewState::UserConfirmed,
            ProjectLinkReviewState::UserRejected => RelationshipReviewState::UserRejected,
            ProjectLinkReviewState::Suggested => RelationshipReviewState::Suggested,
        };
        let metadata = project_link_review_relationship_metadata(application);
        let relationship = NewRelationship {
            source_entity_kind: RelationshipEntityKind::Project,
            source_entity_id: application.project_id.to_owned(),
            target_entity_kind,
            target_entity_id: application.target_id.to_owned(),
            relationship_type: relationship_type.to_owned(),
            trust_score: 0.5,
            strength_score: match application.review_state {
                ProjectLinkReviewState::UserConfirmed => 0.8,
                ProjectLinkReviewState::UserRejected => 0.2,
                ProjectLinkReviewState::Suggested => 0.5,
            },
            confidence: 1.0,
            review_state,
            valid_from: None,
            valid_to: None,
            metadata: metadata.clone(),
        };
        let evidence_excerpt = match application.review_state {
            ProjectLinkReviewState::Suggested => {
                format!("User reset {target_label} link review for project.")
            }
            ProjectLinkReviewState::UserConfirmed | ProjectLinkReviewState::UserRejected => {
                format!("User {review_action} {target_label} link to project.")
            }
        };
        let observation = capture_project_link_review_observation(
            transaction,
            application,
            "relationship",
            "project_link_review",
            &evidence_excerpt,
            &metadata,
        )
        .await?;
        let observation_id = observation.observation_id.clone();
        let evidence = NewRelationshipEvidence::observation(observation_id.clone())
            .excerpt(&evidence_excerpt)
            .metadata(metadata);

        let stored = RelationshipStore::upsert_with_evidence_in_transaction(
            transaction,
            &relationship,
            &[evidence],
        )
        .await?;
        sync_relationship_review_state_in_transaction(transaction, &stored).await?;

        Ok(())
    }
}

fn project_link_review_decision_metadata(application: &ReviewEventApplication<'_>) -> Value {
    json!({
        "source": "project_link_review_adapter",
        "project_link_review_event_id": application.event_id,
        "project_id": application.project_id,
        "target_kind": application.target_kind.as_str(),
        "target_id": application.target_id,
        "review_state": application.review_state.as_str(),
        "actor_id": application.actor_id,
    })
}

fn project_link_review_relationship_metadata(application: &ReviewEventApplication<'_>) -> Value {
    json!({
        "compatibility_table": "project_link_reviews",
        "project_link_review_event_id": application.event_id,
        "project_id": application.project_id,
        "target_kind": application.target_kind.as_str(),
        "target_id": application.target_id,
        "review_state": application.review_state.as_str(),
        "actor_id": application.actor_id,
        "reviewed_at": application.reviewed_at.to_rfc3339(),
    })
}

fn link_review_source_ref(application: &ReviewEventApplication<'_>, component: &str) -> String {
    format!(
        "project-link-review://{}/{}?event={}",
        component, application.target_id, application.event_id
    )
}

async fn capture_project_link_review_observation(
    transaction: &mut Transaction<'_, Postgres>,
    application: &ReviewEventApplication<'_>,
    component: &str,
    domain: &str,
    evidence: &str,
    metadata: &Value,
) -> Result<crate::platform::observations::Observation, ProjectLinkReviewError> {
    let observation = NewObservation::new(
        "PROJECT_LINK_REVIEW",
        ObservationOriginKind::LocalRuntime,
        application.reviewed_at,
        json!({
            "component": component,
            "domain": domain,
            "evidence": evidence,
            "event_id": application.event_id,
            "project_id": application.project_id,
            "target_kind": application.target_kind.as_str(),
            "target_id": application.target_id,
            "review_state": application.review_state.as_str(),
            "actor_id": application.actor_id,
            "metadata": metadata,
        }),
        link_review_source_ref(application, component),
    )
    .confidence(1.0)
    .provenance(json!({
        "domain": "project_link_review",
        "component": component,
        "project_id": application.project_id,
        "event_id": application.event_id,
    }));

    ObservationStore::capture_in_transaction(transaction, &observation)
        .await
        .map_err(ProjectLinkReviewError::from)
}
