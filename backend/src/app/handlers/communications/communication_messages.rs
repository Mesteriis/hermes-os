use super::*;
use crate::domains::communications::messages::ProjectedMessagePageQuery;

pub(crate) async fn get_v1_communication_messages(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<CommunicationMessagesResponse>, ApiError> {
    let query = parse_communication_messages_query(raw_query.as_deref())?;
    let limit = query.limit.unwrap_or(5000).clamp(1, 5000);
    let workflow_state = query
        .workflow_state
        .as_deref()
        .map(str::parse::<WorkflowState>)
        .transpose()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid workflow state value"))?;
    let local_state = query
        .local_state
        .as_deref()
        .unwrap_or("active")
        .parse::<LocalMessageState>()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid local_state value"))?;
    let page = message_store(&state)?
        .list_messages_page(ProjectedMessagePageQuery {
            account_id: query.account_id.as_deref(),
            workflow_state,
            channel_kind: query.channel_kind.as_deref(),
            query: query.q.as_deref(),
            match_mode: query.match_mode,
            search: query.search.clone(),
            local_state,
            cursor: query.cursor.as_deref(),
            limit,
        })
        .await?;
    let items = page
        .items
        .into_iter()
        .map(CommunicationMessageSummaryResponse::from)
        .collect();

    Ok(Json(CommunicationMessagesResponse {
        items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn get_v1_communication_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<CommunicationMessageDetailResponse>, ApiError> {
    let Some(message) = message_store(&state)?.message(&message_id).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let rich_detail = rich_email_message_detail_for_message(&state, &message).await?;
    let message_metadata = message_metadata_with_raw_headers(
        &message.message_metadata,
        rich_detail.headers.as_slice(),
    );
    let attachments = mail_storage_store(&state)?
        .attachments_for_message(&message.message_id)
        .await?
        .into_iter()
        .map(CommunicationAttachmentResponse::from)
        .collect();

    Ok(Json(CommunicationMessageDetailResponse {
        message: CommunicationMessageDetailItem::from_message_with_metadata(
            message,
            rich_detail.body_html,
            message_metadata,
        ),
        attachments,
    }))
}

pub(super) async fn rich_body_html_for_message(
    state: &AppState,
    message: &ProjectedMessage,
) -> Result<Option<String>, ApiError> {
    Ok(rich_email_message_detail_for_message(state, message)
        .await?
        .body_html)
}

#[derive(Default)]
pub(super) struct RichCommunicationMessageDetail {
    pub(super) body_html: Option<String>,
    pub(super) headers: Vec<(String, String)>,
}

async fn rich_email_message_detail_for_message(
    state: &AppState,
    message: &ProjectedMessage,
) -> Result<RichCommunicationMessageDetail, ApiError> {
    let Some(raw) = communication_ingestion_store(state)?
        .raw_record(&message.raw_record_id)
        .await?
    else {
        return Ok(RichCommunicationMessageDetail::default());
    };
    if raw.record_kind != "email_message" {
        return Ok(RichCommunicationMessageDetail::default());
    }
    if raw
        .payload
        .get("raw_blob_storage_kind")
        .and_then(Value::as_str)
        != Some("local_fs")
    {
        return Ok(RichCommunicationMessageDetail::default());
    }
    if raw
        .payload
        .get("raw_blob_storage_path")
        .and_then(Value::as_str)
        .is_none()
    {
        return Ok(RichCommunicationMessageDetail::default());
    }

    let blob_store = LocalMailBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    match parse_raw_email_message_from_blob(&blob_store, &raw).await {
        Ok(parsed) => Ok(RichCommunicationMessageDetail {
            body_html: parsed.body_html.filter(|value| !value.trim().is_empty()),
            headers: parsed.headers,
        }),
        Err(error) => {
            tracing::warn!(
                error = %error,
                message_id = %message.message_id,
                raw_record_id = %message.raw_record_id,
                "mail detail rich html extraction failed; falling back to projected body_text"
            );
            Ok(RichCommunicationMessageDetail::default())
        }
    }
}

fn message_metadata_with_raw_headers(
    message_metadata: &Value,
    headers: &[(String, String)],
) -> Value {
    let mut metadata = message_metadata.as_object().cloned().unwrap_or_default();
    if !headers.is_empty() && !metadata.contains_key("headers") {
        metadata.insert(
            "headers".to_owned(),
            Value::Array(
                headers
                    .iter()
                    .map(|(name, value)| json!({ "name": name, "value": value }))
                    .collect(),
            ),
        );
    }
    Value::Object(metadata)
}
