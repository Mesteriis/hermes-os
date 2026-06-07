use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub task_candidate_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub source_kind: String,
    pub source_id: String,
    pub source_type: String,
    pub project_id: Option<String>,
    pub status: String,
    pub hermes_status: String,
    pub priority_score: Option<f64>,
    pub risk_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub area: Option<String>,
    pub why: Option<String>,
    pub outcome: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub waiting_reason: Option<String>,
    pub energy_type: Option<String>,
    pub confidentiality: String,
    pub tags: Value,
    pub task_metadata: Value,
    pub linked_person_id: Option<String>,
    pub linked_organization_id: Option<String>,
    pub created_from_event_id: Option<String>,
    pub created_by_actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskStore {
    pool: PgPool,
}

impl TaskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &NewTask) -> Result<Task, TaskError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let task_id = format!("task:v1:{:x}", ts);
        let row = sqlx::query(
            "INSERT INTO tasks (task_id, title, description, source_kind, source_id, source_type, project_id, hermes_status, priority_score, area, why, due_at, energy_type, confidentiality, tags, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
             RETURNING task_id, task_candidate_id, title, description, source_kind, source_id, source_type, project_id, status, hermes_status, priority_score, risk_score, readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        ).bind(&task_id).bind(&req.title).bind(req.description.as_deref())
         .bind(req.source_kind.as_deref().unwrap_or("manual")).bind(req.source_id.as_deref().unwrap_or("manual"))
         .bind(req.source_type.as_deref().unwrap_or("manual")).bind(req.project_id.as_deref())
         .bind(req.hermes_status.as_deref().unwrap_or("new")).bind(req.priority_score)
         .bind(req.area.as_deref()).bind(req.why.as_deref()).bind(req.due_at)
         .bind(req.energy_type.as_deref()).bind(req.confidentiality.as_deref().unwrap_or("private_local"))
         .bind(&req.tags).bind(req.linked_person_id.as_deref()).bind(req.linked_organization_id.as_deref())
         .bind(req.created_from_event_id.as_deref()).bind(req.created_by_actor_id.as_deref())
         .fetch_one(&self.pool).await?;
        Ok(row_to_task(row)?)
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<Task>, TaskError> {
        let row = sqlx::query("SELECT task_id, task_candidate_id, title, description, source_kind, source_id, source_type, project_id, status, hermes_status, priority_score, risk_score, readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE task_id=$1")
            .bind(task_id).fetch_optional(&self.pool).await?;
        row.map(|r| row_to_task(r).map_err(TaskError::from))
            .transpose()
    }

    pub async fn list(&self, query: &TaskListQuery) -> Result<Vec<Task>, TaskError> {
        let limit = query.limit.unwrap_or(100).clamp(1, 500);
        let rows = sqlx::query(
            "SELECT task_id, task_candidate_id, title, description, source_kind, source_id, source_type, project_id, status, hermes_status, priority_score, risk_score, readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE ($1::text IS NULL OR hermes_status=$1) AND ($2::text IS NULL OR project_id=$2) AND ($3::text IS NULL OR source_type=$3) ORDER BY COALESCE(priority_score,0) DESC, due_at ASC NULLS LAST, created_at DESC LIMIT $4"
        ).bind(query.status.as_deref()).bind(query.project_id.as_deref()).bind(query.source_type.as_deref()).bind(limit)
         .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(TaskError::from)
    }

    pub async fn update(&self, task_id: &str, update: &TaskUpdate) -> Result<Task, TaskError> {
        let row = sqlx::query(
            "UPDATE tasks SET title=COALESCE($2,title), description=COALESCE($3,description), hermes_status=COALESCE($4,hermes_status), priority_score=COALESCE($5,priority_score), risk_score=COALESCE($6,risk_score), readiness_score=COALESCE($7,readiness_score), area=COALESCE($8,area), why=COALESCE($9,why), outcome=COALESCE($10,outcome), due_at=COALESCE($11,due_at), waiting_reason=COALESCE($12,waiting_reason), energy_type=COALESCE($13,energy_type), confidentiality=COALESCE($14,confidentiality), tags=COALESCE($15,tags), task_metadata=COALESCE($16,task_metadata), linked_person_id=COALESCE($17,linked_person_id), linked_organization_id=COALESCE($18,linked_organization_id), completed_at=COALESCE($19,completed_at), updated_at=now() WHERE task_id=$1 RETURNING task_id, task_candidate_id, title, description, source_kind, source_id, source_type, project_id, status, hermes_status, priority_score, risk_score, readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        ).bind(task_id).bind(update.title.as_deref()).bind(update.description.as_deref())
         .bind(update.hermes_status.as_deref()).bind(update.priority_score).bind(update.risk_score).bind(update.readiness_score)
         .bind(update.area.as_deref()).bind(update.why.as_deref()).bind(update.outcome.as_deref())
         .bind(update.due_at).bind(update.waiting_reason.as_deref()).bind(update.energy_type.as_deref())
         .bind(update.confidentiality.as_deref()).bind(update.tags.as_ref()).bind(update.task_metadata.as_ref())
         .bind(update.linked_person_id.as_deref()).bind(update.linked_organization_id.as_deref())
         .bind(update.completed_at)
         .fetch_one(&self.pool).await?;
        Ok(row_to_task(row)?)
    }

    pub async fn set_status(&self, task_id: &str, status: &str) -> Result<(), TaskError> {
        let mut q =
            sqlx::query("UPDATE tasks SET hermes_status=$2, updated_at=now() WHERE task_id=$1");
        if status == "done" || status == "completed" {
            q = sqlx::query(
                "UPDATE tasks SET hermes_status=$2, completed_at=now(), updated_at=now() WHERE task_id=$1",
            );
        } else if status == "archived" {
            q = sqlx::query(
                "UPDATE tasks SET hermes_status=$2, archived_at=now(), updated_at=now() WHERE task_id=$1",
            );
        }
        q.bind(task_id).bind(status).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn archive(&self, task_id: &str) -> Result<(), TaskError> {
        self.set_status(task_id, "archived").await
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub source_kind: Option<String>,
    pub source_id: Option<String>,
    pub source_type: Option<String>,
    pub project_id: Option<String>,
    pub hermes_status: Option<String>,
    pub priority_score: Option<f64>,
    pub area: Option<String>,
    pub why: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub energy_type: Option<String>,
    pub confidentiality: Option<String>,
    pub tags: Option<Value>,
    pub linked_person_id: Option<String>,
    pub linked_organization_id: Option<String>,
    pub created_from_event_id: Option<String>,
    pub created_by_actor_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub hermes_status: Option<String>,
    pub priority_score: Option<f64>,
    pub risk_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub area: Option<String>,
    pub why: Option<String>,
    pub outcome: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub waiting_reason: Option<String>,
    pub energy_type: Option<String>,
    pub confidentiality: Option<String>,
    pub tags: Option<Value>,
    pub task_metadata: Option<Value>,
    pub linked_person_id: Option<String>,
    pub linked_organization_id: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TaskListQuery {
    pub status: Option<String>,
    pub project_id: Option<String>,
    pub source_type: Option<String>,
    pub limit: Option<i64>,
}

fn row_to_task(row: PgRow) -> Result<Task, sqlx::Error> {
    Ok(Task {
        task_id: row.try_get("task_id")?,
        task_candidate_id: row.try_get("task_candidate_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        source_type: row.try_get("source_type")?,
        project_id: row.try_get("project_id")?,
        status: row.try_get("status")?,
        hermes_status: row.try_get("hermes_status")?,
        priority_score: row.try_get("priority_score")?,
        risk_score: row.try_get("risk_score")?,
        readiness_score: row.try_get("readiness_score")?,
        area: row.try_get("area")?,
        why: row.try_get("why")?,
        outcome: row.try_get("outcome")?,
        due_at: row.try_get("due_at")?,
        completed_at: row.try_get("completed_at")?,
        archived_at: row.try_get("archived_at")?,
        waiting_reason: row.try_get("waiting_reason")?,
        energy_type: row.try_get("energy_type")?,
        confidentiality: row.try_get("confidentiality")?,
        tags: row.try_get("tags")?,
        task_metadata: row.try_get("task_metadata")?,
        linked_person_id: row.try_get("linked_person_id")?,
        linked_organization_id: row.try_get("linked_organization_id")?,
        created_from_event_id: row.try_get("created_from_event_id")?,
        created_by_actor_id: row.try_get("created_by_actor_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
