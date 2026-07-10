use super::support::*;
// ── Persona Investigator ───────────────────────────────────────────────────

pub(crate) async fn post_persona_investigate(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let (dossier, snapshot) = PersonaInvestigator::new(pool)
        .assemble_cache_and_record_refresh(
            &persona_id,
            "investigate",
            "personas_api.post_persona_investigate",
            "post_persona_investigate",
            format!("persona://{persona_id}/investigate"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dossier_snapshot_response(&dossier, &snapshot)))
}

pub(crate) async fn get_persona_dossier(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let (dossier, snapshot) = PersonaInvestigator::new(pool)
        .assemble_cache_and_record_refresh(
            &persona_id,
            "dossier_read_refresh",
            "personas_api.get_persona_dossier",
            "get_persona_dossier",
            format!("persona://{persona_id}/dossier"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dossier_snapshot_response(&dossier, &snapshot)))
}

#[derive(Deserialize)]
pub(crate) struct DossierReviewRequest {
    review_state: String,
}

pub(crate) async fn put_persona_dossier_review(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<DossierReviewRequest>,
) -> Result<Json<Value>, ApiError> {
    let review_state = DossierReviewState::parse(&req.review_state).map_err(ApiError::from)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let snapshot = crate::domains::personas::service::PersonaCommandService::new(pool)
        .review_dossier_manual(&persona_id, review_state)
        .await?;
    Ok(Json(dossier_snapshot_only_response(&snapshot)))
}

fn dossier_snapshot_response(dossier: &PersonaDossier, snapshot: &DossierSnapshot) -> Value {
    let mut value = serde_json::to_value(dossier).unwrap_or_default();
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "dossier_snapshot_id".to_owned(),
            json!(snapshot.dossier_snapshot_id),
        );
        object.insert("persona_id".to_owned(), json!(snapshot.persona_id));
        object.insert("review_state".to_owned(), json!(snapshot.review_state));
        object.insert("reviewed_by".to_owned(), json!(snapshot.reviewed_by));
        object.insert("reviewed_at".to_owned(), json!(snapshot.reviewed_at));
    }
    value
}

fn dossier_snapshot_only_response(snapshot: &DossierSnapshot) -> Value {
    json!({
        "dossier_snapshot_id": snapshot.dossier_snapshot_id,
        "persona_id": snapshot.persona_id,
        "review_state": snapshot.review_state,
        "reviewed_by": snapshot.reviewed_by,
        "reviewed_at": snapshot.reviewed_at,
        "generated_at": snapshot.generated_at,
        "updated_at": snapshot.updated_at
    })
}

pub(crate) async fn get_persona_meeting_prep(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let prep = PersonaInvestigator::new(pool)
        .meeting_prep(&persona_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&prep).unwrap_or_default()))
}
