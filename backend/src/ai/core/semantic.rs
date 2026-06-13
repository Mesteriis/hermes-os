use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::integrations::ai_runtime::AiRuntimeClient;

use super::constants::AI_EMBEDDING_DIMENSION;
use super::errors::AiError;
use super::helpers::{
    content_hash, halfvec_literal, recipients_text, semantic_embedding_id, validate_limit,
    validate_non_empty,
};

#[derive(Clone)]
pub struct SemanticEmbeddingStore {
    pool: PgPool,
}

impl SemanticEmbeddingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_embedding(
        &self,
        embedding: NewSemanticEmbedding<'_>,
    ) -> Result<SemanticEmbedding, AiError> {
        let source_id = validate_non_empty("source_id", embedding.source_id)?;
        let title = validate_non_empty("title", embedding.title)?;
        let source_text = validate_non_empty("source_text", embedding.source_text)?;
        let embedding_model = validate_non_empty("embedding_model", embedding.embedding_model)?;
        let content_hash = content_hash(&source_text);
        let vector_literal = halfvec_literal(embedding.embedding)?;
        let semantic_embedding_id =
            semantic_embedding_id(embedding.source_kind.as_str(), &source_id, &embedding_model);

        let row = sqlx::query(
            r#"
            INSERT INTO semantic_embeddings (
                semantic_embedding_id,
                source_kind,
                source_id,
                title,
                source_text,
                content_hash,
                embedding_model,
                embedding_dimension,
                embedding,
                graph_node_id,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::halfvec, $10, now())
            ON CONFLICT (source_kind, source_id, embedding_model)
            DO UPDATE SET
                title = EXCLUDED.title,
                source_text = EXCLUDED.source_text,
                content_hash = EXCLUDED.content_hash,
                embedding_dimension = EXCLUDED.embedding_dimension,
                embedding = EXCLUDED.embedding,
                graph_node_id = EXCLUDED.graph_node_id,
                updated_at = now()
            RETURNING
                semantic_embedding_id,
                source_kind,
                source_id,
                title,
                source_text,
                content_hash,
                embedding_model,
                embedding_dimension,
                graph_node_id,
                created_at,
                updated_at
            "#,
        )
        .bind(semantic_embedding_id)
        .bind(embedding.source_kind.as_str())
        .bind(&source_id)
        .bind(&title)
        .bind(&source_text)
        .bind(&content_hash)
        .bind(&embedding_model)
        .bind(AI_EMBEDDING_DIMENSION as i32)
        .bind(vector_literal)
        .bind(embedding.graph_node_id)
        .fetch_one(&self.pool)
        .await?;

        row_to_semantic_embedding(row)
    }

    pub async fn search(
        &self,
        embedding_model: &str,
        query_embedding: &[f32],
        limit: i64,
    ) -> Result<Vec<SemanticSearchResult>, AiError> {
        let embedding_model = validate_non_empty("embedding_model", embedding_model)?;
        let limit = validate_limit(limit)?;
        let vector_literal = halfvec_literal(query_embedding)?;

        let rows = sqlx::query(
            r#"
            SELECT
                source_kind,
                source_id,
                title,
                source_text,
                graph_node_id,
                (1.0 - (embedding <=> $2::halfvec))::DOUBLE PRECISION AS score
            FROM semantic_embeddings
            WHERE embedding_model = $1
            ORDER BY embedding <=> $2::halfvec ASC, updated_at DESC, source_id
            LIMIT $3
            "#,
        )
        .bind(&embedding_model)
        .bind(vector_literal)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_semantic_search_result)
            .collect()
    }

    pub(super) async fn text_search(
        &self,
        embedding_model: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<SemanticSearchResult>, AiError> {
        let embedding_model = validate_non_empty("embedding_model", embedding_model)?;
        let query = validate_non_empty("query", query)?;
        let limit = validate_limit(limit)?;

        let rows = sqlx::query(
            r#"
            WITH query AS (
                SELECT plainto_tsquery('simple', $2) AS ts_query
            )
            SELECT
                source_kind,
                source_id,
                title,
                source_text,
                graph_node_id,
                ts_rank_cd(
                    to_tsvector('simple', title || ' ' || source_text),
                    query.ts_query
                )::DOUBLE PRECISION AS score
            FROM semantic_embeddings, query
            WHERE embedding_model = $1
              AND to_tsvector('simple', title || ' ' || source_text) @@ query.ts_query
            ORDER BY score DESC, updated_at DESC, source_id
            LIMIT $3
            "#,
        )
        .bind(&embedding_model)
        .bind(&query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_semantic_search_result)
            .collect()
    }

    pub async fn index_canonical_sources(
        &self,
        runtime: &AiRuntimeClient,
        embedding_model: &str,
    ) -> Result<SemanticIndexReport, AiError> {
        let sources = self.canonical_sources().await?;
        let mut report = SemanticIndexReport::default();

        for source in sources {
            report.sources_seen += 1;
            let source_hash = content_hash(&source.source_text);
            if self
                .is_current(
                    source.source_kind,
                    &source.source_id,
                    embedding_model,
                    &source_hash,
                )
                .await?
            {
                report.sources_skipped += 1;
                continue;
            }

            let embedding = runtime
                .embed_with_model(&source.source_text, embedding_model)
                .await?;
            if embedding.embedding.len() != AI_EMBEDDING_DIMENSION {
                return Err(AiError::InvalidEmbeddingDimension {
                    expected: AI_EMBEDDING_DIMENSION,
                    actual: embedding.embedding.len(),
                });
            }
            self.upsert_embedding(NewSemanticEmbedding {
                source_kind: source.source_kind,
                source_id: &source.source_id,
                title: &source.title,
                source_text: &source.source_text,
                embedding_model,
                embedding: &embedding.embedding,
                graph_node_id: source.graph_node_id.as_deref(),
            })
            .await?;
            report.sources_indexed += 1;
        }

        Ok(report)
    }

    async fn is_current(
        &self,
        source_kind: SemanticSourceKind,
        source_id: &str,
        embedding_model: &str,
        content_hash: &str,
    ) -> Result<bool, AiError> {
        let current_hash = sqlx::query_scalar::<_, String>(
            r#"
            SELECT content_hash
            FROM semantic_embeddings
            WHERE source_kind = $1
              AND source_id = $2
              AND embedding_model = $3
            "#,
        )
        .bind(source_kind.as_str())
        .bind(source_id)
        .bind(embedding_model)
        .fetch_optional(&self.pool)
        .await?;

        Ok(current_hash.as_deref() == Some(content_hash))
    }

    async fn canonical_sources(&self) -> Result<Vec<SemanticSource>, AiError> {
        let mut sources = Vec::new();

        let message_rows = sqlx::query(
            r#"
            SELECT message_id, subject, sender, recipients, body_text
            FROM communication_messages
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        for row in message_rows {
            let message_id: String = row.try_get("message_id")?;
            let subject: String = row.try_get("subject")?;
            let sender: String = row.try_get("sender")?;
            let recipients = recipients_text(row.try_get("recipients")?);
            let body_text: String = row.try_get("body_text")?;
            sources.push(SemanticSource {
                source_kind: SemanticSourceKind::Message,
                source_id: message_id.clone(),
                title: subject.clone(),
                source_text: format!(
                    "Subject: {subject}\nFrom: {sender}\nTo: {recipients}\n\n{body_text}"
                ),
                graph_node_id: Some(node_id(GraphNodeKind::Message, &message_id)),
            });
        }

        let document_rows = sqlx::query(
            r#"
            SELECT document_id, title, extracted_text
            FROM documents
            WHERE length(trim(extracted_text)) > 0
            ORDER BY imported_at DESC, document_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        for row in document_rows {
            let document_id: String = row.try_get("document_id")?;
            let title: String = row.try_get("title")?;
            let extracted_text: String = row.try_get("extracted_text")?;
            sources.push(SemanticSource {
                source_kind: SemanticSourceKind::Document,
                source_id: document_id.clone(),
                title: title.clone(),
                source_text: format!("{title}\n\n{extracted_text}"),
                graph_node_id: Some(node_id(GraphNodeKind::Document, &document_id)),
            });
        }

        let project_rows = sqlx::query(
            r#"
            SELECT
                p.project_id,
                p.name,
                p.kind,
                p.status,
                p.description,
                p.owner_display_name,
                COALESCE(string_agg(k.keyword, ', ' ORDER BY k.keyword), '') AS keywords
            FROM projects p
            LEFT JOIN project_keywords k ON k.project_id = p.project_id
            GROUP BY p.project_id
            ORDER BY p.updated_at DESC, p.project_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        for row in project_rows {
            let project_id: String = row.try_get("project_id")?;
            let name: String = row.try_get("name")?;
            let kind: String = row.try_get("kind")?;
            let status: String = row.try_get("status")?;
            let description: String = row.try_get("description")?;
            let owner: String = row.try_get("owner_display_name")?;
            let keywords: String = row.try_get("keywords")?;
            sources.push(SemanticSource {
                source_kind: SemanticSourceKind::Project,
                source_id: project_id.clone(),
                title: name.clone(),
                source_text: format!(
                    "{name}\nKind: {kind}\nStatus: {status}\nOwner: {owner}\nKeywords: {keywords}\n\n{description}"
                ),
                graph_node_id: Some(node_id(GraphNodeKind::Project, &project_id)),
            });
        }

        let task_rows = sqlx::query(
            r#"
            SELECT task_id, title, source_kind, source_id, status
            FROM tasks
            ORDER BY updated_at DESC, task_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        for row in task_rows {
            let task_id: String = row.try_get("task_id")?;
            let title: String = row.try_get("title")?;
            let source_kind: String = row.try_get("source_kind")?;
            let source_id: String = row.try_get("source_id")?;
            let status: String = row.try_get("status")?;
            sources.push(SemanticSource {
                source_kind: SemanticSourceKind::Task,
                source_id: task_id,
                title: title.clone(),
                source_text: format!(
                    "{title}\nStatus: {status}\nSource: {source_kind}:{source_id}"
                ),
                graph_node_id: None,
            });
        }

        let person_rows = sqlx::query(
            r#"
            SELECT person_id, display_name, email_address
            FROM persons
            ORDER BY updated_at DESC, person_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        for row in person_rows {
            let person_id: String = row.try_get("person_id")?;
            let display_name: String = row.try_get("display_name")?;
            let email_address: String = row.try_get("email_address")?;
            sources.push(SemanticSource {
                source_kind: SemanticSourceKind::Person,
                source_id: person_id,
                title: display_name.clone(),
                source_text: format!("{display_name}\nEmail: {email_address}"),
                graph_node_id: None,
            });
        }

        Ok(sources)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticSourceKind {
    Message,
    Document,
    Project,
    Task,
    Person,
}

impl SemanticSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
            Self::Project => "project",
            Self::Task => "task",
            Self::Person => "contact",
        }
    }

    fn parse(value: &str) -> Result<Self, AiError> {
        match value {
            "message" => Ok(Self::Message),
            "document" => Ok(Self::Document),
            "project" => Ok(Self::Project),
            "task" => Ok(Self::Task),
            "contact" | "person" => Ok(Self::Person),
            _ => Err(AiError::InvalidSourceKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticEmbedding {
    pub semantic_embedding_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub title: String,
    pub source_text: String,
    pub content_hash: String,
    pub embedding_model: String,
    pub embedding_dimension: i32,
    pub graph_node_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug)]
pub struct NewSemanticEmbedding<'a> {
    pub source_kind: SemanticSourceKind,
    pub source_id: &'a str,
    pub title: &'a str,
    pub source_text: &'a str,
    pub embedding_model: &'a str,
    pub embedding: &'a [f32],
    pub graph_node_id: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSearchResult {
    pub source_kind: String,
    pub source_id: String,
    pub title: String,
    pub source_text: String,
    pub graph_node_id: Option<String>,
    pub score: f64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticIndexReport {
    pub sources_seen: usize,
    pub sources_indexed: usize,
    pub sources_skipped: usize,
}

struct SemanticSource {
    source_kind: SemanticSourceKind,
    source_id: String,
    title: String,
    source_text: String,
    graph_node_id: Option<String>,
}

fn row_to_semantic_embedding(row: PgRow) -> Result<SemanticEmbedding, AiError> {
    let source_kind: String = row.try_get("source_kind")?;
    SemanticSourceKind::parse(&source_kind)?;
    Ok(SemanticEmbedding {
        semantic_embedding_id: row.try_get("semantic_embedding_id")?,
        source_kind,
        source_id: row.try_get("source_id")?,
        title: row.try_get("title")?,
        source_text: row.try_get("source_text")?,
        content_hash: row.try_get("content_hash")?,
        embedding_model: row.try_get("embedding_model")?,
        embedding_dimension: row.try_get("embedding_dimension")?,
        graph_node_id: row.try_get("graph_node_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_semantic_search_result(row: PgRow) -> Result<SemanticSearchResult, AiError> {
    let source_kind: String = row.try_get("source_kind")?;
    SemanticSourceKind::parse(&source_kind)?;
    Ok(SemanticSearchResult {
        source_kind,
        source_id: row.try_get("source_id")?,
        title: row.try_get("title")?,
        source_text: row.try_get("source_text")?,
        graph_node_id: row.try_get("graph_node_id")?,
        score: row.try_get("score")?,
    })
}
