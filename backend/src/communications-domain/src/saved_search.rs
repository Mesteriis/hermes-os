use crate::{CommunicationsSearchTokenErrorV1, normalize_search_query_v1};

pub const COMMUNICATIONS_SAVED_SEARCH_MAX_NAME_BYTES_V1: usize = 128;
pub const COMMUNICATIONS_SAVED_SEARCH_MAX_DESCRIPTION_BYTES_V1: usize = 512;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSavedSearchDraftV1 {
    pub saved_search_id: [u8; 16],
    pub name: String,
    pub description: Option<String>,
    pub account_id: Option<[u8; 16]>,
    pub normalized_tokens: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSavedSearchDraftErrorV1 {
    InvalidId,
    InvalidName,
    InvalidDescription,
    InvalidAccountId,
    InvalidQuery,
}

pub fn validate_saved_search_draft_v1(
    saved_search_id: &[u8],
    name: &str,
    description: Option<&str>,
    account_id: Option<&[u8]>,
    query: &str,
) -> Result<CommunicationsSavedSearchDraftV1, CommunicationsSavedSearchDraftErrorV1> {
    let saved_search_id = id16(saved_search_id)
        .filter(|value| value.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsSavedSearchDraftErrorV1::InvalidId)?;
    let name = normalize_display_text(name, COMMUNICATIONS_SAVED_SEARCH_MAX_NAME_BYTES_V1, false)
        .ok_or(CommunicationsSavedSearchDraftErrorV1::InvalidName)?;
    let description = description
        .map(|value| {
            normalize_display_text(
                value,
                COMMUNICATIONS_SAVED_SEARCH_MAX_DESCRIPTION_BYTES_V1,
                true,
            )
            .ok_or(CommunicationsSavedSearchDraftErrorV1::InvalidDescription)
        })
        .transpose()?
        .filter(|value| !value.is_empty());
    let account_id = account_id
        .map(|value| {
            id16(value)
                .filter(|candidate| candidate.iter().any(|byte| *byte != 0))
                .ok_or(CommunicationsSavedSearchDraftErrorV1::InvalidAccountId)
        })
        .transpose()?;
    let normalized_tokens = normalize_search_query_v1(query)
        .map_err(map_query_error)?
        .tokens;
    Ok(CommunicationsSavedSearchDraftV1 {
        saved_search_id,
        name,
        description,
        account_id,
        normalized_tokens,
    })
}

fn normalize_display_text(value: &str, max_bytes: usize, allow_empty: bool) -> Option<String> {
    let normalized = value.trim();
    if normalized.len() > max_bytes
        || (!allow_empty && normalized.is_empty())
        || normalized.chars().any(char::is_control)
    {
        return None;
    }
    Some(normalized.to_owned())
}

fn id16(value: &[u8]) -> Option<[u8; 16]> {
    value.try_into().ok()
}

const fn map_query_error(
    _: CommunicationsSearchTokenErrorV1,
) -> CommunicationsSavedSearchDraftErrorV1 {
    CommunicationsSavedSearchDraftErrorV1::InvalidQuery
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_normalizes_query_without_retaining_the_plaintext_expression() {
        let draft = validate_saved_search_draft_v1(
            &[7; 16],
            "  Important  ",
            Some("  Owner review  "),
            Some(&[9; 16]),
            "Alpha ALPHA beta",
        )
        .expect("valid saved search");

        assert_eq!(draft.name, "Important");
        assert_eq!(draft.description.as_deref(), Some("Owner review"));
        assert_eq!(draft.normalized_tokens, ["alpha", "beta"]);
        assert!(!format!("{draft:?}").contains("Alpha ALPHA beta"));
    }

    #[test]
    fn draft_rejects_zero_ids_controls_and_empty_queries() {
        assert_eq!(
            validate_saved_search_draft_v1(&[0; 16], "name", None, None, "query"),
            Err(CommunicationsSavedSearchDraftErrorV1::InvalidId),
        );
        assert_eq!(
            validate_saved_search_draft_v1(&[1; 16], "bad\nname", None, None, "query"),
            Err(CommunicationsSavedSearchDraftErrorV1::InvalidName),
        );
        assert_eq!(
            validate_saved_search_draft_v1(&[1; 16], "name", None, None, " "),
            Err(CommunicationsSavedSearchDraftErrorV1::InvalidQuery),
        );
    }
}
