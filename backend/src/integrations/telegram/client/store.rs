use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct TelegramStore {
    pub(super) pool: PgPool,
}

impl TelegramStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
