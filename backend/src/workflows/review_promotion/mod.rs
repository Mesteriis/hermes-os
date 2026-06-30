use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::decisions::{
    DecisionReviewPort, DecisionReviewState, NewDecision, NewDecisionEvidence,
};
use crate::domains::documents::core::{DocumentImportError, DocumentImportPort, NewDocumentImport};
use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationReviewPort,
    ObligationReviewState,
};
use crate::domains::organizations::api::{OrganizationCommandPort, OrganizationError};
use crate::domains::persons::api::{PersonProjectionError, PersonProjectionPort};
use crate::domains::persons::identity::{
    PersonIdentityPort, PersonIdentityReviewCommand, PersonIdentityReviewState,
};
use crate::domains::projects::core::ProjectCommandPort;
use crate::domains::projects::core::{NewProject, ProjectCommandPortError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewPort, ProjectLinkReviewState, ProjectLinkTargetKind,
};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewPort,
    RelationshipReviewState,
};
use crate::domains::review::{
    ReviewInboxError, ReviewInboxPort, ReviewItem, ReviewItemEvidenceRecord, ReviewItemKind,
    ReviewPromotionTarget,
};
use crate::domains::tasks::api::{NewTask, TaskCommandPort, TaskError};
use crate::domains::tasks::core::{ObligationTaskLinkPort, TaskCoreError};
use crate::platform::observations::{
    NewObservation, Observation, ObservationOriginKind, ObservationPort, ObservationPortError,
    link_domain_entity, materialize_review_transition_link,
};

