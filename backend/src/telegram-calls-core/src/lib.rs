pub const PACKAGE: &str = "hermes-telegram-calls-core";
pub const MAX_CALL_ID_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramProviderCallState {
    Pending,
    ExchangingKeys,
    MediaReady,
    HangingUp,
    Discarded,
    Error,
}

impl TelegramProviderCallState {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::ExchangingKeys => "exchanging_keys",
            Self::MediaReady => "media_ready",
            Self::HangingUp => "hanging_up",
            Self::Discarded => "discarded",
            Self::Error => "error",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "exchanging_keys" => Some(Self::ExchangingKeys),
            "media_ready" => Some(Self::MediaReady),
            "hanging_up" => Some(Self::HangingUp),
            "discarded" => Some(Self::Discarded),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Discarded | Self::Error)
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Pending => 1,
            Self::ExchangingKeys => 2,
            Self::MediaReady => 3,
            Self::HangingUp => 4,
            Self::Discarded | Self::Error => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallDiscardReason {
    Empty,
    Missed,
    Declined,
    Disconnected,
    HungUp,
}

impl TelegramCallDiscardReason {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Missed => "missed",
            Self::Declined => "declined",
            Self::Disconnected => "disconnected",
            Self::HungUp => "hung_up",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "missed" => Some(Self::Missed),
            "declined" => Some(Self::Declined),
            "disconnected" => Some(Self::Disconnected),
            "hung_up" => Some(Self::HungUp),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallFailureCategory {
    Network,
    NotAvailable,
    Permission,
    Protocol,
    Unknown,
}

impl TelegramCallFailureCategory {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::NotAvailable => "not_available",
            Self::Permission => "permission",
            Self::Protocol => "protocol",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "network" => Some(Self::Network),
            "not_available" => Some(Self::NotAvailable),
            "permission" => Some(Self::Permission),
            "protocol" => Some(Self::Protocol),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramProviderCallUpdate {
    pub account_id: String,
    pub runtime_generation: u64,
    pub tdlib_call_id: i32,
    pub provider_call_unique_id: Option<i64>,
    pub provider_user_id: String,
    pub direction: TelegramCallDirection,
    pub state: TelegramProviderCallState,
    pub pending_created: bool,
    pub pending_received: bool,
    pub discard_reason: Option<TelegramCallDiscardReason>,
    pub failure_category: Option<TelegramCallFailureCategory>,
    pub observed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallSession {
    pub call_session_id: String,
    pub account_id: String,
    pub runtime_generation: u64,
    pub tdlib_call_id: i32,
    pub provider_call_unique_id: Option<i64>,
    pub provider_user_id: String,
    pub direction: TelegramCallDirection,
    pub state: TelegramProviderCallState,
    pub pending_created: bool,
    pub pending_received: bool,
    pub discard_reason: Option<TelegramCallDiscardReason>,
    pub failure_category: Option<TelegramCallFailureCategory>,
    pub revision: u64,
    pub created_at_unix_seconds: u64,
    pub updated_at_unix_seconds: u64,
    pub ended_at_unix_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectedCallUpdate {
    pub session: TelegramCallSession,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallProjectionError {
    InvalidRequest(&'static str),
    IdentityConflict,
    StateRegression,
    TerminalConflict,
}

pub fn project_provider_call_update(
    existing: Option<&TelegramCallSession>,
    new_call_session_id: &str,
    update: &TelegramProviderCallUpdate,
) -> Result<ProjectedCallUpdate, TelegramCallProjectionError> {
    validate_update(new_call_session_id, update)?;

    let Some(current) = existing else {
        return Ok(ProjectedCallUpdate {
            session: TelegramCallSession {
                call_session_id: new_call_session_id.to_owned(),
                account_id: update.account_id.clone(),
                runtime_generation: update.runtime_generation,
                tdlib_call_id: update.tdlib_call_id,
                provider_call_unique_id: update.provider_call_unique_id,
                provider_user_id: update.provider_user_id.clone(),
                direction: update.direction,
                state: update.state,
                pending_created: update.pending_created,
                pending_received: update.pending_received,
                discard_reason: update.discard_reason,
                failure_category: update.failure_category,
                revision: 1,
                created_at_unix_seconds: update.observed_at_unix_seconds,
                updated_at_unix_seconds: update.observed_at_unix_seconds,
                ended_at_unix_seconds: update
                    .state
                    .is_terminal()
                    .then_some(update.observed_at_unix_seconds),
            },
            changed: true,
        });
    };

    validate_identity(current, update)?;
    if update.state.rank() < current.state.rank() {
        return Ok(ProjectedCallUpdate {
            session: current.clone(),
            changed: false,
        });
    }
    validate_transition(current, update)?;

    let provider_call_unique_id = current
        .provider_call_unique_id
        .or(update.provider_call_unique_id);
    let changed = provider_call_unique_id != current.provider_call_unique_id
        || update.state != current.state
        || update.pending_created != current.pending_created
        || update.pending_received != current.pending_received
        || update.discard_reason != current.discard_reason
        || update.failure_category != current.failure_category;

    if !changed {
        return Ok(ProjectedCallUpdate {
            session: current.clone(),
            changed: false,
        });
    }

    Ok(ProjectedCallUpdate {
        session: TelegramCallSession {
            call_session_id: current.call_session_id.clone(),
            account_id: current.account_id.clone(),
            runtime_generation: current.runtime_generation,
            tdlib_call_id: current.tdlib_call_id,
            provider_call_unique_id,
            provider_user_id: current.provider_user_id.clone(),
            direction: current.direction,
            state: update.state,
            pending_created: update.pending_created,
            pending_received: update.pending_received,
            discard_reason: update.discard_reason,
            failure_category: update.failure_category,
            revision: current.revision.saturating_add(1),
            created_at_unix_seconds: current.created_at_unix_seconds,
            updated_at_unix_seconds: update.observed_at_unix_seconds,
            ended_at_unix_seconds: if update.state.is_terminal() {
                current
                    .ended_at_unix_seconds
                    .or(Some(update.observed_at_unix_seconds))
            } else {
                None
            },
        },
        changed: true,
    })
}

fn validate_update(
    new_call_session_id: &str,
    update: &TelegramProviderCallUpdate,
) -> Result<(), TelegramCallProjectionError> {
    validate_id("call_session_id", new_call_session_id)?;
    validate_id("account_id", &update.account_id)?;
    validate_id("provider_user_id", &update.provider_user_id)?;
    if update.runtime_generation == 0 {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "runtime_generation",
        ));
    }
    if update.tdlib_call_id <= 0 {
        return Err(TelegramCallProjectionError::InvalidRequest("tdlib_call_id"));
    }
    if update
        .provider_call_unique_id
        .is_some_and(|value| value <= 0)
    {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "provider_call_unique_id",
        ));
    }
    if update.observed_at_unix_seconds == 0 {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "observed_at_unix_seconds",
        ));
    }
    if update.state != TelegramProviderCallState::Pending
        && (update.pending_created || update.pending_received)
    {
        return Err(TelegramCallProjectionError::InvalidRequest("pending_state"));
    }
    if (update.state == TelegramProviderCallState::Discarded) != update.discard_reason.is_some() {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "discard_reason",
        ));
    }
    if (update.state == TelegramProviderCallState::Error) != update.failure_category.is_some() {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "failure_category",
        ));
    }
    Ok(())
}

