use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct MailboxHealth {
    pub total_messages: i64,
    pub unread: i64,
    pub needs_action: i64,
    pub waiting: i64,
    pub done: i64,
    pub archived: i64,
    pub spam: i64,
    pub important: i64,
    pub with_attachments: i64,
    pub average_importance: f64,
    pub oldest_message_days: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SenderStats {
    pub sender: String,
    pub message_count: i64,
    pub avg_importance: f64,
    pub last_message_days: Option<f64>,
}

#[derive(Clone)]
pub struct EmailAnalyticsStore {
    pool: PgPool,
}

impl EmailAnalyticsStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mailbox_health(
        &self,
        account_id: Option<&str>,
    ) -> Result<MailboxHealth, EmailAnalyticsError> {
        let row = sqlx::query(
            r#"SELECT
                count(*)::BIGINT AS total_messages,
                count(*) FILTER (WHERE workflow_state = 'new')::BIGINT AS unread,
                count(*) FILTER (WHERE workflow_state = 'needs_action')::BIGINT AS needs_action,
                count(*) FILTER (WHERE workflow_state = 'waiting')::BIGINT AS waiting,
                count(*) FILTER (WHERE workflow_state = 'done')::BIGINT AS done,
                count(*) FILTER (WHERE workflow_state = 'archived')::BIGINT AS archived,
                count(*) FILTER (WHERE workflow_state = 'spam')::BIGINT AS spam,
                count(*) FILTER (WHERE importance_score >= 75)::BIGINT AS important,
                count(*) FILTER (WHERE EXISTS(SELECT 1 FROM communication_attachments a WHERE a.message_id = communication_messages.message_id))::BIGINT AS with_attachments,
                COALESCE(avg(importance_score), 0)::DOUBLE PRECISION AS average_importance,
                EXTRACT(EPOCH FROM now() - min(occurred_at))::DOUBLE PRECISION / 86400.0::DOUBLE PRECISION AS oldest_message_days
            FROM communication_messages
            WHERE ($1::text IS NULL OR account_id = $1)
              AND channel_kind = 'email'
              AND local_state = 'active'"#,
        ).bind(account_id).fetch_one(&self.pool).await?;

        Ok(MailboxHealth {
            total_messages: row.try_get("total_messages")?,
            unread: row.try_get("unread")?,
            needs_action: row.try_get("needs_action")?,
            waiting: row.try_get("waiting")?,
            done: row.try_get("done")?,
            archived: row.try_get("archived")?,
            spam: row.try_get("spam")?,
            important: row.try_get("important")?,
            with_attachments: row.try_get("with_attachments")?,
            average_importance: row.try_get("average_importance")?,
            oldest_message_days: row.try_get("oldest_message_days")?,
        })
    }

    pub async fn top_senders(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SenderStats>, EmailAnalyticsError> {
        let limit = limit.clamp(1, 50);
        let rows = sqlx::query(
            r#"SELECT sender, count(*)::BIGINT AS message_count,
                COALESCE(avg(importance_score), 0)::DOUBLE PRECISION AS avg_importance,
                EXTRACT(EPOCH FROM now() - max(occurred_at))::DOUBLE PRECISION / 86400.0::DOUBLE PRECISION AS last_message_days
            FROM communication_messages
            WHERE ($1::text IS NULL OR account_id = $1)
              AND channel_kind = 'email'
              AND local_state = 'active'
            GROUP BY sender ORDER BY message_count DESC LIMIT $2"#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(SenderStats {
                sender: row.try_get("sender")?,
                message_count: row.try_get("message_count")?,
                avg_importance: row.try_get("avg_importance")?,
                last_message_days: row.try_get("last_message_days")?,
            });
        }
        Ok(stats)
    }
}

#[derive(Debug, Error)]
pub enum EmailAnalyticsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
