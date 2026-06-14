use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn project_link_review_set(
        actor_id: impl Into<String>,
        project_id: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        let project_id = project_id.into();
        let target_kind = target_kind.into();
        let target_id = target_id.into();

        Self::new(
            actor_id,
            "project.link_review.set",
            "PUT",
            "/api/v1/projects/{project_id}/link-reviews",
            "project_link",
            Some(format!("{project_id}:{target_kind}:{target_id}")),
            json!({
                "project_id": project_id,
                "target_kind": target_kind,
                "target_id": target_id,
            }),
        )
    }

    pub fn task_candidate_review_set(
        actor_id: impl Into<String>,
        task_candidate_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "task_candidate.review.set",
            "PUT",
            "/api/v1/task-candidates/{task_candidate_id}/review",
            "task_candidate",
            Some(task_candidate_id.into()),
            json!({}),
        )
    }

    pub fn obligation_review_set(
        actor_id: impl Into<String>,
        obligation_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "obligation.review.set",
            "PUT",
            "/api/v1/obligations/{obligation_id}/review",
            "obligation",
            Some(obligation_id.into()),
            json!({}),
        )
    }

    pub fn decision_review_set(
        actor_id: impl Into<String>,
        decision_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "decision.review.set",
            "PUT",
            "/api/v1/decisions/{decision_id}/review",
            "decision",
            Some(decision_id.into()),
            json!({}),
        )
    }

    pub fn relationship_review_set(
        actor_id: impl Into<String>,
        relationship_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "relationship.review.set",
            "PUT",
            "/api/v1/relationships/{relationship_id}/review",
            "relationship",
            Some(relationship_id.into()),
            json!({}),
        )
    }

    pub fn contradiction_review_set(
        actor_id: impl Into<String>,
        observation_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "contradiction.review.set",
            "PUT",
            "/api/v1/contradictions/{observation_id}/review",
            "contradiction_observation",
            Some(observation_id.into()),
            json!({}),
        )
    }

    pub fn message_workflow_state_set(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "message.workflow_state.set",
            "PUT",
            "/api/v1/communications/messages/{message_id}/workflow-state",
            "communication_message",
            Some(message_id.into()),
            json!({}),
        )
    }

    pub fn person_identity_review_set(
        actor_id: impl Into<String>,
        identity_candidate_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "person_identity.review.set",
            "PUT",
            "/api/v1/identity-candidates/{identity_candidate_id}/review",
            "person_identity_candidate",
            Some(identity_candidate_id.into()),
            json!({}),
        )
    }
}
