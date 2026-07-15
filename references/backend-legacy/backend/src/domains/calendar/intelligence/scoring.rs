use super::CalendarIntelligenceService;
use super::classification::contains_any;

impl CalendarIntelligenceService {
    pub fn calculate_importance(
        title: &str,
        participants_count: usize,
        has_project: bool,
        has_deadline: bool,
    ) -> f64 {
        let mut score: f64 = 0.3;
        let title = title.to_lowercase();
        if contains_any(&title, &["urgent", "asap", "срочно"]) {
            score += 0.3;
        }
        if contains_any(&title, &["client", "клиент", "customer"]) {
            score += 0.2;
        }
        if contains_any(&title, &["tax", "legal", "aeat"]) {
            score += 0.2;
        }
        if participants_count > 2 {
            score += 0.1;
        }
        if has_project {
            score += 0.1;
        }
        if has_deadline {
            score += 0.1;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn calculate_readiness(
        has_agenda: bool,
        has_docs: bool,
        has_context: bool,
        has_checklist: bool,
        has_participants: bool,
    ) -> f64 {
        let mut score: f64 = 0.0;
        if has_agenda {
            score += 0.25;
        }
        if has_docs {
            score += 0.2;
        }
        if has_context {
            score += 0.2;
        }
        if has_checklist {
            score += 0.15;
        }
        if has_participants {
            score += 0.2;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn detect_risks(
        has_agenda: bool,
        has_docs: bool,
        has_participants: bool,
        has_project: bool,
        is_upcoming_soon: bool,
    ) -> Vec<String> {
        let mut risks = Vec::new();
        if !has_agenda {
            risks.push("No agenda prepared".into());
        }
        if !has_docs {
            risks.push("No documents attached".into());
        }
        if !has_participants {
            risks.push("No participants resolved".into());
        }
        if !has_project {
            risks.push("Not linked to a project".into());
        }
        if is_upcoming_soon && (!has_agenda || !has_docs) {
            risks.push("Event is soon but preparation incomplete".into());
        }
        risks
    }
}
