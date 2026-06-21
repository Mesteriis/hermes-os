use super::super::errors::PersonIdentityError;
use super::super::validation::validate_limit;
use super::PersonIdentityStore;
use super::name_merge_candidates::refresh_name_merge_candidates;
use super::split_candidates::refresh_split_candidates;

impl PersonIdentityStore {
    pub async fn refresh_candidates(&self, limit: i64) -> Result<usize, PersonIdentityError> {
        let limit = validate_limit(limit)?;
        let merge_count = refresh_name_merge_candidates(self.pool(), limit).await?;
        let split_count = refresh_split_candidates(self.pool(), limit).await?;

        Ok(merge_count + split_count)
    }
}
