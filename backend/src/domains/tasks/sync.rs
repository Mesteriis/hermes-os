use serde_json::{Value, json};
use thiserror::Error;

pub fn export_task_md(
    title: &str,
    description: Option<&str>,
    status: &str,
    why: Option<&str>,
    outcome: Option<&str>,
) -> String {
    let mut md = format!("# {title}\n\n**Status:** {status}\n\n");
    if let Some(why) = why
        && !why.is_empty()
    {
        md.push_str(&format!("**Why:** {why}\n\n"));
    }
    if let Some(desc) = description
        && !desc.is_empty()
    {
        md.push_str(&format!("{desc}\n\n"));
    }
    if let Some(out) = outcome
        && !out.is_empty()
    {
        md.push_str(&format!("**Outcome:** {out}\n\n"));
    }
    md
}

pub fn export_task_json(
    title: &str,
    description: Option<&str>,
    status: &str,
    priority: Option<f64>,
    due_at: Option<&str>,
) -> Value {
    json!({ "title": title, "description": description, "status": status, "priority": priority, "due_at": due_at })
}

#[derive(Debug, Error)]
pub enum TaskSyncError {
    #[error("sync failed: {0}")]
    SyncFailed(String),
}
