use thiserror::Error;

use crate::app::error::types::ApiError;

#[derive(Debug, Error)]
pub(super) enum RemoteImageFetchError {
    #[error("remote image host is unavailable")]
    MissingHost,
    #[error("remote image has no public DNS address")]
    NoPublicAddress,
    #[error("remote image client failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("remote image returned non-success status")]
    NonSuccessStatus,
    #[error("remote image content type is not image")]
    NotImage,
    #[error("remote image exceeds size limit")]
    TooLarge,
    #[error("remote image response header is invalid")]
    InvalidHeader,
}

pub(super) fn remote_image_fetch_api_error(error: RemoteImageFetchError) -> ApiError {
    match error {
        RemoteImageFetchError::TooLarge => {
            ApiError::InvalidCommunicationQuery("remote image exceeds size limit")
        }
        RemoteImageFetchError::NotImage => {
            ApiError::InvalidCommunicationQuery("remote asset is not an image")
        }
        RemoteImageFetchError::NoPublicAddress => {
            ApiError::InvalidCommunicationQuery("remote image host has no public address")
        }
        _ => ApiError::InvalidCommunicationQuery("remote image unavailable"),
    }
}
