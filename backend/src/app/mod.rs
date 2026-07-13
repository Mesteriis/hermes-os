pub(crate) mod api_support;
pub(crate) mod connectrpc;
pub(crate) mod error;
pub(crate) mod guard;
pub(crate) mod handlers;
pub(crate) mod provider_runtime_handlers;
pub(crate) mod router;
pub(crate) mod runtime;
pub(crate) mod runtime_lifecycle;
pub(crate) mod signal_hub_support;
pub(crate) mod state;
pub(crate) mod vault_reconciliation;

pub(crate) use error::{ApiError, AppError};
pub use router::{
    build_router, build_router_with_database, build_router_with_database_and_runtime, init_tracing,
    run,
};
pub(crate) use state::{AccountSetupState, AppState};
