use thiserror::Error;

/// Basic ICS export for a single event
pub fn export_event_ics(
    title: &str,
    description: Option<&str>,
    location: Option<&str>,
    start_at: &str,
    end_at: &str,
    timezone: Option<&str>,
) -> String {
    let tz = timezone.unwrap_or("Europe/Madrid");
    let desc = description.unwrap_or("");
    let loc = location.unwrap_or("");
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Hermes Hub//Calendar//EN\r\nBEGIN:VEVENT\r\nDTSTART;TZID={tz}:{start}\r\nDTEND;TZID={tz}:{end}\r\nSUMMARY:{summary}\r\nDESCRIPTION:{desc}\r\nLOCATION:{loc}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        tz = tz,
        start = start_at,
        end = end_at,
        summary = title,
        desc = desc,
        loc = loc,
    )
}

/// Export event as markdown
pub fn export_event_md(
    title: &str,
    description: Option<&str>,
    location: Option<&str>,
    start_at: &str,
    end_at: &str,
    participants: &[String],
) -> String {
    let mut md = format!("# {}\n\n**When:** {} - {}\n\n", title, start_at, end_at);
    if let Some(loc) = location {
        if !loc.is_empty() {
            md.push_str(&format!("**Where:** {}\n\n", loc));
        }
    }
    if let Some(desc) = description {
        if !desc.is_empty() {
            md.push_str(&format!("{}\n\n", desc));
        }
    }
    if !participants.is_empty() {
        md.push_str("## Participants\n\n");
        for p in participants {
            md.push_str(&format!("- {}\n", p));
        }
    }
    md
}

#[derive(Debug, Error)]
pub enum CalendarSyncError {
    #[error("sync failed: {0}")]
    SyncFailed(String),
    #[error("import failed: {0}")]
    ImportFailed(String),
}
