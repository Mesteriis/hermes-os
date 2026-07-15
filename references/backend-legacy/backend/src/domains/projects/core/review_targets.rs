use super::super::link_reviews::models::ProjectReviewedTarget;

pub(super) fn reviewed_target_ids(targets: &[ProjectReviewedTarget]) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.target_id.clone())
        .collect()
}
