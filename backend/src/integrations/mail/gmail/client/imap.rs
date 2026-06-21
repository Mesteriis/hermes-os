use std::fmt::Debug;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use futures::TryStreamExt;
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::integrations::mail::sync::{
    EmailSyncBatch, FetchedCommunicationSourceMessage, imap_mailbox_stream_id,
};
use crate::platform::secrets::ResolvedSecret;

use super::errors::EmailProviderNetworkError;
use super::helpers::{
    imap_checkpoint, imap_uid_search_query, next_imap_uid_floor, retain_uids_from_floor,
    select_uids_for_fetch, sha256_fingerprint, uid_set,
};
use super::options::ImapFetchOptions;

const IMAP_UID_FETCH_CHUNK_SIZE: usize = 10;
const IMAP_UID_FETCH_TIMEOUT_SECONDS: u64 = 60;

#[derive(Clone, Debug, Default)]
pub struct ImapNetworkClient;

impl ImapNetworkClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch_raw_messages(
        &self,
        password: &ResolvedSecret,
        options: &ImapFetchOptions,
    ) -> Result<EmailSyncBatch, EmailProviderNetworkError> {
        options.validate()?;

        let address = (options.host.as_str(), options.port);
        let tcp_stream = tokio::net::TcpStream::connect(address).await?;
        if options.tls {
            let tls_stream = async_native_tls::connect(options.host.as_str(), tcp_stream).await?;
            fetch_imap_with_client(async_imap::Client::new(tls_stream), password, options).await
        } else {
            fetch_imap_with_client(async_imap::Client::new(tcp_stream), password, options).await
        }
    }
}

async fn fetch_imap_with_client<T>(
    mut client: async_imap::Client<T>,
    password: &ResolvedSecret,
    options: &ImapFetchOptions,
) -> Result<EmailSyncBatch, EmailProviderNetworkError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    client
        .read_response()
        .await?
        .ok_or(EmailProviderNetworkError::UnexpectedProviderResponse {
            message: "missing IMAP greeting",
        })?;

    let mut session = client
        .login(&options.username, password.expose_for_runtime())
        .await
        .map_err(|(error, _client)| EmailProviderNetworkError::Imap(error))?;
    let mailbox = session.examine(&options.mailbox).await?;
    let requested_uid_floor = next_imap_uid_floor(options.last_seen_uid);
    let uids = match requested_uid_floor {
        Some(first_uid) => {
            let uids: Vec<u32> = session
                .uid_search(imap_uid_search_query(first_uid))
                .await?
                .into_iter()
                .collect();
            let uids = retain_uids_from_floor(uids, first_uid);
            select_uids_for_fetch(uids, options.max_messages, options.latest_messages)
        }
        None => Vec::new(),
    };

    let messages = fetch_imap_uid_chunks(&mut session, &mailbox, options, &uids).await?;
    let latest_uid = messages
        .iter()
        .filter_map(|message| message.provider_record_id.parse::<u32>().ok())
        .max()
        .or(options.last_seen_uid);
    session.logout().await?;

    Ok(EmailSyncBatch {
        provider_kind: options.provider_kind,
        stream_id: imap_mailbox_stream_id(&options.mailbox),
        checkpoint: Some(imap_checkpoint(
            &options.mailbox,
            mailbox.uid_validity,
            latest_uid,
        )),
        messages,
    })
}

async fn fetch_imap_uid_chunks<T>(
    session: &mut async_imap::Session<T>,
    mailbox: &async_imap::types::Mailbox,
    options: &ImapFetchOptions,
    uids: &[u32],
) -> Result<Vec<FetchedCommunicationSourceMessage>, EmailProviderNetworkError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    let mut messages = Vec::new();
    for chunk in uids.chunks(IMAP_UID_FETCH_CHUNK_SIZE) {
        let uid_set = uid_set(chunk);
        let fetched_messages =
            tokio::time::timeout(Duration::from_secs(IMAP_UID_FETCH_TIMEOUT_SECONDS), async {
                session
                    .uid_fetch(uid_set, "(UID BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
                    .await?
                    .try_collect::<Vec<_>>()
                    .await
            })
            .await
            .map_err(|_| EmailProviderNetworkError::ProviderTimeout {
                operation: "imap_uid_fetch",
            })??;

        for fetched_message in fetched_messages {
            let uid = fetched_message
                .uid
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "uid" })?;
            let body = fetched_message
                .body()
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "rfc822" })?;
            let uid_string = uid.to_string();
            let occurred_at = fetched_message
                .internal_date()
                .map(|internal_date| internal_date.with_timezone(&Utc));

            messages.push(FetchedCommunicationSourceMessage {
                provider_record_id: uid_string.clone(),
                source_fingerprint: sha256_fingerprint([
                    "imap".as_bytes(),
                    uid_string.as_bytes(),
                    body,
                ]),
                occurred_at,
                payload: json!({
                    "provider": options.provider_kind.as_str(),
                    "transport": "imap",
                    "mailbox": options.mailbox,
                    "uid": uid,
                    "uid_validity": mailbox.uid_validity,
                    "raw_rfc822_base64": BASE64_STANDARD.encode(body),
                    "rfc822_size": fetched_message.size
                }),
            });
        }
    }

    Ok(messages)
}
