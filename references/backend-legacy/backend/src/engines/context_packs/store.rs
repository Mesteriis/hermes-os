use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Transaction};

use super::errors::ContextPackStoreError;
use super::models::{
    ContextPack, ContextPackKind, ContextPackSource, ContextPackSourceKind, NewContextPack,
    NewContextPackSource, validate_context_pack_with_sources, validate_non_empty,
};

#[derive(Clone)]
pub struct ContextPackStore {
    pool: PgPool,
}

impl ContextPackStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        kind: ContextPackKind,
        subject_id: &str,
    ) -> Result<Option<ContextPack>, ContextPackStoreError> {
        validate_non_empty("subject_id", subject_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                context_pack_id,
                kind,
                subject_id,
                content,
                metadata,
                rebuildable,
                built_at,
                updated_at
            FROM context_packs
            WHERE kind = $1 AND subject_id = $2
            LIMIT 1
            "#,
        )
        .bind(kind.as_str())
        .bind(subject_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_context_pack).transpose()
    }

    pub async fn exists(
        &self,
        kind: ContextPackKind,
        subject_id: &str,
    ) -> Result<bool, ContextPackStoreError> {
        validate_non_empty("subject_id", subject_id)?;
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM context_packs
                WHERE kind = $1 AND subject_id = $2
            )
            "#,
        )
        .bind(kind.as_str())
        .bind(subject_id.trim())
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    pub async fn upsert_with_sources(
        &self,
        pack: &NewContextPack,
        sources: &[NewContextPackSource],
    ) -> Result<ContextPack, ContextPackStoreError> {
        validate_context_pack_with_sources(pack, sources)?;

        let mut transaction = self.pool.begin().await?;
        let stored = self
            .upsert_with_sources_in_transaction(&mut transaction, pack, sources)
            .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_sources(
        &self,
        context_pack_id: &str,
    ) -> Result<Vec<ContextPackSource>, ContextPackStoreError> {
        validate_non_empty("context_pack_id", context_pack_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                context_pack_id,
                source_kind,
                source_id,
                role,
                metadata,
                created_at
            FROM context_pack_sources
            WHERE context_pack_id = $1
            ORDER BY role ASC, source_kind ASC, source_id ASC
            "#,
        )
        .bind(context_pack_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_context_pack_source).collect()
    }

    async fn upsert_with_sources_in_transaction(
        &self,
        transaction: &mut Transaction<'_, sqlx::Postgres>,
        pack: &NewContextPack,
        sources: &[NewContextPackSource],
    ) -> Result<ContextPack, ContextPackStoreError> {
        let context_pack_id = context_pack_id(pack)?;
        let row = sqlx::query(
            r#"
            INSERT INTO context_packs (
                context_pack_id,
                kind,
                subject_id,
                content,
                metadata,
                rebuildable
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (kind, subject_id)
            DO UPDATE SET
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                rebuildable = EXCLUDED.rebuildable,
                built_at = now(),
                updated_at = now()
            RETURNING
                context_pack_id,
                kind,
                subject_id,
                content,
                metadata,
                rebuildable,
                built_at,
                updated_at
            "#,
        )
        .bind(&context_pack_id)
        .bind(pack.kind.as_str())
        .bind(pack.subject_id.trim())
        .bind(&pack.content)
        .bind(&pack.metadata)
        .bind(pack.rebuildable)
        .fetch_one(&mut **transaction)
        .await?;
        let stored = row_to_context_pack(row)?;

        sqlx::query("DELETE FROM context_pack_sources WHERE context_pack_id = $1")
            .bind(&stored.context_pack_id)
            .execute(&mut **transaction)
            .await?;

        for source in sources {
            sqlx::query(
                r#"
                INSERT INTO context_pack_sources (
                    context_pack_id,
                    source_kind,
                    source_id,
                    role,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(&stored.context_pack_id)
            .bind(source.source_kind.as_str())
            .bind(source.source_id.trim())
            .bind(source.role.trim())
            .bind(&source.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Ok(stored)
    }
}

fn row_to_context_pack(row: PgRow) -> Result<ContextPack, ContextPackStoreError> {
    let kind: String = row.try_get("kind")?;
    Ok(ContextPack {
        context_pack_id: row.try_get("context_pack_id")?,
        kind: ContextPackKind::parse(kind)?,
        subject_id: row.try_get("subject_id")?,
        content: row.try_get("content")?,
        metadata: row.try_get("metadata")?,
        rebuildable: row.try_get("rebuildable")?,
        built_at: row.try_get("built_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_context_pack_source(row: PgRow) -> Result<ContextPackSource, ContextPackStoreError> {
    let source_kind: String = row.try_get("source_kind")?;
    Ok(ContextPackSource {
        context_pack_id: row.try_get("context_pack_id")?,
        source_kind: ContextPackSourceKind::parse(source_kind)?,
        source_id: row.try_get("source_id")?,
        role: row.try_get("role")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn context_pack_id(pack: &NewContextPack) -> Result<String, ContextPackStoreError> {
    let mut digest = Sha256::new();
    digest.update(pack.kind.as_str().as_bytes());
    digest.update(b"\n");
    digest.update(pack.subject_id.trim().as_bytes());
    Ok(format!("context_pack:v1:{:x}", digest.finalize()))
}
