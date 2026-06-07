use serde_json::{Value, json};
use thiserror::Error;

pub fn export_task_md(
    title: &str,
    description: Option<&str>,
    status: &str,
    why: Option<&str>,
    outcome: Option<&str>,
) -> String {
    let mut md = format!("# {}\n\n**Status:** {}\n\n", title, status);
    if let Some(why) = why {
        if !why.is_empty() {
            md.push_str(&format!("**Why:** {}\n\n", why));
        }
    }
    if let Some(desc) = description {
        if !desc.is_empty() {
            md.push_str(&format!("{}\n\n", desc));
        }
    }
    if let Some(out) = outcome {
        if !out.is_empty() {
            md.push_str(&format!("**Outcome:** {}\n\n", out));
        }
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
