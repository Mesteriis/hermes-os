//! Bounded Gmail network execution outside the Mail control loop.

use hermes_mail_gmail::{
    GmailAdapterErrorV1, GmailApiClientV1, GmailListMessagesRequestV1, GmailRawMessageV1,
    history_message_ids,
};
use zeroize::Zeroizing;

pub struct PreparedGmailSyncProviderOperationV1 {
    pub(crate) connection_id: String,
    pub(crate) operation_id: String,
    pub(crate) client: GmailApiClientV1,
    pub(crate) access_token: Zeroizing<Vec<u8>>,
    pub(crate) cursor: GmailSyncProviderCursorV1,
    pub(crate) max_results: u16,
    pub(crate) windows: u32,
    pub(crate) observed_at_unix_seconds: i64,
    pub(crate) observed_at_nanos: i32,
}

pub(crate) enum GmailSyncProviderCursorV1 {
    Full {
        page_token: Option<String>,
    },
    History {
        start_history_id: String,
        page_token: Option<String>,
    },
}

pub struct CompletedGmailSyncProviderOperationV1 {
    pub(crate) connection_id: String,
    pub(crate) operation_id: String,
    pub(crate) pages: Vec<GmailSyncProviderPageV1>,
    pub(crate) outcome: GmailSyncProviderOutcomeV1,
    pub(crate) observed_at_unix_seconds: i64,
    pub(crate) observed_at_nanos: i32,
}

impl CompletedGmailSyncProviderOperationV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

pub(crate) enum GmailSyncProviderPageV1 {
    Full {
        messages: Vec<(String, GmailRawMessageV1)>,
        next_page_token: Option<String>,
    },
    History {
        messages: Vec<(String, GmailRawMessageV1)>,
        start_history_id: String,
        checkpoint_history_id: String,
        next_page_token: Option<String>,
    },
}

pub(crate) enum GmailSyncProviderOutcomeV1 {
    Complete,
    HistoryExpired,
    Failed(GmailSyncProviderFailureV1),
}

pub(crate) enum GmailSyncProviderFailureV1 {
    Credential,
    Provider,
}

pub async fn execute_gmail_sync_provider_operation(
    prepared: PreparedGmailSyncProviderOperationV1,
) -> CompletedGmailSyncProviderOperationV1 {
    let PreparedGmailSyncProviderOperationV1 {
        connection_id,
        operation_id,
        client,
        access_token,
        cursor,
        max_results,
        windows,
        observed_at_unix_seconds,
        observed_at_nanos,
    } = prepared;
    let token = match std::str::from_utf8(&access_token) {
        Ok(token) => token,
        Err(_) => {
            return CompletedGmailSyncProviderOperationV1 {
                connection_id,
                operation_id,
                pages: Vec::new(),
                outcome: GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Credential),
                observed_at_unix_seconds,
                observed_at_nanos,
            };
        }
    };
    let (pages, outcome) = match cursor {
        GmailSyncProviderCursorV1::Full { page_token } => {
            fetch_full_pages(&client, token, page_token, max_results, windows).await
        }
        GmailSyncProviderCursorV1::History {
            start_history_id,
            page_token,
        } => fetch_history_pages(&client, token, start_history_id, page_token, windows).await,
    };
    CompletedGmailSyncProviderOperationV1 {
        connection_id,
        operation_id,
        pages,
        outcome,
        observed_at_unix_seconds,
        observed_at_nanos,
    }
}

async fn fetch_full_pages(
    client: &GmailApiClientV1,
    token: &str,
    mut page_token: Option<String>,
    max_results: u16,
    windows: u32,
) -> (Vec<GmailSyncProviderPageV1>, GmailSyncProviderOutcomeV1) {
    let mut pages = Vec::new();
    for _ in 0..windows {
        let page = match client
            .list_messages(
                token,
                &GmailListMessagesRequestV1 {
                    max_results,
                    page_token,
                    query: None,
                    label_ids: Vec::new(),
                },
            )
            .await
        {
            Ok(page) => page,
            Err(_) => {
                return (
                    pages,
                    GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider),
                );
            }
        };
        let next_page_token = page.next_page_token;
        let messages = match fetch_raw_messages(
            client,
            token,
            page.messages.into_iter().map(|message| message.id),
        )
        .await
        {
            Ok(messages) => messages,
            Err(error) => return (pages, error),
        };
        let has_next_page = next_page_token.is_some();
        pages.push(GmailSyncProviderPageV1::Full {
            messages,
            next_page_token: next_page_token.clone(),
        });
        page_token = next_page_token;
        if !has_next_page {
            break;
        }
    }
    (pages, GmailSyncProviderOutcomeV1::Complete)
}

async fn fetch_history_pages(
    client: &GmailApiClientV1,
    token: &str,
    start_history_id: String,
    mut page_token: Option<String>,
    windows: u32,
) -> (Vec<GmailSyncProviderPageV1>, GmailSyncProviderOutcomeV1) {
    let mut pages = Vec::new();
    for _ in 0..windows {
        let page = match client
            .list_history(token, &start_history_id, page_token.as_deref())
            .await
        {
            Ok(page) => page,
            Err(GmailAdapterErrorV1::ProviderStatus(404)) => {
                return (pages, GmailSyncProviderOutcomeV1::HistoryExpired);
            }
            Err(_) => {
                return (
                    pages,
                    GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider),
                );
            }
        };
        let message_ids = history_message_ids(&page);
        let Some(checkpoint_history_id) = page
            .history_id
            .as_deref()
            .filter(|value| valid_history_id(value))
            .map(str::to_owned)
        else {
            return (
                pages,
                GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider),
            );
        };
        let messages = match fetch_raw_messages(client, token, message_ids.into_iter()).await {
            Ok(messages) => messages,
            Err(error) => return (pages, error),
        };
        let next_page_token = page.next_page_token;
        let has_next_page = next_page_token.is_some();
        pages.push(GmailSyncProviderPageV1::History {
            messages,
            start_history_id: start_history_id.clone(),
            checkpoint_history_id,
            next_page_token: next_page_token.clone(),
        });
        page_token = next_page_token;
        if !has_next_page {
            break;
        }
    }
    (pages, GmailSyncProviderOutcomeV1::Complete)
}

async fn fetch_raw_messages(
    client: &GmailApiClientV1,
    token: &str,
    message_ids: impl Iterator<Item = String>,
) -> Result<Vec<(String, GmailRawMessageV1)>, GmailSyncProviderOutcomeV1> {
    let mut messages = Vec::new();
    for message_id in message_ids {
        let raw = client
            .fetch_raw_message(token, &message_id)
            .await
            .map_err(|_| {
                GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider)
            })?;
        messages.push((message_id, raw));
    }
    Ok(messages)
}

fn valid_history_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::valid_history_id;

    #[test]
    fn history_cursor_accepts_only_nonempty_decimal_ids() {
        assert!(valid_history_id("123"));
        assert!(!valid_history_id(""));
        assert!(!valid_history_id("12a"));
        assert!(!valid_history_id("-1"));
    }
}
