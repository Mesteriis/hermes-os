use chrono::{DateTime, Utc};

use super::super::state::{TelegramRuntimeActorState, TelegramRuntimeState};

pub(in crate::integrations::telegram::runtime::manager) fn running_actor_state(
    updated_at: DateTime<Utc>,
) -> TelegramRuntimeActorState {
    TelegramRuntimeActorState {
        status: TelegramRuntimeState::Running,
        last_error: None,
        updated_at,
    }
}
