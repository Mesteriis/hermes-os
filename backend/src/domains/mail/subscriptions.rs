use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
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

#[derive(Clone, Debug, Serialize)]
pub struct SubscriptionSourceListPage {
    pub items: Vec<SubscriptionSource>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
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
        Ok(self
            .detect_subscriptions_page(account_id, limit, None)
            .await?
            .items)
    }

    pub async fn detect_subscriptions_page(
        &self,
        account_id: Option<&str>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<SubscriptionSourceListPage, SubscriptionError> {
        let limit = limit.clamp(1, 100);
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_subscription_cursor)
            .transpose()?;
        let rows = sqlx::query(
            r#"WITH subscription_sources AS (
                SELECT sender, count(*)::BIGINT AS message_count,
                    min(occurred_at)::TEXT AS first_seen, max(occurred_at)::TEXT AS last_seen,
                    bool_or(lower(body_text) LIKE '%unsubscribe%' OR lower(body_text) LIKE '%opt out%' OR lower(body_text) LIKE '%manage preferences%') AS has_unsubscribe,
                    bool_or(lower(subject) LIKE '%newsletter%' OR lower(subject) LIKE '%digest%' OR lower(body_text) LIKE '%newsletter%') AS is_newsletter
                FROM communication_messages
                WHERE ($1::text IS NULL OR account_id = $1)
                  AND channel_kind = 'email'
                  AND local_state = 'active'
                GROUP BY sender
                HAVING count(*) > 1
            )
            SELECT sender, message_count, first_seen, last_seen, has_unsubscribe, is_newsletter
            FROM subscription_sources
            WHERE (
                $2::BIGINT IS NULL
                OR message_count < $2
                OR (message_count = $2 AND sender > $3)
            )
            ORDER BY message_count DESC, sender ASC
            LIMIT $4"#,
        )
        .bind(account_id)
        .bind(cursor.as_ref().map(|value| value.message_count))
        .bind(cursor.as_ref().map(|value| value.sender.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;

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
        let has_more = subs.len() > limit as usize;
        if has_more {
            subs.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            subs.last().map(encode_subscription_cursor).transpose()?
        } else {
            None
        };
        Ok(SubscriptionSourceListPage {
            items: subs,
            next_cursor,
            has_more,
        })
    }
}

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid subscription cursor")]
    InvalidCursor,
}

#[derive(Debug, Deserialize, Serialize)]
struct SubscriptionCursor {
    message_count: i64,
    sender: String,
}

fn encode_subscription_cursor(source: &SubscriptionSource) -> Result<String, SubscriptionError> {
    let cursor = SubscriptionCursor {
        message_count: source.message_count,
        sender: source.sender.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| SubscriptionError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_subscription_cursor(cursor: &str) -> Result<SubscriptionCursor, SubscriptionError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| SubscriptionError::InvalidCursor)?;
    let cursor: SubscriptionCursor =
        serde_json::from_slice(&bytes).map_err(|_| SubscriptionError::InvalidCursor)?;
    if cursor.message_count < 0 || cursor.sender.trim().is_empty() {
        return Err(SubscriptionError::InvalidCursor);
    }
    Ok(cursor)
}
