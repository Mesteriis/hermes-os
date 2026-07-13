use super::super::support::*;

pub(crate) async fn post_persona_fingerprint(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let messages = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::communications::messages::MessageProjectionStore,
    >(pool.clone())
    .recent_messages(50)
    .await?;
    let person_messages = messages
        .into_iter()
        .filter(|message| {
            message.message.sender.contains(&persona_id)
                || message
                    .message
                    .recipients
                    .iter()
                    .any(|recipient| recipient.contains(&persona_id))
        })
        .map(
            |message| crate::domains::personas::intelligence::PersonaMessage {
                subject: message.message.subject,
                body_text: message.message.body_text,
                occurred_at: message.message.occurred_at,
            },
        )
        .collect::<Vec<_>>();
    Ok(Json(
        crate::domains::personas::command_service::PersonaCommandService::new(pool)
            .fingerprint_persona_manual(&persona_id, &person_messages)
            .await?,
    ))
}

pub(crate) async fn post_persona_favorite(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let fav = crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .toggle_favorite_manual(&persona_id)
        .await?;
    Ok(Json(json!({"is_favorite": fav})))
}

#[derive(Deserialize)]
pub(crate) struct PersonaAddressBookMembershipRequest {
    is_address_book: bool,
}

pub(crate) async fn put_persona_address_book_membership(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<PersonaAddressBookMembershipRequest>,
) -> Result<Json<crate::domains::personas::api::Persona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let person = crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .set_persona_address_book_membership_manual(&persona_id, req.is_address_book)
        .await?;
    Ok(Json(person))
}

#[derive(Deserialize)]
pub(crate) struct PersonaNotesRequest {
    notes: String,
}

pub(crate) async fn put_persona_notes(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<PersonaNotesRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .set_notes_manual(&persona_id, &req.notes)
        .await?;
    Ok(Json(json!({"saved": true})))
}
