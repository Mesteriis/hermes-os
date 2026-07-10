use chrono::{DateTime, Utc};
use thiserror::Error;

pub struct TaskIntelligenceService;

impl TaskIntelligenceService {
    pub fn calculate_priority(
        due_at: Option<DateTime<Utc>>,
        has_persona_link: bool,
        has_org: bool,
        has_project: bool,
        is_legal: bool,
        is_tax: bool,
        has_blockers: bool,
    ) -> f64 {
        let mut score: f64 = 0.2;
        if let Some(due) = due_at {
            let hours_left = (due - Utc::now()).num_hours();
            if hours_left <= 0 {
                score += 0.5;
            } else if hours_left <= 24 {
                score += 0.4;
            } else if hours_left <= 72 {
                score += 0.3;
            } else if hours_left <= 168 {
                score += 0.15;
            }
        }
        if is_legal || is_tax {
            score += 0.3;
        }
        if has_blockers {
            score += 0.15;
        }
        if has_persona_link {
            score += 0.1;
        }
        if has_org {
            score += 0.1;
        }
        if has_project {
            score += 0.05;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn calculate_risk(
        has_deadline_close: bool,
        missing_docs: bool,
        no_owner: bool,
        external_dep: bool,
        is_legal: bool,
        title: &str,
    ) -> f64 {
        let mut score: f64 = 0.1;
        if has_deadline_close {
            score += 0.3;
        }
        if missing_docs {
            score += 0.2;
        }
        if no_owner {
            score += 0.15;
        }
        if external_dep {
            score += 0.2;
        }
        if is_legal {
            score += 0.15;
        }
        let t = title.to_lowercase();
        if t.contains("urgent") || t.contains("asap") || t.contains("срочно") {
            score += 0.2;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn calculate_readiness(
        has_desc: bool,
        has_context: bool,
        has_docs: bool,
        has_deadline: bool,
        no_blockers: bool,
        personas_resolved: bool,
    ) -> f64 {
        let mut score: f64 = 0.0;
        if has_desc {
            score += 0.2;
        }
        if has_context {
            score += 0.2;
        }
        if has_docs {
            score += 0.15;
        }
        if has_deadline {
            score += 0.15;
        }
        if no_blockers {
            score += 0.15;
        }
        if personas_resolved {
            score += 0.15;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn detect_missing_context(
        has_desc: bool,
        has_context: bool,
        has_deadline: bool,
        has_persona_link: bool,
        has_project: bool,
    ) -> Vec<String> {
        let mut missing = Vec::new();
        if !has_desc {
            missing.push("No description".into());
        }
        if !has_context {
            missing.push("No context pack".into());
        }
        if !has_deadline {
            missing.push("No deadline".into());
        }
        if !has_persona_link {
            missing.push("No linked persona".into());
        }
        if !has_project {
            missing.push("No linked project".into());
        }
        missing
    }

    pub fn suggest_next_action(
        status: &str,
        _has_docs: bool,
        has_blockers: bool,
        waiting_reason: Option<&str>,
    ) -> String {
        match status {
            "new" | "triaged" => "Review and set priority".into(),
            "ready" => "Start working on this task".into(),
            "in_progress" => "Continue working".into(),
            "waiting" => format!("Follow up: {}", waiting_reason.unwrap_or("check status")),
            "blocked" => {
                if has_blockers {
                    "Resolve blockers first".into()
                } else {
                    "Investigate blocking reason".into()
                }
            }
            "review" => "Review and approve or request changes".into(),
            "done" => "Archive this task".into(),
            _ => "Review task context".into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum TaskIntelligenceError {
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),
}
