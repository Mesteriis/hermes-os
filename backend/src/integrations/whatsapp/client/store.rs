mod accounts;
mod evidence;
mod ingestion;
mod intelligence;
mod queries;
mod sessions;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) pool: PgPool,
}

impl WhatsappWebStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
