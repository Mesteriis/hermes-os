use sqlx::{Postgres, Transaction};

use super::errors::ProjectLinkReviewError;
use super::models::{ProjectLinkReviewState, ReviewEventApplication};
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
            }
        }

        Ok(())
    }
}
