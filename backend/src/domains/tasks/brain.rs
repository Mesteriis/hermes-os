use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

pub struct TaskBrainService;

impl TaskBrainService {
    pub async fn explain_task(pool: &PgPool, task_id: &str) -> Result<Value, TaskBrainError> {
        let task = sqlx::query("SELECT task_id, title, description, source_type, hermes_status, why, outcome, due_at FROM tasks WHERE task_id=$1")
            .bind(task_id).fetch_optional(pool).await?;
        let task = task.ok_or(TaskBrainError::NotFound)?;

        let ctx = sqlx::query("SELECT summary, blockers, risks, suggested_next_action FROM task_context_packs WHERE task_id=$1 ORDER BY generated_at DESC LIMIT 1")
            .bind(task_id).fetch_optional(pool).await?;

        let evidence =
            sqlx::query("SELECT source_type, quote FROM task_evidence WHERE task_id=$1 LIMIT 5")
                .bind(task_id)
                .fetch_all(pool)
                .await?;

        Ok(json!({
            "task_id": task.try_get::<String,_>("task_id").unwrap_or_default(),
            "title": task.try_get::<String,_>("title").unwrap_or_default(),
            "description": task.try_get::<Option<String>,_>("description").unwrap_or(None),
            "what": format!("Task: {}", task.try_get::<String,_>("title").unwrap_or_default()),
            "why": task.try_get::<Option<String>,_>("why").unwrap_or(Some("No reason recorded".into())),
            "status": task.try_get::<String,_>("hermes_status").unwrap_or_default(),
            "source": task.try_get::<String,_>("source_type").unwrap_or_default(),
            "context": ctx.map(|r| json!({
                "summary": r.try_get::<Option<String>,_>("summary").unwrap_or(None),
                "blockers": r.try_get::<Value,_>("blockers").unwrap_or(json!([])),
                "risks": r.try_get::<Value,_>("risks").unwrap_or(json!([])),
                "next_action": r.try_get::<Option<String>,_>("suggested_next_action").unwrap_or(None),
            })),
            "evidence": evidence.iter().map(|r| json!({
                "source": r.try_get::<String,_>("source_type").unwrap_or_default(),
                "quote": r.try_get::<Option<String>,_>("quote").unwrap_or(None),
            })).collect::<Vec<_>>(),
        }))
    }

    pub async fn search_tasks(pool: &PgPool, query: &str) -> Result<Value, TaskBrainError> {
        let pattern = format!("%{query}%");
        let rows = sqlx::query("SELECT task_id, title, hermes_status, priority_score, due_at FROM tasks WHERE title ILIKE $1 OR description ILIKE $1 ORDER BY COALESCE(priority_score,0) DESC LIMIT 20")
            .bind(&pattern).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                    "title": r.try_get::<String,_>("title").unwrap_or_default(),
                    "status": r.try_get::<String,_>("hermes_status").unwrap_or_default(),
                    "priority": r.try_get::<Option<f64>,_>("priority_score").unwrap_or(None),
                    "due_at": r.try_get::<Option<DateTime<Utc>>,_>("due_at").unwrap_or(None),
                })
            })
            .collect();
        Ok(json!({"query": query, "results": items}))
    }

    pub async fn daily_brief(pool: &PgPool) -> Result<Value, TaskBrainError> {
        let now = Utc::now();
        let _today_end = now
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .map(|d| DateTime::from_naive_utc_and_offset(d, Utc))
            .unwrap_or(now);

        let active = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE hermes_status IN ('new','triaged','ready','in_progress','waiting','blocked','review')")
            .fetch_one(pool).await?;
        let overdue = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE due_at < $1 AND hermes_status NOT IN ('done','cancelled','archived')")
            .bind(now).fetch_one(pool).await?;
        let high_risk = sqlx::query("SELECT task_id, title FROM tasks WHERE risk_score > 0.7 AND hermes_status NOT IN ('done','cancelled','archived') ORDER BY risk_score DESC LIMIT 5")
            .fetch_all(pool).await?;

        Ok(json!({
            "active_tasks": active.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
            "overdue": overdue.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
            "high_risk": high_risk.iter().map(|r| json!({
                "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                "title": r.try_get::<String,_>("title").unwrap_or_default(),
            })).collect::<Vec<_>>(),
        }))
    }
}

#[derive(Debug, Error)]
pub enum TaskBrainError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
