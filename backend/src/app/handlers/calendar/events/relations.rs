use super::super::*;

#[derive(Serialize)]
pub(crate) struct EventRelationsResponse {
    items: Vec<crate::domains::calendar::core::EventRelation>,
}

pub(crate) async fn get_event_relations(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRelationsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventRelationStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRelationsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationRequest {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

pub(crate) async fn post_event_relation(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewRelationRequest>,
) -> Result<Json<crate::domains::calendar::core::EventRelation>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let relation = CalendarCommandService::new(pool)
        .link_event_relation_manual(
            &event_id,
            &req.entity_type,
            &req.entity_id,
            &req.relation_type,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(relation))
}
