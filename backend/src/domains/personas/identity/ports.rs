use std::ops::Deref;

use sqlx::postgres::PgPool;

use super::store::PersonaIdentityReviewStore;

/// Review/application port for persona identity candidates.
#[derive(Clone)]
pub struct PersonaIdentityReviewPort(PersonaIdentityReviewStore);

impl PersonaIdentityReviewPort {
    pub fn new(pool: PgPool) -> Self {
        Self(PersonaIdentityReviewStore::new(pool))
    }
}

impl Deref for PersonaIdentityReviewPort {
    type Target = PersonaIdentityReviewStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