fn validate_identity(
    current: &TelegramCallSession,
    update: &TelegramProviderCallUpdate,
) -> Result<(), TelegramCallProjectionError> {
    let same_persistent_call = matches!(
        (
            current.provider_call_unique_id,
            update.provider_call_unique_id
        ),
        (Some(current_id), Some(update_id)) if current_id == update_id
    );
    if current.account_id != update.account_id
        || current.provider_user_id != update.provider_user_id
        || current.direction != update.direction
        || (!same_persistent_call
            && (current.runtime_generation != update.runtime_generation
                || current.tdlib_call_id != update.tdlib_call_id))
    {
        return Err(TelegramCallProjectionError::IdentityConflict);
    }
    if let (Some(current_id), Some(update_id)) = (
        current.provider_call_unique_id,
        update.provider_call_unique_id,
    ) && current_id != update_id
    {
        return Err(TelegramCallProjectionError::IdentityConflict);
    }
    Ok(())
}

fn validate_transition(
    current: &TelegramCallSession,
    update: &TelegramProviderCallUpdate,
) -> Result<(), TelegramCallProjectionError> {
    if current.state.is_terminal() {
        if current.state != update.state
            || current.discard_reason != update.discard_reason
            || current.failure_category != update.failure_category
        {
            return Err(TelegramCallProjectionError::TerminalConflict);
        }
        return Ok(());
    }
    if update.observed_at_unix_seconds < current.updated_at_unix_seconds {
        return Err(TelegramCallProjectionError::StateRegression);
    }
    Ok(())
}

