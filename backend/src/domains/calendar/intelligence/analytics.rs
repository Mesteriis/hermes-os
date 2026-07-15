use chrono::{DateTime, Utc};

use super::CalendarIntelligenceService;
use super::models::BackToBackGroup;

impl CalendarIntelligenceService {
    pub fn categorize_time(event_type: &str, title: &str) -> String {
        let title = title.to_lowercase();
        match event_type {
            "meeting" => "meetings".into(),
            "focus" => "focus".into(),
            "deadline" => "deadlines".into(),
            "travel" => "travel".into(),
            "tax" | "legal" | "government" => "admin".into(),
            "finance" => "finance".into(),
            "personal" | "birthday" => "personal".into(),
            "review" | "planning" => "planning".into(),
            _ => fallback_time_category(&title),
        }
    }

    pub fn detect_back_to_back(
        events: &[(DateTime<Utc>, DateTime<Utc>, String)],
    ) -> Vec<BackToBackGroup> {
        let mut sorted = events.to_vec();
        sorted.sort_by_key(|(start_at, _, _)| *start_at);
        let mut groups = Vec::new();
        let mut current: Vec<String> = Vec::new();

        for window in sorted.windows(2) {
            let (_, first_end_at, first_title) = &window[0];
            let (second_start_at, _, second_title) = &window[1];
            let gap = (*second_start_at - *first_end_at).num_minutes();

            if gap <= 5 && current.is_empty() {
                current.push(first_title.clone());
                current.push(second_title.clone());
            } else if gap <= 5 {
                current.push(second_title.clone());
            } else if !current.is_empty() {
                groups.push(BackToBackGroup {
                    titles: current.clone(),
                    count: current.len(),
                });
                current.clear();
            }
        }

        if !current.is_empty() {
            groups.push(BackToBackGroup {
                titles: current.clone(),
                count: current.len(),
            });
        }
        groups
    }
}

fn fallback_time_category(title: &str) -> String {
    if title.contains("meeting") || title.contains("call") {
        "meetings".into()
    } else if title.contains("focus") {
        "focus".into()
    } else if title.contains("lunch") || title.contains("dinner") || title.contains("coffee") {
        "personal".into()
    } else {
        "other".into()
    }
}
