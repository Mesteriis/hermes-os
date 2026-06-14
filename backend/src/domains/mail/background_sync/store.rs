use sqlx::postgres::PgPool;

mod account;
mod orphaned;
mod run_finish;
mod run_latest;
mod run_progress;
mod run_start;
mod scheduling;
mod settings;
mod statuses;

#[derive(Clone)]
pub struct MailSyncStore {
    pub(in crate::domains::mail::background_sync::store) pool: PgPool,
}

impl MailSyncStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
