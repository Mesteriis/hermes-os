//! Executes an owner-local search query with a transient Vault-derived key.

use hermes_communications_api::CommunicationSearchHitV1;
use hermes_communications_domain::normalize_search_query_v1;
use hermes_communications_persistence::CommunicationsDurablePersistence;

use crate::{
    search_access::{CommunicationsSearchAccessErrorV1, CommunicationsSearchAccessV1},
    search_digest::keyed_search_token_digest_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchQueryErrorV1 {
    InvalidQuery,
    Unavailable,
}

pub async fn search_communications_v1(
    persistence: &CommunicationsDurablePersistence,
    access: &mut CommunicationsSearchAccessV1,
    query: &str,
    limit: u16,
) -> Result<Vec<CommunicationSearchHitV1>, CommunicationsSearchQueryErrorV1> {
    if !(1..=100).contains(&limit) {
        return Err(CommunicationsSearchQueryErrorV1::InvalidQuery);
    }
    let key = access.ensure_index_key().map_err(access_error)?;
    let digests = query_token_digests_v1(query, &key)?;
    persistence
        .search_by_token_digests(&digests, limit)
        .await
        .map_err(|_| CommunicationsSearchQueryErrorV1::Unavailable)
}

fn query_token_digests_v1(
    query: &str,
    key: &[u8],
) -> Result<Vec<[u8; 32]>, CommunicationsSearchQueryErrorV1> {
    let normalized = normalize_search_query_v1(query)
        .map_err(|_| CommunicationsSearchQueryErrorV1::InvalidQuery)?;
    normalized
        .tokens
        .iter()
        .map(|token| {
            keyed_search_token_digest_v1(key, token)
                .map_err(|_| CommunicationsSearchQueryErrorV1::Unavailable)
        })
        .collect()
}

fn access_error(_: CommunicationsSearchAccessErrorV1) -> CommunicationsSearchQueryErrorV1 {
    CommunicationsSearchQueryErrorV1::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_digests_are_normalized_and_do_not_retain_query_text() {
        let digests = query_token_digests_v1("Привет, ПРИВЕТ!", &[7; 32]).expect("digests");
        assert_eq!(digests.len(), 1);
        assert_eq!(
            query_token_digests_v1("", &[7; 32]),
            Err(CommunicationsSearchQueryErrorV1::InvalidQuery),
        );
    }
}
