use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/status", get(get_v1_status))
        .route("/api/v1/vault/status", get(get_v1_vault_status))
        .route(
            "/api/v1/vault/collect-entropy",
            post(post_v1_vault_collect_entropy),
        )
        .route("/api/v1/vault/create", post(post_v1_vault_create))
        .route("/api/v1/vault/unlock", post(post_v1_vault_unlock))
        .route(
            "/api/v1/vault/recovery/export",
            post(post_v1_vault_recovery_export),
        )
        .route(
            "/api/v1/vault/recovery/import",
            post(post_v1_vault_recovery_import),
        )
}
