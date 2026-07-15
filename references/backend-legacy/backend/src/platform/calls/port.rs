use sqlx::postgres::PgPool;

use super::errors::CallError;
use super::models::{NewTelegramCall, TelegramCall};
use super::store::CallIntelligenceStore;

/// Semantic application port for call-intelligence persistence.
#[derive(Clone)]
pub struct CallIntelligencePort(CallIntelligenceStore);

impl CallIntelligencePort {
    pub fn new(pool: PgPool) -> Self {
        Self(CallIntelligenceStore::new(pool))
    }

    pub async fn upsert_call(&self, call: &NewTelegramCall) -> Result<TelegramCall, CallError> {
        self.0.upsert_call(call).await
    }
}
