use axum::Json;
use axum::extract::{Path, State};

use crate::app::{ApiError, AppState};
use crate::platform::maintenance::{
    MaintenanceActionRequest, MaintenanceActionResponse, MaintenanceError, MaintenanceOverview,
    build_maintenance_overview, run_maintenance_action,
};

pub(crate) async fn get_maintenance_overview(
    State(state): State<AppState>,
) -> Result<Json<MaintenanceOverview>, ApiError> {
    Ok(Json(
        build_maintenance_overview(&state.config, &state.database).await,
    ))
}

pub(crate) async fn post_maintenance_action(
    Path(action_id): Path<String>,
    Json(request): Json<MaintenanceActionRequest>,
) -> Result<Json<MaintenanceActionResponse>, ApiError> {
    run_maintenance_action(&action_id, request)
        .map(Json)
        .map_err(maintenance_error)
}

fn maintenance_error(error: MaintenanceError) -> ApiError {
    let public_message = error.public_message();
    match error {
        MaintenanceError::Io(source) => {
            tracing::error!(error = %source, "maintenance I/O operation failed");
            ApiError::FailedPrecondition(public_message)
        }
        MaintenanceError::CommandFailed => {
            tracing::error!("maintenance command failed");
            ApiError::FailedPrecondition(public_message)
        }
        MaintenanceError::UnknownAction
        | MaintenanceError::ConfirmationRequired(_)
        | MaintenanceError::Unsupported(_) => ApiError::FailedPrecondition(public_message),
    }
}
