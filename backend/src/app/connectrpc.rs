mod communications;
mod signal_hub;

use axum::Router;
use axum::middleware;
use connectrpc::Router as ConnectRouter;
use sqlx::postgres::PgPool;

use crate::app::guard;
use crate::app::state::AppState;
use crate::platform::config::AppConfig;

pub(crate) fn protected_routes(
    pool: Option<PgPool>,
    config: AppConfig,
    api_secret: String,
) -> Router<AppState> {
    let connect_router = signal_hub::register(
        communications::register(ConnectRouter::new(), pool.clone(), config.clone()),
        pool,
        config,
    );
    Router::<AppState>::new()
        .fallback_service(connect_router.into_axum_router().into_service())
        .layer(middleware::from_fn_with_state(
            api_secret,
            guard::require_secret,
        ))
}
