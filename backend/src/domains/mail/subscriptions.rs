use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct SubscriptionSource {
    pub sender: String,
    pub message_count: i64,
    pub first_seen: String,
    pub last_seen: String,
    pub is_newsletter: bool,
    pub has_unsubscribe: bool,
}

#[derive(Clone)]
pub struct SubscriptionStore {
    pool: PgPool,
}

impl SubscriptionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn detect_subscriptions(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SubscriptionSource>, SubscriptionError> {
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"SELECT sender, count(*)::BIGINT AS message_count,
                min(occurred_at)::TEXT AS first_seen, max(occurred_at)::TEXT AS last_seen,
                bool_or(lower(body_text) LIKE '%unsubscribe%' OR lower(body_text) LIKE '%opt out%' OR lower(body_text) LIKE '%manage preferences%') AS has_unsubscribe,
                bool_or(lower(subject) LIKE '%newsletter%' OR lower(subject) LIKE '%digest%' OR lower(body_text) LIKE '%newsletter%') AS is_newsletter
            FROM communication_messages
            WHERE ($1::text IS NULL OR account_id = $1)
              AND channel_kind = 'email'
              AND local_state = 'active'
            GROUP BY sender HAVING count(*) > 1
            ORDER BY message_count DESC LIMIT $2"#,
        ).bind(account_id).bind(limit).fetch_all(&self.pool).await?;

        let mut subs = Vec::new();
        for row in rows {
            subs.push(SubscriptionSource {
                sender: row.try_get("sender")?,
                message_count: row.try_get("message_count")?,
                first_seen: row.try_get("first_seen")?,
                last_seen: row.try_get("last_seen")?,
                is_newsletter: row.try_get("is_newsletter")?,
                has_unsubscribe: row.try_get("has_unsubscribe")?,
            });
        }
        Ok(subs)
    }
}

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
