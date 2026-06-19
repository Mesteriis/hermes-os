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
    let messages = crate::domains::mail::messages::MessageProjectionStore::new(pool.clone())
        .recent_messages(50)
        .await?;
    let person_messages = messages
        .into_iter()
        .filter(|message| {
            message.message.sender.contains(&person_id)
                || message
                    .message
                    .recipients
                    .iter()
                    .any(|recipient| recipient.contains(&person_id))
        })
        .map(
            |message| crate::domains::persons::intelligence::PersonMessage {
                subject: message.message.subject,
                body_text: message.message.body_text,
                occurred_at: message.message.occurred_at,
            },
        )
        .collect::<Vec<_>>();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .fingerprint_person_manual(&person_id, &person_messages)
            .await?,
    ))
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
    let fav = crate::domains::persons::service::PersonCommandService::new(pool)
        .toggle_favorite_manual(&person_id)
        .await?;
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
    crate::domains::persons::service::PersonCommandService::new(pool)
        .set_notes_manual(&person_id, &req.notes)
        .await?;
    Ok(Json(json!({"saved": true})))
}