fn validate_id(field: &'static str, value: &str) -> Result<(), TelegramCallProjectionError> {
    if value.trim().is_empty() || value.len() > MAX_CALL_ID_BYTES {
        return Err(TelegramCallProjectionError::InvalidRequest(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(state: TelegramProviderCallState, observed_at: u64) -> TelegramProviderCallUpdate {
        TelegramProviderCallUpdate {
            account_id: "account-1".to_owned(),
            runtime_generation: 7,
            tdlib_call_id: 41,
            provider_call_unique_id: None,
            provider_user_id: "provider-user-9".to_owned(),
            direction: TelegramCallDirection::Incoming,
            state,
            pending_created: state == TelegramProviderCallState::Pending,
            pending_received: false,
            discard_reason: None,
            failure_category: None,
            observed_at_unix_seconds: observed_at,
        }
    }

    #[test]
    fn binds_persistent_provider_identity_without_replacing_local_session() {
        let first = project_provider_call_update(
            None,
            "call-session-1",
            &update(TelegramProviderCallState::Pending, 10),
        )
        .expect("first update");
        let mut bound = update(TelegramProviderCallState::Pending, 11);
        bound.provider_call_unique_id = Some(9001);
        let projected =
            project_provider_call_update(Some(&first.session), "ignored-session", &bound)
                .expect("identity binding");

        assert_eq!(projected.session.call_session_id, "call-session-1");
        assert_eq!(projected.session.provider_call_unique_id, Some(9001));
        assert_eq!(projected.session.revision, 2);
    }

    #[test]
    fn duplicate_update_is_replayed_without_revision_or_timestamp_drift() {
        let first = project_provider_call_update(
            None,
            "call-session-1",
            &update(TelegramProviderCallState::Pending, 10),
        )
        .expect("first update");
        let replay = project_provider_call_update(
            Some(&first.session),
            "ignored-session",
            &update(TelegramProviderCallState::Pending, 10),
        )
        .expect("duplicate update");

        assert!(!replay.changed);
        assert_eq!(replay.session, first.session);
    }

    #[test]
    fn persistent_call_identity_survives_runtime_generation_change() {
        let mut first_update = update(TelegramProviderCallState::Discarded, 20);
        first_update.provider_call_unique_id = Some(5001);
        first_update.discard_reason = Some(TelegramCallDiscardReason::Missed);
        let first = project_provider_call_update(None, "call-session-1", &first_update)
            .expect("first call");

        let mut replay_update = first_update.clone();
        replay_update.runtime_generation += 1;
        replay_update.tdlib_call_id += 10;
        let replay =
            project_provider_call_update(Some(&first.session), "ignored-session", &replay_update)
                .expect("cross-generation replay");

        assert!(!replay.changed);
        assert_eq!(replay.session, first.session);
    }

    #[test]
    fn terminal_state_is_immutable_and_rejects_provider_identity_conflict() {
        let mut discarded = update(TelegramProviderCallState::Discarded, 12);
        discarded.discard_reason = Some(TelegramCallDiscardReason::Missed);
        discarded.pending_created = false;
        let terminal =
            project_provider_call_update(None, "call-session-1", &discarded).expect("terminal");
        let mut conflict = discarded.clone();
        conflict.provider_call_unique_id = Some(12);
        let bound =
            project_provider_call_update(Some(&terminal.session), "ignored-session", &conflict)
                .expect("late persistent identity");
        conflict.provider_call_unique_id = Some(13);

        assert_eq!(
            project_provider_call_update(Some(&bound.session), "ignored-session", &conflict),
            Err(TelegramCallProjectionError::IdentityConflict)
        );
    }

    #[test]
    fn stale_state_replay_is_ignored_and_untyped_terminal_details_fail_closed() {
        let ready = project_provider_call_update(
            None,
            "call-session-1",
            &update(TelegramProviderCallState::MediaReady, 12),
        )
        .expect("ready");

        let replay = project_provider_call_update(
            Some(&ready.session),
            "ignored-session",
            &update(TelegramProviderCallState::Pending, 13),
        )
        .expect("stale replay");
        assert!(!replay.changed);
        assert_eq!(replay.session, ready.session);
        assert_eq!(
            project_provider_call_update(
                None,
                "call-session-2",
                &update(TelegramProviderCallState::Discarded, 14),
            ),
            Err(TelegramCallProjectionError::InvalidRequest(
                "discard_reason"
            ))
        );
    }
}
