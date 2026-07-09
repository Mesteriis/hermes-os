// §20-21: Task + Note extraction from email via LLM + heuristics
use crate::ai::hub::{AiHubError, AiModelRoute, SharedAiHub};
use crate::domains::communications::messages::ProjectedMessage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedTask {
    pub title: String,
    pub due_date: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<String>,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedNote {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone)]
pub struct EmailExtractService {
    hub: Option<SharedAiHub>,
}

impl EmailExtractService {
    pub fn new(hub: Option<SharedAiHub>) -> Self {
        Self { hub }
    }

    pub async fn extract_tasks(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Vec<ExtractedTask>, ExtractError> {
        let mut tasks = Vec::new();
        let body = &message.body_text;

        // Heuristic: look for task-like patterns
        for line in body.lines() {
            let ll = line.trim().to_lowercase();
            let is_task = ll.starts_with("todo:")
                || ll.starts_with("task:")
                || ll.starts_with("- [ ]")
                || ll.contains("action item")
                || ll.contains("please ") && ll.contains(" by ");
            if is_task && ll.len() > 10 {
                let due = extract_due_date(line);
                tasks.push(ExtractedTask {
                    title: line
                        .trim()
                        .trim_start_matches(['-', '[', ']', ' '])
                        .to_owned(),
                    due_date: due,
                    assignee: None,
                    priority: if ll.contains("urgent") {
                        Some("high".into())
                    } else {
                        Some("normal".into())
                    },
                    source: "heuristic".into(),
                });
            }
        }

        // LLM extraction if available
        if let Some(ref hub) = self.hub {
            let prompt = format!(
                "Extract tasks from this email. Return a JSON array of objects with fields: title, due_date (ISO date or null), assignee (or null), priority (high/medium/low).\n\nEmail:\nSubject: {}\nBody:\n{}",
                message.subject,
                truncate(body, 3000)
            );
            if let Ok(result) = hub.chat(AiModelRoute::Extraction, &prompt).await
                && let Ok(mut llm_tasks) =
                    serde_json::from_str::<Vec<ExtractedTask>>(result.content.trim())
            {
                for t in &mut llm_tasks {
                    t.source = "ai_hub.external_llm".into();
                }
                tasks.extend(llm_tasks);
            }
        }
        Ok(tasks)
    }

    pub async fn extract_notes(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Vec<ExtractedNote>, ExtractError> {
        let mut notes = Vec::new();
        let body = &message.body_text;
        let lower = body.to_lowercase();

        // Heuristic: important information patterns
        let has_finance =
            lower.contains("invoice") || lower.contains("payment") || lower.contains("amount");
        let has_legal =
            lower.contains("contract") || lower.contains("agreement") || lower.contains("nda");
        let has_decision =
            lower.contains("decided") || lower.contains("approved") || lower.contains("confirmed");
        let has_deadline =
            lower.contains("deadline") || lower.contains("due date") || lower.contains("by ");

        if has_finance || has_legal || has_decision || has_deadline {
            let mut tags = Vec::new();
            if has_finance {
                tags.push("finance".into());
            }
            if has_legal {
                tags.push("legal".into());
            }
            if has_decision {
                tags.push("decision".into());
            }

            let preview = body.lines().take(5).collect::<Vec<_>>().join("\n");
            notes.push(ExtractedNote {
                title: message.subject.clone(),
                content: preview,
                tags,
                source: "heuristic".into(),
            });
        }
        Ok(notes)
    }
}

fn extract_due_date(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for prefix in &["by ", "due ", "deadline", "before "] {
        if let Some(pos) = lower.find(prefix) {
            let after = &text[pos + prefix.len()..];
            let rest = after.trim_start_matches([':', ' ']);
            return Some(
                rest.split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error(transparent)]
    Hub(#[from] AiHubError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extract_due_date_by() {
        assert_eq!(
            extract_due_date("Please submit by Friday 5pm"),
            Some("Friday 5pm".into())
        );
    }
    #[test]
    fn extract_due_date_deadline() {
        assert_eq!(
            extract_due_date("Deadline: 2026-06-15 for submission"),
            Some("2026-06-15 for submission".into())
        );
    }
    #[test]
    fn extract_due_date_none() {
        assert_eq!(extract_due_date("Hello, how are you?"), None);
    }
}