#[derive(Debug, Error)]
pub enum ReviewPromotionError {
    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),
    #[error(transparent)]
    Task(#[from] TaskError),
    #[error(transparent)]
    TaskCore(#[from] TaskCoreError),
    #[error(transparent)]
    DocumentImport(#[from] DocumentImportError),
    #[error(transparent)]
    Decision(#[from] crate::domains::decisions::DecisionReviewPortError),
    #[error(transparent)]
    Obligation(#[from] crate::domains::obligations::ObligationReviewPortError),
    #[error(transparent)]
    Relationship(#[from] crate::domains::relationships::RelationshipReviewPortError),
    #[error(transparent)]
    PersonIdentity(#[from] crate::domains::persons::identity::PersonIdentityError),
    #[error(transparent)]
    PersonProjection(#[from] PersonProjectionError),
    #[error(transparent)]
    ProjectLinkReview(#[from] crate::domains::projects::link_reviews::ProjectLinkReviewError),
    #[error(transparent)]
    ProjectCommandPort(#[from] ProjectCommandPortError),
    #[error(transparent)]
    Organization(#[from] OrganizationError),
    #[error(transparent)]
    Observation(#[from] ObservationPortError),
    #[error("{0}")]
    InvalidTarget(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct ReviewPromotionService {
    pool: PgPool,
}

impl ReviewPromotionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn promote(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
    ) -> Result<ReviewItem, ReviewPromotionError> {
        self.promote_with_observation(review_item_id, target, None, None, None, None)
            .await
    }

    pub async fn promote_with_observation(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
        observation_id: Option<&str>,
        metadata: Option<Value>,
        causation_id: Option<&str>,
        correlation_id: Option<&str>,
    ) -> Result<ReviewItem, ReviewPromotionError> {
        let review_store = ReviewInboxPort::new(self.pool.clone());
        let item = review_store.get(review_item_id).await?;
        let evidence = review_store.list_evidence(review_item_id).await?;
        let resolved_target = self.materialize_target(&item, &target, &evidence).await?;
        Ok(review_store
            .promote_with_observation(
                review_item_id,
                resolved_target,
                observation_id,
                metadata,
                causation_id,
                correlation_id,
            )
            .await?)
    }

    async fn materialize_target(
        &self,
        item: &ReviewItem,
        target: &ReviewPromotionTarget,
        evidence: &[ReviewItemEvidenceRecord],
    ) -> Result<ReviewPromotionTarget, ReviewPromotionError> {
        match item.item_kind {
            ReviewItemKind::NewPerson => {
                let person_id = self
                    .upsert_person_from_review(item, target, evidence)
                    .await?;
                Ok(ReviewPromotionTarget::new("persons", "persona", person_id))
            }
            ReviewItemKind::NewOrganization => {
                let organization_id = self
                    .upsert_organization_from_review(item, target, evidence)
                    .await?;
                Ok(ReviewPromotionTarget::new(
                    "organizations",
                    "organization",
                    organization_id,
                ))
            }
            ReviewItemKind::IdentityCandidate => {
                let identity_candidate_id =
                    metadata_string(&item.metadata, "identity_candidate_id")
                        .unwrap_or_else(|| item.review_item_id.clone());
                let review_observation = self
                    .capture_review_transition_observation(
                        item,
                        "identity_candidate_review_promotion",
                        json!({
                            "identity_candidate_id": identity_candidate_id,
                            "review_state": "user_confirmed",
                        }),
                        format!("review-item://{}/identity-candidate", item.review_item_id),
                    )
                    .await?;
                let result = PersonIdentityPort::new(self.pool.clone())
                    .set_review_state(&PersonIdentityReviewCommand {
                        command_id: format!("review-promotion:{}", item.review_item_id),
                        identity_candidate_id: identity_candidate_id.clone(),
                        review_state: PersonIdentityReviewState::UserConfirmed,
                        actor_id: "review_promotion".to_owned(),
                    })
                    .await?;
                self.link_review_transition_observation(
                    &review_observation,
                    "persons",
                    "identity_candidate",
                    &result.identity_candidate_id,
                    json!({
                        "review_item_id": item.review_item_id,
                        "review_state": result.review_state.as_str(),
                        "event_id": result.event_id,
                    }),
                )
                .await?;
                Ok(ReviewPromotionTarget::new(
                    "persons",
                    "identity_candidate",
                    result.identity_candidate_id,
                ))
            }
            ReviewItemKind::ProjectLinkCandidate => {
                let project_id = metadata_string(&item.metadata, "project_id")
                    .unwrap_or_else(|| item.review_item_id.clone());
                let target_kind = metadata_string(&item.metadata, "target_kind")
                    .unwrap_or_else(|| "message".to_owned());
                let target_id = metadata_string(&item.metadata, "target_id")
                    .unwrap_or_else(|| item.review_item_id.clone());
                let review_observation = self
                    .capture_review_transition_observation(
                        item,
                        "project_link_review_promotion",
                        json!({
                            "project_id": project_id,
                            "target_kind": target_kind,
                            "target_id": target_id,
                            "review_state": "user_confirmed",
                        }),
                        format!(
                            "review-item://{}/project-link/{}/{}",
                            item.review_item_id, target_kind, target_id
                        ),
                    )
                    .await?;
                let result = ProjectLinkReviewPort::new(self.pool.clone())
                    .set_review_state(&ProjectLinkReviewCommand {
                        command_id: format!("review-promotion:{}", item.review_item_id),
                        project_id: project_id.clone(),
                        target_kind: match target_kind.as_str() {
                            "document" => ProjectLinkTargetKind::Document,
                            _ => ProjectLinkTargetKind::Message,
                        },
                        target_id: target_id.clone(),
                        review_state: ProjectLinkReviewState::UserConfirmed,
                        actor_id: "review_promotion".to_owned(),
                    })
                    .await?;
                self.link_review_transition_observation(
                    &review_observation,
                    "projects",
                    "project_link_review",
                    &result.event_id,
                    json!({
                        "review_item_id": item.review_item_id,
                        "project_id": result.project_id,
                        "target_kind": result.target_kind.as_str(),
                        "target_id": result.target_id,
                        "review_state": result.review_state.as_str(),
                    }),
                )
                .await?;
                Ok(ReviewPromotionTarget::new(
                    "projects",
                    "project_link_candidate",
                    format!(
                        "{}:{}:{}",
                        result.project_id,
                        result.target_kind.as_str(),
                        result.target_id
                    ),
                ))
            }
            ReviewItemKind::ContradictionCandidate => Err(ReviewPromotionError::InvalidTarget(
                "contradiction review items cannot be promoted".to_owned(),
            )),
            ReviewItemKind::PotentialTask => {
                let primary_observation_id =
                    primary_observation_id(evidence).unwrap_or_else(|| item.review_item_id.clone());
                let review_observation = self
                    .capture_review_transition_observation(
                        item,
                        "task_review_promotion",
                        json!({
                            "source_observation_id": primary_observation_id,
                            "target_kind": "task",
                            "review_state": "promoted",
                        }),
                        format!("review-item://{}/task", item.review_item_id),
                    )
                    .await?;
                let task = TaskCommandPort::new(self.pool.clone())
                    .create(&NewTask {
                        title: item.title.clone(),
                        description: Some(item.summary.clone()),
                        provenance_kind: Some("review_item".to_owned()),
                        provenance_id: Some(item.review_item_id.clone()),
                        source_kind: Some("observation".to_owned()),
                        source_id: Some(primary_observation_id),
                        source_type: Some("observation".to_owned()),
                        hermes_status: Some("ready".to_owned()),
                        priority_score: Some(item.confidence),
                        why: Some(item.summary.clone()),
                        tags: Some(json!(["review_promoted"])),
                        ..Default::default()
                    })
                    .await?;
                self.link_review_transition_observation(
                    &review_observation,
                    "tasks",
                    "task",
                    &task.task_id,
                    json!({
                        "review_item_id": item.review_item_id,
                        "review_state": "promoted",
                        "source_observation_id": task.source_id,
                    }),
                )
                .await?;
                for record in evidence {
                    self.link_supporting_review_evidence(
                        &record.observation_id,
                        "tasks",
                        "task",
                        &task.task_id,
                        item.confidence,
                        json!({
                            "review_item_id": item.review_item_id,
                            "promoted_from_review": true,
                            "evidence_role": record.evidence_role,
                        }),
                    )
                    .await?;
                }
                if metadata_string(&item.metadata, "candidate_kind").as_deref()
                    == Some("obligation_task")
                {
                    let obligation = ObligationReviewPort::new(self.pool.clone())
                        .upsert_with_evidence(
                            &NewObligation::new(
                                ObligationEntityKind::Knowledge,
                                item.review_item_id.clone(),
                                item.title.clone(),
                                item.confidence,
                                ObligationReviewState::UserConfirmed,
                            )
                            .metadata(json!({
                                "review_item_id": item.review_item_id,
                                "title": item.title.clone(),
                                "promoted_from_review": true,
                                "candidate_kind": "obligation_task"
                            })),
                            &obligation_evidence(evidence),
                        )
                        .await?;
                    ObligationTaskLinkPort::new(self.pool.clone())
                        .link_fulfillment_task(&obligation.obligation_id, &task.task_id)
                        .await?;
                }
                Ok(ReviewPromotionTarget::new("tasks", "task", task.task_id))
            }
            ReviewItemKind::PotentialDecision => {
                if evidence.is_empty() {
                    return Err(ReviewPromotionError::InvalidTarget(
                        "decision evidence is required".to_owned(),
                    ));
                }
                let decision_store = DecisionReviewPort::new(self.pool.clone());
                let decision = if metadata_string(&item.metadata, "mirrored_from").as_deref()
                    == Some("decisions")
                {
                    let decision_id = metadata_string(&item.metadata, "decision_id")
                        .unwrap_or_else(|| item.review_item_id.clone());
                    let review_observation = self
                        .capture_review_transition_observation(
                            item,
                            "decision_review_promotion",
                            json!({
                                "decision_id": decision_id,
                                "review_state": "user_confirmed",
                            }),
                            format!("review-item://{}/decision", item.review_item_id),
                        )
                        .await?;
                    let decision = decision_store
                        .set_review_state(&decision_id, DecisionReviewState::UserConfirmed)
                        .await?;
                    self.link_review_transition_observation(
                        &review_observation,
                        "decisions",
                        "decision",
                        &decision.decision_id,
                        json!({
                            "review_item_id": item.review_item_id,
                            "review_state": decision.review_state.as_str(),
                        }),
                    )
                    .await?;
                    decision
                } else {
                    let review_observation = self
                        .capture_review_transition_observation(
                            item,
                            "decision_review_promotion",
                            json!({
                                "decision_title": item.title,
                                "review_state": "user_confirmed",
                                "promoted_from_review": true,
                            }),
                            format!("review-item://{}/decision", item.review_item_id),
                        )
                        .await?;
                    let decision = decision_store
                        .upsert_with_evidence(
                            &NewDecision::new(
                                item.title.clone(),
                                item.summary.clone(),
                                item.confidence,
                                DecisionReviewState::UserConfirmed,
                            )
                            .metadata(json!({
                                "review_item_id": item.review_item_id,
                                "promoted_from_review": true
                            })),
                            &decision_evidence(evidence),
                            &[],
                        )
                        .await?;
                    self.link_review_transition_observation(
                        &review_observation,
                        "decisions",
                        "decision",
                        &decision.decision_id,
                        json!({
                            "review_item_id": item.review_item_id,
                            "review_state": decision.review_state.as_str(),
                        }),
                    )
                    .await?;
                    decision
                };
                Ok(ReviewPromotionTarget::new(
                    "decisions",
                    "decision",
                    decision.decision_id,
                ))
            }
            ReviewItemKind::PotentialObligation => {
                if evidence.is_empty() {
                    return Err(ReviewPromotionError::InvalidTarget(
                        "obligation evidence is required".to_owned(),
                    ));
                }
                let obligation_store = ObligationReviewPort::new(self.pool.clone());
                let obligation = if metadata_string(&item.metadata, "mirrored_from").as_deref()
                    == Some("obligations")
                {
                    let obligation_id = metadata_string(&item.metadata, "obligation_id")
                        .unwrap_or_else(|| item.review_item_id.clone());
                    let review_observation = self
                        .capture_review_transition_observation(
                            item,
                            "obligation_review_promotion",
                            json!({
                                "obligation_id": obligation_id,
                                "review_state": "user_confirmed",
                            }),
                            format!("review-item://{}/obligation", item.review_item_id),
                        )
                        .await?;
                    let obligation = obligation_store
                        .set_review_state(&obligation_id, ObligationReviewState::UserConfirmed)
                        .await?;
                    self.link_review_transition_observation(
                        &review_observation,
                        "obligations",
                        "obligation",
                        &obligation.obligation_id,
                        json!({
                            "review_item_id": item.review_item_id,
                            "review_state": obligation.review_state.as_str(),
                        }),
                    )
                    .await?;
                    obligation
                } else {
                    let review_observation = self
                        .capture_review_transition_observation(
                            item,
                            "obligation_review_promotion",
                            json!({
                                "obligation_title": item.title,
                                "review_state": "user_confirmed",
                                "promoted_from_review": true,
                            }),
                            format!("review-item://{}/obligation", item.review_item_id),
                        )
                        .await?;
                    let obligation = obligation_store
                        .upsert_with_evidence(
                            &NewObligation::new(
                                ObligationEntityKind::Knowledge,
                                item.review_item_id.clone(),
                                item.summary.clone(),
                                item.confidence,
                                ObligationReviewState::UserConfirmed,
                            )
                            .metadata(json!({
                                "review_item_id": item.review_item_id,
                                "title": item.title.clone(),
                                "promoted_from_review": true
                            })),
                            &obligation_evidence(evidence),
                        )
                        .await?;
                    self.link_review_transition_observation(
                        &review_observation,
                        "obligations",
                        "obligation",
                        &obligation.obligation_id,
                        json!({
                            "review_item_id": item.review_item_id,
                            "review_state": obligation.review_state.as_str(),
                        }),
                    )
                    .await?;
                    obligation
                };
                Ok(ReviewPromotionTarget::new(
                    "obligations",
                    "obligation",
                    obligation.obligation_id,
                ))
            }
            ReviewItemKind::PotentialRelationship => {
                if evidence.is_empty() {
                    return Err(ReviewPromotionError::InvalidTarget(
                        "relationship evidence is required".to_owned(),
                    ));
                }
                let relationship_store = RelationshipReviewPort::new(self.pool.clone());
                let relationship = if metadata_string(&item.metadata, "mirrored_from").as_deref()
                    == Some("relationships")
                {
                    let relationship_id = metadata_string(&item.metadata, "relationship_id")
                        .unwrap_or_else(|| item.review_item_id.clone());
                    let review_observation = self
                        .capture_review_transition_observation(
                            item,
                            "relationship_review_promotion",
                            json!({
                                "relationship_id": relationship_id,
                                "review_state": "user_confirmed",
                            }),
                            format!("review-item://{}/relationship", item.review_item_id),
                        )
                        .await?;
                    let relationship = relationship_store
                        .set_review_state(&relationship_id, RelationshipReviewState::UserConfirmed)
                        .await?;
                    self.link_review_transition_observation(
                        &review_observation,
                        "relationships",
                        "relationship",
                        &relationship.relationship_id,
                        json!({
                            "review_item_id": item.review_item_id,
                            "review_state": relationship.review_state.as_str(),
                        }),
                    )
                    .await?;
                    relationship
                } else {
                    let primary_observation_id = primary_observation_id(evidence)
                        .unwrap_or_else(|| format!("observation:{}", item.review_item_id));
                    let relationship_type = metadata_string(&item.metadata, "relationship_type")
                        .unwrap_or_else(|| "supports".to_owned());
                    let review_observation = self
                        .capture_review_transition_observation(
                            item,
                            "relationship_review_promotion",
                            json!({
                                "relationship_type": relationship_type,
                                "review_state": "user_confirmed",
                                "promoted_from_review": true,
                            }),
                            format!("review-item://{}/relationship", item.review_item_id),
                        )
                        .await?;
                    let relationship = relationship_store
                        .upsert_with_evidence(
                            &NewRelationship {
                                source_entity_kind: RelationshipEntityKind::Knowledge,
                                source_entity_id: item.review_item_id.clone(),
                                target_entity_kind: RelationshipEntityKind::Communication,
                                target_entity_id: primary_observation_id,
                                relationship_type,
                                trust_score: item.confidence,
                                strength_score: item.confidence,
                                confidence: item.confidence,
                                review_state: RelationshipReviewState::UserConfirmed,
                                valid_from: Some(Utc::now()),
                                valid_to: None,
                                metadata: json!({
                                    "review_item_id": item.review_item_id,
                                    "promoted_from_review": true
                                }),
                            },
                            &relationship_evidence(evidence),
                        )
                        .await?;
                    self.link_review_transition_observation(
                        &review_observation,
                        "relationships",
                        "relationship",
                        &relationship.relationship_id,
                        json!({
                            "review_item_id": item.review_item_id,
                            "review_state": relationship.review_state.as_str(),
                        }),
                    )
                    .await?;
                    relationship
                };
                Ok(ReviewPromotionTarget::new(
                    "relationships",
                    "relationship",
                    relationship.relationship_id,
                ))
            }
            ReviewItemKind::PotentialProject => {
                let project_id = self
                    .upsert_project_from_review(item, target, evidence)
                    .await?;
                Ok(ReviewPromotionTarget::new(
                    "projects", "project", project_id,
                ))
            }
            ReviewItemKind::KnowledgeCandidate => {
                self.promote_knowledge_candidate(item, target, evidence)
                    .await
            }
        }
    }

    async fn upsert_person_from_review(
        &self,
        item: &ReviewItem,
        target: &ReviewPromotionTarget,
        evidence: &[ReviewItemEvidenceRecord],
    ) -> Result<String, ReviewPromotionError> {
        let person_id = choose_target_id(target, "person", &item.review_item_id);
        let review_observation = self
            .capture_review_transition_observation(
                item,
                "person_review_promotion",
                json!({
                    "person_id": person_id,
                    "review_state": "user_confirmed",
                }),
                format!("review-item://{}/person", item.review_item_id),
            )
            .await?;
        PersonProjectionPort::new(self.pool.clone())
            .upsert_review_person(&person_id, item.title.trim())
            .await?;
        self.link_review_transition_observation(
            &review_observation,
            "persons",
            "persona",
            &person_id,
            json!({
                "review_item_id": item.review_item_id,
                "review_state": "user_confirmed",
            }),
        )
        .await?;

        for record in evidence {
            self.link_supporting_review_evidence(
                &record.observation_id,
                "persons",
                "persona",
                &person_id,
                item.confidence,
                json!({
                    "review_item_id": item.review_item_id,
                    "promoted_from_review": true,
                    "evidence_role": record.evidence_role,
                }),
            )
            .await?;
        }
        Ok(person_id)
    }

    async fn upsert_organization_from_review(
        &self,
        item: &ReviewItem,
        target: &ReviewPromotionTarget,
        evidence: &[ReviewItemEvidenceRecord],
    ) -> Result<String, ReviewPromotionError> {
        let organization_id = choose_target_id(target, "org", &item.review_item_id);
        let review_observation = self
            .capture_review_transition_observation(
                item,
                "organization_review_promotion",
                json!({
                    "organization_id": organization_id,
                    "review_state": "user_confirmed",
                }),
                format!("review-item://{}/organization", item.review_item_id),
            )
            .await?;
        OrganizationCommandPort::new(self.pool.clone())
            .upsert_review_organization(
                &organization_id,
                item.title.trim(),
                Some(item.summary.trim()),
            )
            .await?;
        self.link_review_transition_observation(
            &review_observation,
            "organizations",
            "organization",
            &organization_id,
            json!({
                "review_item_id": item.review_item_id,
                "review_state": "user_confirmed",
            }),
        )
        .await?;

        for record in evidence {
            self.link_supporting_review_evidence(
                &record.observation_id,
                "organizations",
                "organization",
                &organization_id,
                item.confidence,
                json!({
                    "review_item_id": item.review_item_id,
                    "promoted_from_review": true,
                    "evidence_role": record.evidence_role,
                }),
            )
            .await?;
        }
        Ok(organization_id)
    }

    async fn upsert_project_from_review(
        &self,
        item: &ReviewItem,
        target: &ReviewPromotionTarget,
        evidence: &[ReviewItemEvidenceRecord],
    ) -> Result<String, ReviewPromotionError> {
        let project_id = choose_target_id(target, "project", &item.review_item_id);
        let review_observation = self
            .capture_review_transition_observation(
                item,
                "project_review_promotion",
                json!({
                    "project_id": project_id,
                    "review_state": "user_confirmed",
                }),
                format!("review-item://{}/project", item.review_item_id),
            )
            .await?;
        ProjectCommandPort::new(self.pool.clone())
            .upsert_project(&NewProject::active(
                &project_id,
                item.title.trim(),
                "review_promoted",
                item.summary.trim(),
                "Hermes Review",
                review_keywords(item),
            ))
            .await?;
        self.link_review_transition_observation(
            &review_observation,
            "projects",
            "project",
            &project_id,
            json!({
                "review_item_id": item.review_item_id,
                "review_state": "user_confirmed",
            }),
        )
        .await?;

        for record in evidence {
            self.link_supporting_review_evidence(
                &record.observation_id,
                "projects",
                "project",
                &project_id,
                item.confidence,
                json!({
                    "review_item_id": item.review_item_id,
                    "promoted_from_review": true,
                    "evidence_role": record.evidence_role,
                }),
            )
            .await?;
        }
        Ok(project_id)
    }

    async fn promote_knowledge_candidate(
        &self,
        item: &ReviewItem,
        target: &ReviewPromotionTarget,
        evidence: &[ReviewItemEvidenceRecord],
    ) -> Result<ReviewPromotionTarget, ReviewPromotionError> {
        if target.target_domain.trim() != "documents"
            || target.target_entity_kind.trim() != "document"
        {
            return Err(ReviewPromotionError::InvalidTarget(
                "knowledge candidates can only be promoted to documents/document".to_owned(),
            ));
        }
        if evidence.is_empty() {
            return Err(ReviewPromotionError::InvalidTarget(
                "knowledge candidate evidence is required".to_owned(),
            ));
        }

        let document_id = choose_target_id(target, "document", &item.review_item_id);
        let review_observation = self
            .capture_review_transition_observation(
                item,
                "knowledge_review_promotion",
                json!({
                    "document_id": document_id,
                    "candidate_group": metadata_string(&item.metadata, "candidate_group")
                        .unwrap_or_else(|| "knowledge".to_owned()),
                    "review_state": "promoted",
                }),
                format!("review-item://{}/knowledge-document", item.review_item_id),
            )
            .await?;
        let markdown = self.knowledge_candidate_markdown(item, evidence).await?;
        let imported = DocumentImportPort::new(self.pool.clone())
            .import_document(&NewDocumentImport::markdown(
                &document_id,
                knowledge_document_title(item),
                markdown,
            ))
            .await?;
        self.link_review_transition_observation(
            &review_observation,
            "documents",
            "document",
            &imported.document_id,
            json!({
                "review_item_id": item.review_item_id,
                "review_state": "promoted",
            }),
        )
        .await?;

        for record in evidence {
            self.link_supporting_review_evidence(
                &record.observation_id,
                "documents",
                "document",
                &imported.document_id,
                item.confidence,
                json!({
                    "review_item_id": item.review_item_id,
                    "knowledge_candidate": true,
                    "evidence_role": record.evidence_role,
                }),
            )
            .await?;
        }

        Ok(ReviewPromotionTarget::new(
            "documents",
            "document",
            imported.document_id,
        ))
    }

    async fn knowledge_candidate_markdown(
        &self,
        item: &ReviewItem,
        evidence: &[ReviewItemEvidenceRecord],
    ) -> Result<String, ReviewPromotionError> {
        let observation_store = ObservationPort::new(self.pool.clone());
        let mut markdown = String::new();
        markdown.push_str("# ");
        markdown.push_str(item.title.trim());
        markdown.push_str("\n\n");
        markdown.push_str(item.summary.trim());
        markdown.push_str("\n\n## Review context\n\n");
        markdown.push_str("- Review item: `");
        markdown.push_str(&item.review_item_id);
        markdown.push_str("`\n");
        markdown.push_str("- Candidate group: `");
        markdown.push_str(
            metadata_string(&item.metadata, "candidate_group")
                .as_deref()
                .unwrap_or("knowledge"),
        );
        markdown.push_str("`\n");
        markdown.push_str("- Confidence: `");
        markdown.push_str(&format!("{:.2}", item.confidence));
        markdown.push_str("`\n");

        markdown.push_str("\n## Evidence\n");
        for record in evidence {
            markdown.push_str("\n- Observation `");
            markdown.push_str(&record.observation_id);
            markdown.push('`');
            if let Some(observation) = observation_store.get(&record.observation_id).await?
                && let Some(excerpt) = observation_excerpt(&observation)
            {
                markdown.push_str(": ");
                markdown.push_str(&excerpt);
            }
        }

        Ok(markdown)
    }

    async fn capture_review_transition_observation(
        &self,
        item: &ReviewItem,
        operation: &str,
        payload: Value,
        source_ref: String,
    ) -> Result<Observation, ReviewPromotionError> {
        Ok(ObservationPort::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "review_item_id": item.review_item_id,
                        "operation": operation,
                        "payload": payload,
                    }),
                    source_ref,
                )
                .provenance(json!({
                    "captured_by": "engines.review_promotion",
                    "operation": operation,
                })),
            )
            .await?)
    }

    async fn link_review_transition_observation(
        &self,
        observation: &Observation,
        domain: &str,
        entity_kind: &str,
        entity_id: &str,
        metadata: Value,
    ) -> Result<(), ReviewPromotionError> {
        materialize_review_transition_link(
            &self.pool,
            Some(&observation.observation_id),
            domain,
            entity_kind,
            entity_id,
            "transition_kind",
            "review_transition",
            Some(metadata),
        )
        .await?;
        Ok(())
    }

    async fn link_supporting_review_evidence(
        &self,
        observation_id: &str,
        domain: &str,
        entity_kind: &str,
        entity_id: &str,
        confidence: f64,
        metadata: Value,
    ) -> Result<(), ReviewPromotionError> {
        link_domain_entity(
            &self.pool,
            observation_id,
            domain,
            entity_kind,
            entity_id.to_owned(),
            Some("supports"),
            Some(confidence),
            Some(metadata),
        )
        .await?;
        Ok(())
    }
}

fn review_keywords(item: &ReviewItem) -> Vec<String> {
    let mut keywords = vec![item.title.trim().to_owned()];
    for word in item.summary.split_whitespace().take(3) {
        let cleaned = word
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect::<String>();
        if !cleaned.is_empty() {
            keywords.push(cleaned);
        }
    }
    keywords.sort();
    keywords.dedup();
    keywords
}

fn choose_target_id(target: &ReviewPromotionTarget, prefix: &str, seed: &str) -> String {
    let candidate = target.target_entity_id.trim();
    if !candidate.is_empty() {
        return candidate.to_owned();
    }
    format!("{prefix}:v1:{}", stable_short_hash(seed))
}

fn stable_short_hash(seed: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(seed.as_bytes());
    format!("{:x}", digest.finalize())[..16].to_owned()
}

fn knowledge_document_title(item: &ReviewItem) -> String {
    let title = item.title.trim();
    if title.ends_with(".md") {
        title.to_owned()
    } else {
        format!("{title}.md")
    }
}

fn primary_observation_id(evidence: &[ReviewItemEvidenceRecord]) -> Option<String> {
    evidence
        .iter()
        .find(|record| record.evidence_role == "primary")
        .or_else(|| evidence.first())
        .map(|record| record.observation_id.clone())
}

fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn observation_excerpt(observation: &Observation) -> Option<String> {
    for key in [
        "quote",
        "evidence",
        "body",
        "transcript",
        "subject",
        "title",
        "extracted_text",
    ] {
        if let Some(value) = observation.payload.get(key).and_then(Value::as_str) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_owned());
            }
        }
    }

    None
}

fn decision_evidence(evidence: &[ReviewItemEvidenceRecord]) -> Vec<NewDecisionEvidence> {
    evidence
        .iter()
        .map(|record| {
            NewDecisionEvidence::observation(record.observation_id.clone()).metadata(json!({
                "evidence_role": record.evidence_role,
                "review_metadata": record.metadata
            }))
        })
        .collect()
}

fn obligation_evidence(evidence: &[ReviewItemEvidenceRecord]) -> Vec<NewObligationEvidence> {
    evidence
        .iter()
        .map(|record| {
            NewObligationEvidence::observation(record.observation_id.clone()).metadata(json!({
                "evidence_role": record.evidence_role,
                "review_metadata": record.metadata
            }))
        })
        .collect()
}

fn relationship_evidence(evidence: &[ReviewItemEvidenceRecord]) -> Vec<NewRelationshipEvidence> {
    evidence
        .iter()
        .map(|record| {
            NewRelationshipEvidence::observation(record.observation_id.clone()).metadata(json!({
                "evidence_role": record.evidence_role,
                "review_metadata": record.metadata
            }))
        })
        .collect()
}
