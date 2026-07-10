mod ai;
mod audit_events;
mod calendar;
mod communications;
mod email_accounts;
mod knowledge;
mod maintenance;
mod messaging;
mod organizations;
mod personas;
mod public;
mod review;
mod settings;
mod signal_hub;
mod status_vault;
mod support;
mod tasks;

use support::*;

pub(super) fn protected_routes(api_secret: String) -> Router<AppState> {
    let routes = Router::<AppState>::new();
    let routes = status_vault::add_routes(routes);
    let routes = communications::add_routes(routes);
    let routes = knowledge::add_routes(routes);
    let routes = personas::add_routes(routes);
    let routes = calendar::add_routes(routes);
    let routes = organizations::add_routes(routes);
    let routes = tasks::add_routes(routes);
    let routes = review::add_routes(routes);
    let routes = settings::add_routes(routes);
    let routes = maintenance::add_routes(routes);
    let routes = signal_hub::add_routes(routes);
    let routes = ai::add_routes(routes);
    let routes = messaging::add_routes(routes);
    let routes = email_accounts::add_routes(routes);
    let routes = audit_events::add_routes(routes);

    routes.route_layer(middleware::from_fn_with_state(
        api_secret,
        guard::require_secret,
    ))
}

pub(super) fn public_routes() -> Router<AppState> {
    public::routes()
}
