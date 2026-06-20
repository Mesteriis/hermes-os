use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct DuplicateGroup {
    pub sha256: String,
    pub filenames: Vec<String>,
    pub message_ids: Vec<String>,
    pub count: i64,
}

#[derive(Clone)]
pub struct AttachmentDedupStore {
    pool: PgPool,
}

impl AttachmentDedupStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_duplicates(
        &self,
        limit: i64,
    ) -> Result<Vec<DuplicateGroup>, AttachmentDedupError> {
        let limit = limit.clamp(1, 50);
        let rows = sqlx::query(
            r#"SELECT sha256, array_agg(DISTINCT filename) AS filenames,
                array_agg(DISTINCT a.message_id) AS message_ids, count(*)::BIGINT AS cnt
            FROM communication_attachments a
            JOIN communication_messages m ON m.message_id = a.message_id
            WHERE m.local_state = 'active'
            GROUP BY sha256 HAVING count(*) > 1
            ORDER BY cnt DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut groups = Vec::new();
        for row in rows {
            let filenames: Vec<String> = row
                .try_get::<Vec<Option<String>>, _>("filenames")?
                .into_iter()
                .flatten()
                .collect();
            let message_ids: Vec<String> = row.try_get("message_ids")?;
            groups.push(DuplicateGroup {
                sha256: row.try_get("sha256")?,
                filenames,
                message_ids,
                count: row.try_get("cnt")?,
            });
        }
        Ok(groups)
    }

    pub async fn find_similar_filenames(
        &self,
        limit: i64,
    ) -> Result<Vec<DuplicateGroup>, AttachmentDedupError> {
        let limit = limit.clamp(1, 50);
        let rows = sqlx::query(
            r#"WITH normalized AS (
                SELECT lower(regexp_replace(regexp_replace(regexp_replace(regexp_replace(filename,
                    '_final', '', 'i'), '_v\d+', '', 'i'), '_copy', '', 'i'), '\s*\(\d+\)', '', 'i')) AS base_name,
                    filename, a.message_id, sha256
                FROM communication_attachments a
                JOIN communication_messages m ON m.message_id = a.message_id
                WHERE filename IS NOT NULL
                  AND m.local_state = 'active'
            )
            SELECT base_name, array_agg(DISTINCT filename) AS filenames,
                array_agg(DISTINCT message_id) AS message_ids, count(*)::BIGINT AS cnt
            FROM normalized
            GROUP BY base_name HAVING count(*) > 1
            ORDER BY cnt DESC LIMIT $1"#,
        ).bind(limit).fetch_all(&self.pool).await?;

        let mut groups = Vec::new();
        for row in rows {
            let base_name: String = row.try_get("base_name")?;
            let filenames: Vec<String> = row
                .try_get::<Vec<Option<String>>, _>("filenames")?
                .into_iter()
                .flatten()
                .collect();
            let message_ids: Vec<String> = row.try_get("message_ids")?;
            groups.push(DuplicateGroup {
                sha256: format!("name_group:{base_name}"),
                filenames,
                message_ids,
                count: row.try_get("cnt")?,
            });
        }
        Ok(groups)
    }
}

#[derive(Debug, Error)]
pub enum AttachmentDedupError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
