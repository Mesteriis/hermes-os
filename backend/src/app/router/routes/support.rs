pub(super) use axum::routing::{delete, get, patch, post, put};
pub(super) use axum::{Router, middleware};

pub(super) use crate::ai::api::*;
pub(super) use crate::app::AppState;
pub(super) use crate::app::api_support::{
    automation_calls::*,
    communications::*,
    ensure_fixture_routes_enabled,
    messaging_integrations::*,
    platform_dtos::*,
    query_parsing::{communication::*, documents::*, graph::*, personas::*, projects::*, tasks::*},
    review_commands::*,
    review_lists::*,
    stores::{ai_runtime::*, domain_stores::*, integration_stores::*, settings_vault::*},
    telegram_capabilities::*,
    whatsapp_capabilities::*,
};
pub(super) use crate::app::guard;
pub(super) use crate::app::handlers::automation::*;
pub(super) use crate::app::handlers::calendar::*;
pub(super) use crate::app::handlers::calls::*;
pub(super) use crate::app::handlers::communications::*;
pub(super) use crate::app::handlers::consistency::*;
pub(super) use crate::app::handlers::decisions::*;
pub(super) use crate::app::handlers::documents::*;
pub(super) use crate::app::handlers::events::*;
pub(super) use crate::app::handlers::graph::*;
pub(super) use crate::app::handlers::maintenance::*;
pub(super) use crate::app::handlers::obligations::*;
pub(super) use crate::app::handlers::organizations::*;
pub(super) use crate::app::handlers::personas::*;
pub(super) use crate::app::handlers::projects::*;
pub(super) use crate::app::handlers::relationships::*;
pub(super) use crate::app::handlers::review::*;
pub(super) use crate::app::handlers::settings::*;
pub(super) use crate::app::handlers::signal_hub::*;
pub(super) use crate::app::handlers::tasks::*;
pub(super) use crate::app::provider_runtime_handlers::telegram::*;
pub(super) use crate::app::provider_runtime_handlers::yandex_telemost::*;
pub(super) use crate::app::provider_runtime_handlers::zoom::*;
pub(super) use crate::app::provider_runtime_handlers::zulip::*;
