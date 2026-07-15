use super::finance_analytics::models::{ImportantToggleResponse, PinToggleResponse};
use super::*;
use crate::domains::communications::command_service::CommunicationCommandService;

#[derive(Deserialize)]
pub(crate) struct BulkMessageActionRequest {
    action: String,
    message_ids: Vec<String>,
    label: Option<String>,
    snooze_until: Option<String>,
}

pub(crate) async fn post_v1_messages_bulk_action(
    State(state): State<AppState>,
    Json(request): Json<BulkMessageActionRequest>,
) -> Result<Json<crate::domains::communications::bulk_actions::BulkMessageActionOutcome>, ApiError>
{
    let action = parse_bulk_message_action(&request)?;
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::communications::bulk_actions::BulkMessageActionStore,
    >(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    );
    Ok(Json(store.apply(request.message_ids, action).await?))
}

fn parse_bulk_message_action(
    request: &BulkMessageActionRequest,
) -> Result<crate::domains::communications::bulk_actions::BulkMessageAction, ApiError> {
    let action = match request.action.trim() {
        "mark_read" => crate::domains::communications::bulk_actions::BulkMessageAction::MarkRead,
        "mark_unread" => {
            crate::domains::communications::bulk_actions::BulkMessageAction::MarkUnread
        }
        "archive" => crate::domains::communications::bulk_actions::BulkMessageAction::Archive,
        "trash" => crate::domains::communications::bulk_actions::BulkMessageAction::Trash,
        "restore" => crate::domains::communications::bulk_actions::BulkMessageAction::Restore,
        "pin" => crate::domains::communications::bulk_actions::BulkMessageAction::Pin,
        "unpin" => crate::domains::communications::bulk_actions::BulkMessageAction::Unpin,
        "important" => crate::domains::communications::bulk_actions::BulkMessageAction::Important,
        "not_important" => {
            crate::domains::communications::bulk_actions::BulkMessageAction::NotImportant
        }
        "star" => crate::domains::communications::bulk_actions::BulkMessageAction::Star,
        "unstar" => crate::domains::communications::bulk_actions::BulkMessageAction::Unstar,
        "add_label" => {
            let label = request
                .label
                .as_deref()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "label is required for add_label",
                ))?;
            crate::domains::communications::bulk_actions::BulkMessageAction::AddLabel(
                label.to_owned(),
            )
        }
        "remove_label" => {
            let label = request
                .label
                .as_deref()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "label is required for remove_label",
                ))?;
            crate::domains::communications::bulk_actions::BulkMessageAction::RemoveLabel(
                label.to_owned(),
            )
        }
        "snooze" => {
            let until = request
                .snooze_until
                .as_deref()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "snooze_until is required for snooze",
                ))?
                .parse()
                .map_err(|_| ApiError::InvalidCommunicationQuery("invalid snooze_until"))?;
            crate::domains::communications::bulk_actions::BulkMessageAction::Snooze(until)
        }
        _ => {
            return Err(ApiError::InvalidCommunicationQuery(
                "invalid bulk message action",
            ));
        }
    };

    Ok(action)
}

pub(crate) async fn post_v1_message_pin(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pinned = CommunicationCommandService::new(pool)
        .toggle_message_pin(&message_id)
        .await?;
    Ok(Json(PinToggleResponse { message_id, pinned }))
}

pub(crate) async fn post_v1_message_important(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<ImportantToggleResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let important = CommunicationCommandService::new(pool)
        .toggle_message_important(&message_id)
        .await?;
    Ok(Json(ImportantToggleResponse {
        message_id,
        important,
    }))
}

#[derive(Deserialize)]
pub(crate) struct SnoozeRequest {
    pub(super) until: String,
}

pub(crate) async fn post_v1_message_snooze(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<SnoozeRequest>,
) -> Result<Json<Value>, ApiError> {
    let until: DateTime<Utc> = req
        .until
        .parse()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid datetime"))?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .snooze_message(&message_id, until)
        .await?;
    Ok(Json(serde_json::json!({"snoozed": true})))
}

pub(crate) async fn post_v1_message_mute(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let muted = CommunicationCommandService::new(pool)
        .toggle_message_mute(&message_id)
        .await?;
    Ok(Json(PinToggleResponse {
        message_id,
        pinned: muted,
    }))
}

#[derive(Deserialize)]
pub(crate) struct LabelRequest {
    pub(super) label: String,
}

pub(crate) async fn post_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .add_message_label(&message_id, &req.label)
        .await?;
    Ok(Json(serde_json::json!({"labeled": true})))
}

pub(crate) async fn delete_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .remove_message_label(&message_id, &req.label)
        .await?;
    Ok(Json(serde_json::json!({"removed": true})))
}

#[derive(Deserialize)]
pub(crate) struct SubscriptionsQuery {
    pub(super) account_id: Option<String>,
    pub(super) limit: Option<i64>,
    pub(super) cursor: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SubscriptionsResponse {
    pub(super) items: Vec<crate::domains::communications::subscriptions::SubscriptionSource>,
    pub(super) next_cursor: Option<String>,
    pub(super) has_more: bool,
}

pub(crate) async fn get_v1_subscriptions(
    State(state): State<AppState>,
    Query(query): Query<SubscriptionsQuery>,
) -> Result<Json<SubscriptionsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::communications::subscriptions::SubscriptionStore,
    >(pool);
    let page = store
        .detect_subscriptions_page(
            query.account_id.as_deref(),
            query.limit.unwrap_or(50),
            query.cursor.as_deref(),
        )
        .await?;
    Ok(Json(SubscriptionsResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

#[derive(Deserialize)]
pub(crate) struct DupQuery {
    pub(super) limit: Option<i64>,
}

pub(crate) async fn get_v1_attachment_duplicates(
    State(state): State<AppState>,
    Query(query): Query<DupQuery>,
) -> Result<Json<Vec<crate::domains::communications::attachment_dedup::DuplicateGroup>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::communications::attachment_dedup::AttachmentDedupStore,
    >(pool);
    let dups = store.find_duplicates(query.limit.unwrap_or(20)).await?;
    Ok(Json(dups))
}

#[derive(Deserialize)]
pub(crate) struct LegalDocQuery {
    pub(super) document_type: Option<String>,
    pub(super) status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LegalDocListResponse {
    pub(super) items: Vec<crate::domains::communications::legal::LegalDocument>,
}
