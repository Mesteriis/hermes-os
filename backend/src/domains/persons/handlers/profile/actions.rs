use super::super::support::*;

pub(crate) async fn post_person_fingerprint(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let msg_store = crate::domains::mail::messages::MessageProjectionStore::new(pool.clone());
    let messages = msg_store.recent_messages(50).await?;
    let person_msgs = messages
        .into_iter()
        .filter(|m| {
            m.message.sender.contains(&person_id)
                || m.message.recipients.iter().any(|r| r.contains(&person_id))
        })
        .map(|m| crate::domains::persons::intelligence::PersonMessage {
            subject: m.message.subject,
            body_text: m.message.body_text,
            occurred_at: m.message.occurred_at,
        })
        .collect::<Vec<_>>();
    let fp =
        crate::domains::persons::intelligence::PersonIntelligenceService::heuristic_fingerprint(
            &person_msgs,
        );
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    store.enrich_person(&person_id, &fp).await?;
    Ok(Json(json!({"enriched": true, "fingerprint": fp})))
}

pub(crate) async fn post_person_favorite(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let fav = store.toggle_favorite(&person_id).await?;
    Ok(Json(json!({"is_favorite": fav})))
}

#[derive(Deserialize)]
pub(crate) struct PersonNotesRequest {
    notes: String,
}

pub(crate) async fn put_person_notes(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<PersonNotesRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    store.set_notes(&person_id, &req.notes).await?;
    Ok(Json(json!({"saved": true})))
}
