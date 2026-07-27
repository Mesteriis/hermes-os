//! Public Mail-owned sync execution journal and provider-path health contract.

pub const MAX_SYNC_HEALTH_ID_BYTES: usize = 512;
pub const MAX_SYNC_HEALTH_CURSOR_BYTES: usize = 512;
pub const MAX_SYNC_HEALTH_PAGE_SIZE: u32 = 200;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailSyncHealthQueryV1 {
    GetStatus {
        connection_id: String,
    },
    ListRuns {
        connection_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    GetRun {
        connection_id: String,
        operation_id: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncTriggerV1 {
    Manual,
    Scheduled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncOutcomeV1 {
    Running,
    Succeeded,
    Failed,
    Interrupted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncFailureCodeV1 {
    AdmissionRejected,
    ControlUnavailable,
    StorageUnavailable,
    CredentialUnavailable,
    PersistenceUnavailable,
    ProviderUnavailable,
    EventHubUnavailable,
    AttachmentAnchorUnavailable,
    RuntimeRestarted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncProviderPathReadinessV1 {
    Ready,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncRunV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub trigger: MailSyncTriggerV1,
    pub outcome: MailSyncOutcomeV1,
    pub observed_messages: u64,
    pub started_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
    pub failure_code: Option<MailSyncFailureCodeV1>,
    pub runtime_generation: u64,
    pub projection_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncStatusV1 {
    pub connection_id: String,
    pub provider_path_readiness: MailSyncProviderPathReadinessV1,
    pub latest_run: Option<Box<MailSyncRunV1>>,
    pub consecutive_failures: u32,
    pub last_success_at_unix_seconds: Option<i64>,
    pub projection_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncRunPageV1 {
    pub items: Vec<MailSyncRunV1>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailSyncHealthQueryResponseV1 {
    Status(MailSyncStatusV1),
    Runs(MailSyncRunPageV1),
    Run(Option<MailSyncRunV1>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncHealthContractErrorV1 {
    InvalidId,
    InvalidCursor,
    InvalidLimit,
    InvalidRun,
    InvalidStatus,
}

#[must_use]
pub fn sync_health_query_connection_id(query: &MailSyncHealthQueryV1) -> &str {
    match query {
        MailSyncHealthQueryV1::GetStatus { connection_id }
        | MailSyncHealthQueryV1::ListRuns { connection_id, .. }
        | MailSyncHealthQueryV1::GetRun { connection_id, .. } => connection_id,
    }
}

pub fn validate_sync_health_query(
    query: &MailSyncHealthQueryV1,
) -> Result<(), MailSyncHealthContractErrorV1> {
    validate_id(sync_health_query_connection_id(query))?;
    match query {
        MailSyncHealthQueryV1::GetStatus { .. } => Ok(()),
        MailSyncHealthQueryV1::ListRuns { cursor, limit, .. } => {
            if !(1..=MAX_SYNC_HEALTH_PAGE_SIZE).contains(limit) {
                return Err(MailSyncHealthContractErrorV1::InvalidLimit);
            }
            validate_cursor(cursor.as_deref())
        }
        MailSyncHealthQueryV1::GetRun { operation_id, .. } => validate_id(operation_id),
    }
}

pub fn validate_sync_run(run: &MailSyncRunV1) -> Result<(), MailSyncHealthContractErrorV1> {
    validate_id(&run.operation_id)?;
    validate_id(&run.connection_id)?;
    let terminal = !matches!(run.outcome, MailSyncOutcomeV1::Running);
    let failure = matches!(
        run.outcome,
        MailSyncOutcomeV1::Failed | MailSyncOutcomeV1::Interrupted
    );
    if run.started_at_unix_seconds <= 0
        || run.runtime_generation == 0
        || run.projection_revision == 0
        || terminal != run.completed_at_unix_seconds.is_some()
        || failure != run.failure_code.is_some()
        || run
            .completed_at_unix_seconds
            .is_some_and(|completed| completed < run.started_at_unix_seconds)
    {
        return Err(MailSyncHealthContractErrorV1::InvalidRun);
    }
    Ok(())
}

pub fn validate_sync_health_response(
    response: &MailSyncHealthQueryResponseV1,
) -> Result<(), MailSyncHealthContractErrorV1> {
    match response {
        MailSyncHealthQueryResponseV1::Status(status) => {
            validate_id(&status.connection_id)?;
            if status.projection_revision == 0
                || status
                    .last_success_at_unix_seconds
                    .is_some_and(|value| value <= 0)
                || status.latest_run.as_ref().is_some_and(|run| {
                    run.connection_id != status.connection_id || validate_sync_run(run).is_err()
                })
            {
                return Err(MailSyncHealthContractErrorV1::InvalidStatus);
            }
        }
        MailSyncHealthQueryResponseV1::Runs(page) => {
            validate_cursor(page.next_cursor.as_deref())?;
            if page.items.len() > MAX_SYNC_HEALTH_PAGE_SIZE as usize
                || page.items.iter().any(|run| validate_sync_run(run).is_err())
            {
                return Err(MailSyncHealthContractErrorV1::InvalidRun);
            }
        }
        MailSyncHealthQueryResponseV1::Run(run) => {
            if run
                .as_ref()
                .is_some_and(|run| validate_sync_run(run).is_err())
            {
                return Err(MailSyncHealthContractErrorV1::InvalidRun);
            }
        }
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), MailSyncHealthContractErrorV1> {
    if value.is_empty()
        || value.len() > MAX_SYNC_HEALTH_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(MailSyncHealthContractErrorV1::InvalidId);
    }
    Ok(())
}

fn validate_cursor(cursor: Option<&str>) -> Result<(), MailSyncHealthContractErrorV1> {
    if cursor.is_some_and(|value| {
        value.is_empty()
            || value.len() > MAX_SYNC_HEALTH_CURSOR_BYTES
            || value.chars().any(char::is_control)
    }) {
        return Err(MailSyncHealthContractErrorV1::InvalidCursor);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(outcome: MailSyncOutcomeV1) -> MailSyncRunV1 {
        let failure = matches!(
            outcome,
            MailSyncOutcomeV1::Failed | MailSyncOutcomeV1::Interrupted
        );
        MailSyncRunV1 {
            operation_id: "sync-1".to_owned(),
            connection_id: "account-1".to_owned(),
            trigger: MailSyncTriggerV1::Manual,
            outcome,
            observed_messages: 3,
            started_at_unix_seconds: 10,
            completed_at_unix_seconds: (!matches!(outcome, MailSyncOutcomeV1::Running))
                .then_some(11),
            failure_code: failure.then_some(MailSyncFailureCodeV1::ProviderUnavailable),
            runtime_generation: 7,
            projection_revision: 2,
        }
    }

    #[test]
    fn query_bounds_and_ids_fail_closed() {
        assert_eq!(
            validate_sync_health_query(&MailSyncHealthQueryV1::ListRuns {
                connection_id: "account-1".to_owned(),
                cursor: None,
                limit: 200,
            }),
            Ok(())
        );
        assert_eq!(
            validate_sync_health_query(&MailSyncHealthQueryV1::ListRuns {
                connection_id: "account-1".to_owned(),
                cursor: None,
                limit: 0,
            }),
            Err(MailSyncHealthContractErrorV1::InvalidLimit)
        );
        assert_eq!(
            validate_sync_health_query(&MailSyncHealthQueryV1::GetRun {
                connection_id: "account-1".to_owned(),
                operation_id: "bad\noperation".to_owned(),
            }),
            Err(MailSyncHealthContractErrorV1::InvalidId)
        );
    }

    #[test]
    fn terminal_state_requires_exact_completion_and_failure_shape() {
        for outcome in [
            MailSyncOutcomeV1::Running,
            MailSyncOutcomeV1::Succeeded,
            MailSyncOutcomeV1::Failed,
            MailSyncOutcomeV1::Interrupted,
        ] {
            assert_eq!(validate_sync_run(&run(outcome)), Ok(()));
        }
        let mut invalid = run(MailSyncOutcomeV1::Succeeded);
        invalid.failure_code = Some(MailSyncFailureCodeV1::ProviderUnavailable);
        assert_eq!(
            validate_sync_run(&invalid),
            Err(MailSyncHealthContractErrorV1::InvalidRun)
        );
    }
}
