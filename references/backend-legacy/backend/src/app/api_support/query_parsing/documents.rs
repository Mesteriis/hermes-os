use url::form_urlencoded;

use crate::app::error::types::ApiError;

pub(crate) struct DocumentProcessingJobsQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_document_processing_jobs_query(
    raw_query: Option<&str>,
) -> Result<DocumentProcessingJobsQuery, ApiError> {
    let mut query = DocumentProcessingJobsQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidDocumentProcessingQuery("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn validate_non_empty_document_id(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidDocumentProcessingQuery(
            "document_id must not be empty",
        ));
    }

    Ok(normalized.to_owned())
}

pub(crate) fn validate_non_empty_document_processing_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidDocumentProcessingQuery(match field {
            "command_id" => "command_id must not be empty",
            "job_id" => "job_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}
