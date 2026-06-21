use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{CommunicationFolder, CommunicationFolderError, FolderMessage};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct FolderListCursor {
    pub(super) sort_order: i32,
    pub(super) name_lower: String,
    pub(super) folder_id: String,
}

pub(super) fn encode_folder_list_cursor(
    folder: &CommunicationFolder,
) -> Result<String, CommunicationFolderError> {
    let cursor = FolderListCursor {
        sort_order: folder.sort_order,
        name_lower: folder.name.to_lowercase(),
        folder_id: folder.folder_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| CommunicationFolderError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(super) fn decode_folder_list_cursor(
    cursor: &str,
) -> Result<FolderListCursor, CommunicationFolderError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationFolderError::InvalidCursor)?;
    let cursor: FolderListCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationFolderError::InvalidCursor)?;
    if cursor.name_lower.trim().is_empty() || cursor.folder_id.trim().is_empty() {
        return Err(CommunicationFolderError::InvalidCursor);
    }

    Ok(cursor)
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct FolderMessageCursor {
    pub(super) added_at: DateTime<Utc>,
    pub(super) message_id: String,
}

pub(super) fn encode_folder_message_cursor(
    message: &FolderMessage,
) -> Result<String, CommunicationFolderError> {
    let cursor = FolderMessageCursor {
        added_at: message.added_at,
        message_id: message.message_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| CommunicationFolderError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(super) fn decode_folder_message_cursor(
    cursor: &str,
) -> Result<FolderMessageCursor, CommunicationFolderError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationFolderError::InvalidCursor)?;
    let cursor: FolderMessageCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationFolderError::InvalidCursor)?;
    if cursor.message_id.trim().is_empty() {
        return Err(CommunicationFolderError::InvalidCursor);
    }

    Ok(cursor)
}
