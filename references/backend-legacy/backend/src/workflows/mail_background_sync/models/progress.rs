#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncTrigger {
    Scheduled,
    Manual,
}

impl MailSyncTrigger {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }
}

pub(in crate::workflows::mail_background_sync) struct ProgressUpdate<'a> {
    pub(in crate::workflows::mail_background_sync) run_id: &'a str,
    pub(in crate::workflows::mail_background_sync) phase: MailSyncPhase,
    pub(in crate::workflows::mail_background_sync) progress_mode: ProgressMode,
    pub(in crate::workflows::mail_background_sync) progress_percent: Option<i32>,
    pub(in crate::workflows::mail_background_sync) processed_messages: i64,
    pub(in crate::workflows::mail_background_sync) estimated_total_messages: Option<i64>,
    pub(in crate::workflows::mail_background_sync) current_batch_size: i32,
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync) enum MailSyncRunStatus {
    Completed,
    Failed,
    Skipped,
}

impl MailSyncRunStatus {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync) enum MailSyncPhase {
    Listing,
    Fetching,
    Projecting,
    PersonasGraph,
    Completed,
    Failed,
}

impl MailSyncPhase {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::Listing => "listing",
            Self::Fetching => "fetching",
            Self::Projecting => "projecting",
            Self::PersonasGraph => "personas_graph",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync) enum ProgressMode {
    None,
    Determinate,
    Indeterminate,
}

impl ProgressMode {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Determinate => "determinate",
            Self::Indeterminate => "indeterminate",
        }
    }
}
