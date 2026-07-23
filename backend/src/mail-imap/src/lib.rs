//! IMAP read-only adapter boundary for ADR-0239.
//!
//! Supports IMAPS (`993`) against `INBOX` with read-only `EXAMINE`.

#![allow(clippy::items_after_test_module)]

use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

use async_imap::Session;
use async_native_tls::TlsConnector;
use async_std::future;
use async_std::net::TcpStream;
use async_std::task;
use futures_util::TryStreamExt;
use futures_util::io::{AsyncRead, AsyncWrite};
use hermes_mail_api::{MAX_PLAIN_TEXT_BYTES, MAX_WINDOW, MAX_WINDOWS, WINDOW_DEADLINE_SECONDS};
use hermes_mail_core::rfc822::{AttachmentDispositionV1, attachment_metadata};

pub const PACKAGE: &str = "hermes-mail-imap";

mod retry {
    #[derive(Clone, Copy)]
    pub struct ImapRetryPolicy {
        pub max_attempts: u8,
        pub delay_millis: u64,
    }

    // Retry policy is explicitly defined as policy data to make future timeout/attempt tuning
    // visible and testable without changing IMAP parsing/fetching logic.
    pub const MAX_SYNC_ATTEMPTS: u8 = 255;
    pub const RETRY_DELAY_MILLIS: u64 = 120;
    pub const IMAP_SYNC_RETRY_POLICY: ImapRetryPolicy = ImapRetryPolicy {
        max_attempts: MAX_SYNC_ATTEMPTS,
        delay_millis: RETRY_DELAY_MILLIS,
    };
}

pub const MAX_ATTEMPTS: u8 = retry::IMAP_SYNC_RETRY_POLICY.max_attempts;

const IMAP_UID_FETCH_CHUNK_SIZE: usize = MAX_WINDOW as usize;
const IMAP_UID_FETCH_TIMEOUT_SECONDS: u64 = WINDOW_DEADLINE_SECONDS;
const SNAPSHOT_PREVIEW_BYTES: usize = 160;

#[derive(Clone, Debug, PartialEq)]
pub struct ImapMessage {
    pub uid: u32,
    pub subject: String,
    pub snippet: String,
    pub has_plain_text: bool,
    pub plain_text_body: Option<Vec<u8>>,
    pub attachments: Vec<ImapAttachment>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapAttachmentDisposition {
    Attachment,
    Inline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapAttachment {
    pub part_id: u16,
    pub filename: Option<String>,
    pub media_type: String,
    pub declared_bytes: u64,
    pub disposition: ImapAttachmentDisposition,
}

#[derive(Clone, Debug)]
pub struct ImapSyncResult {
    pub messages: Vec<ImapMessage>,
    pub attempts: u8,
    pub window: u32,
    pub has_more: bool,
}

#[derive(Debug)]
pub struct ImapError {
    kind: &'static str,
    message: String,
}

impl Display for ImapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ImapError {}

impl ImapError {
    fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

pub fn sync_inbox(
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    window: u32,
    windows: u32,
) -> Result<ImapSyncResult, String> {
    let password = password.ok_or_else(|| "imap password is required".to_owned())?;
    if !supports_read_only_sync(window) || !supports_read_only_windows(windows) {
        return Err("window unsupported for read-only sync".to_owned());
    }
    let limit = window as u64 * windows as u64;
    sync_inbox_with_retry(host, port, username, password, limit, run_imap_sync)
}

fn sync_inbox_with_retry<F>(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    limit: u64,
    attempt: F,
) -> Result<ImapSyncResult, String>
where
    F: FnMut(&str, u16, &str, &str, usize) -> Result<ImapSyncResult, ImapError>,
{
    let attempted_limit = usize::try_from(limit)
        .map_err(|_| "imap requested window does not fit runtime limits".to_owned())?;
    sync_inbox_with_retry_policy(
        host,
        port,
        username,
        password,
        attempted_limit,
        retry::IMAP_SYNC_RETRY_POLICY,
        attempt,
    )
}

fn sync_inbox_with_retry_policy<F>(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    limit: usize,
    policy: retry::ImapRetryPolicy,
    mut attempt: F,
) -> Result<ImapSyncResult, String>
where
    F: FnMut(&str, u16, &str, &str, usize) -> Result<ImapSyncResult, ImapError>,
{
    let mut attempts = 0u8;
    while attempts < policy.max_attempts {
        attempts += 1;
        match attempt(host, port, username, password, limit) {
            Ok(result) => {
                return Ok(ImapSyncResult {
                    attempts,
                    window: result.window,
                    messages: result.messages,
                    has_more: result.has_more,
                });
            }
            Err(error) => {
                eprintln!("imap sync attempt {attempts} failed: {error}");
                if attempts < policy.max_attempts {
                    std::thread::sleep(Duration::from_millis(policy.delay_millis));
                    continue;
                }
                return Err(format!("imap sync failed: {error}"));
            }
        }
    }
    Err("imap sync failed: unexpected retry loop termination".to_owned())
}

pub fn supports_read_only_sync(window: u32) -> bool {
    window > 0 && window <= MAX_WINDOW
}

pub fn supports_read_only_windows(windows: u32) -> bool {
    windows > 0 && windows <= MAX_WINDOWS
}

#[cfg(test)]
#[test]
fn supports_read_only_sync_uses_mail_window_limit_only() {
    assert!(supports_read_only_sync(MAX_WINDOW));
    assert!(!supports_read_only_sync(MAX_WINDOW + 1));
}

fn run_imap_sync(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    requested: usize,
) -> Result<ImapSyncResult, ImapError> {
    let result =
        task::block_on(
            async move { imap_sync_once(host, port, username, password, requested).await },
        )?;

    Ok(result)
}

async fn imap_sync_once(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    requested: usize,
) -> Result<ImapSyncResult, ImapError> {
    let mut session = open_session(host, port, username, password).await?;
    session.examine("INBOX").await.map_err(|error| {
        ImapError::new("protocol", format!("imap EXAMINE INBOX failed: {error}"))
    })?;

    let all_uids = if requested == 0 {
        Vec::new()
    } else {
        let ids = session.uid_search("UID 1:*").await.map_err(|error| {
            ImapError::new("protocol", format!("imap uid search failed: {error}"))
        })?;

        let mut sorted = ids.into_iter().collect::<Vec<_>>();
        sorted.sort_unstable();
        sorted
    };

    let has_more = all_uids.len() > requested;
    let fetch_uids = if all_uids.len() > requested {
        &all_uids[all_uids.len() - requested..]
    } else {
        &all_uids[..]
    };
    let messages = fetch_messages(&mut session, fetch_uids).await?;
    session
        .logout()
        .await
        .map_err(|error| ImapError::new("protocol", format!("imap logout failed: {error}")))?;

    Ok(ImapSyncResult {
        messages,
        attempts: 1,
        window: uids_window(fetch_uids.len()),
        has_more,
    })
}

async fn open_session(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<Session<async_native_tls::TlsStream<TcpStream>>, ImapError> {
    let address = (host, port);
    let tcp_stream = TcpStream::connect(address).await.map_err(|error| {
        ImapError::new(
            "network",
            format!("tcp connect to {host}:{port} failed: {error}"),
        )
    })?;
    let tls_stream = TlsConnector::new()
        .connect(host, tcp_stream)
        .await
        .map_err(|error| {
            ImapError::new(
                "tls",
                format!("tls connect to {host}:{port} failed: {error}"),
            )
        })?;
    let mut client = async_imap::Client::new(tls_stream);
    client
        .read_response()
        .await
        .map_err(|error| ImapError::new("protocol", format!("imap greeting failed: {error}")))?;
    let session = client
        .login(username, password)
        .await
        .map_err(|(error, _)| ImapError::new("auth", format!("imap login failed: {error}")))?;
    Ok(session)
}

async fn fetch_messages<T>(
    session: &mut Session<T>,
    uids: &[u32],
) -> Result<Vec<ImapMessage>, ImapError>
where
    T: AsyncRead + AsyncWrite + Debug + Send + Unpin,
{
    let mut messages = Vec::new();
    for chunk in uids.chunks(IMAP_UID_FETCH_CHUNK_SIZE) {
        let fetched_messages =
            future::timeout(Duration::from_secs(IMAP_UID_FETCH_TIMEOUT_SECONDS), async {
                session
                    .uid_fetch(uid_set(chunk), "(UID BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
                    .await?
                    .try_collect::<Vec<_>>()
                    .await
            })
            .await
            .map_err(|_| {
                ImapError::new(
                    "timeout",
                    format!("uid fetch exceeded {IMAP_UID_FETCH_TIMEOUT_SECONDS}s window"),
                )
            })?
            .map_err(|error| ImapError::new("protocol", format!("uid fetch failed: {error}")))?;

        for message in fetched_messages {
            let uid = message
                .uid
                .ok_or_else(|| ImapError::new("protocol", "missing UID in fetched message"))?;
            let body = message
                .body()
                .ok_or_else(|| ImapError::new("protocol", "missing BODY.PEEK[] payload"))?;
            let (subject, snippet, has_plain_text) = decode_message_preview(body);
            let attachments = decode_message_attachments(body);
            messages.push(ImapMessage {
                uid,
                subject,
                snippet,
                has_plain_text,
                plain_text_body: hermes_mail_core::rfc822::direct_plain_text_body(body),
                attachments,
            });
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_mail_api::{IMAP_PORT, MAX_WINDOW, MAX_WINDOWS};

    #[test]
    fn uid_fetch_timeout_matches_contract_window_deadline() {
        assert_eq!(IMAP_UID_FETCH_TIMEOUT_SECONDS, WINDOW_DEADLINE_SECONDS);
    }

    #[test]
    fn default_retry_policy_max_attempts_match_public_constant() {
        assert_eq!(MAX_ATTEMPTS, retry::MAX_SYNC_ATTEMPTS);
        assert_eq!(retry::IMAP_SYNC_RETRY_POLICY.max_attempts, MAX_ATTEMPTS);
    }

    #[test]
    fn retries_until_attempt_limit_on_repeated_failures() {
        let mut attempts = 0u8;
        let result = sync_inbox_with_retry(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            |_host, _port, _username, _password, _limit| {
                attempts += 1;
                Err(ImapError::new("protocol", "temporary sync failure"))
            },
        );

        assert_eq!(attempts, MAX_ATTEMPTS);
        assert!(matches!(result, Err(error) if error.contains("imap sync failed")));
    }

    #[test]
    fn retries_respect_custom_retry_policy() {
        let mut attempts = 0u8;
        let policy = retry::ImapRetryPolicy {
            max_attempts: 2,
            delay_millis: 0,
        };
        let result = sync_inbox_with_retry_policy(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            policy,
            |_host, _port, _username, _password, _limit| {
                attempts += 1;
                Err(ImapError::new("protocol", "temporary sync failure"))
            },
        );

        assert_eq!(attempts, 2);
        assert!(matches!(result, Err(error) if error.contains("imap sync failed")));
    }

    #[test]
    fn stops_retrying_after_first_success() {
        let mut attempts = 0u8;
        let result = sync_inbox_with_retry(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            |_host, _port, _username, _password, _limit| {
                attempts += 1;
                if attempts < 3 {
                    return Err(ImapError::new("protocol", "temporary sync failure"));
                }
                Ok(ImapSyncResult {
                    attempts: 1,
                    window: 1,
                    messages: Vec::new(),
                    has_more: false,
                })
            },
        );

        assert!(result.is_ok());
        assert_eq!(attempts, 3);
        let summary = result.expect("success");
        assert_eq!(summary.attempts, 3);
    }

    #[test]
    fn succeeds_on_last_allowed_attempt() {
        let mut attempts = 0u8;
        let result = sync_inbox_with_retry(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            |_host, _port, _username, _password, _limit| {
                attempts += 1;
                if attempts == MAX_ATTEMPTS {
                    return Ok(ImapSyncResult {
                        attempts: 1,
                        window: 1,
                        messages: Vec::new(),
                        has_more: false,
                    });
                }
                Err(ImapError::new("protocol", "temporary sync failure"))
            },
        );

        assert_eq!(attempts, MAX_ATTEMPTS);
        assert!(result.is_ok());
        let summary = result.expect("success on final attempt");
        assert_eq!(summary.attempts, MAX_ATTEMPTS);
    }

    #[test]
    fn sync_inbox_with_retry_carries_large_plan_limit_without_overflow() {
        let expected_limit = (MAX_WINDOW as u64) * (MAX_WINDOWS as u64);
        let expected_limit = usize::try_from(expected_limit).expect("plan limit should fit usize");
        let mut observed_limit = 0usize;
        let mut observed_attempts = 0u8;
        assert!(expected_limit > u32::MAX as usize);
        let result = sync_inbox_with_retry_policy(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            expected_limit,
            retry::ImapRetryPolicy {
                max_attempts: 1,
                delay_millis: 0,
            },
            |_host, _port, _username, _password, limit| {
                observed_attempts += 1;
                observed_limit = limit;
                Err(ImapError::new("protocol", "temporary sync failure"))
            },
        );

        assert_eq!(observed_attempts, 1);
        assert_eq!(observed_limit, expected_limit);
        assert!(matches!(result, Err(error) if error.contains("imap sync failed")));
    }

    #[test]
    fn supports_read_only_window_limits_are_applied() {
        assert!(supports_read_only_sync(MAX_WINDOW));
        assert!(!supports_read_only_sync(MAX_WINDOW + 1));
        assert!(supports_read_only_windows(MAX_WINDOWS));
        assert!(!supports_read_only_windows(MAX_WINDOWS + 1));
    }

    #[test]
    fn extracts_only_explicit_bounded_attachment_metadata_from_nested_mime() {
        let message = concat!(
            "Content-Type: multipart/mixed; boundary=outer\r\n\r\n",
            "--outer\r\nContent-Type: text/plain\r\n\r\nhello\r\n",
            "--outer\r\nContent-Type: multipart/related; boundary=inner\r\n\r\n",
            "--inner\r\nContent-Type: application/pdf; name=report.pdf\r\n",
            "Content-Disposition: attachment; filename=report.pdf\r\n",
            "Content-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n",
            "--inner--\r\n--outer--\r\n",
        );

        assert_eq!(
            decode_message_attachments(message.as_bytes()),
            vec![ImapAttachment {
                part_id: 1,
                filename: Some("report.pdf".to_owned()),
                media_type: "application/pdf".to_owned(),
                declared_bytes: 5,
                disposition: ImapAttachmentDisposition::Attachment,
            }],
        );
    }

    #[test]
    fn rejects_attachment_with_undecidable_transfer_encoding() {
        let message = concat!(
            "Content-Type: multipart/mixed; boundary=outer\r\n\r\n",
            "--outer\r\nContent-Type: application/octet-stream\r\n",
            "Content-Disposition: attachment\r\n",
            "Content-Transfer-Encoding: quoted-printable\r\n\r\nhello=20world\r\n",
            "--outer--\r\n",
        );

        assert!(decode_message_attachments(message.as_bytes()).is_empty());
    }
}

fn decode_message_preview(body: &[u8]) -> (String, String, bool) {
    let raw = String::from_utf8_lossy(body);
    let (subject, text) = split_subject_and_body(&raw);
    let has_plain_text = !text.trim().is_empty();
    let mut snippet = if has_plain_text {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_owned()
    } else {
        String::new()
    };
    if has_plain_text {
        snippet.truncate(SNAPSHOT_PREVIEW_BYTES);
    }
    if snippet.len() > MAX_PLAIN_TEXT_BYTES {
        snippet = snippet.chars().take(SNAPSHOT_PREVIEW_BYTES).collect();
    }
    if snippet.is_empty() {
        snippet = subject.clone();
    }
    if snippet.is_empty() {
        snippet = "message".to_owned();
    }
    let has_plain_text = snippet.len() <= MAX_PLAIN_TEXT_BYTES && has_plain_text;
    (subject, snippet, has_plain_text)
}

fn split_subject_and_body(raw_message: &str) -> (String, String) {
    let mut subject = String::new();
    for line in raw_message.lines() {
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("Subject:") {
            subject = rest.trim().to_owned();
            break;
        }
    }
    let body = raw_message
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
        .replace('\r', "");
    let subject = if subject.is_empty() {
        "message".to_owned()
    } else {
        subject
    };
    (subject, body)
}

fn decode_message_attachments(raw_message: &[u8]) -> Vec<ImapAttachment> {
    attachment_metadata(raw_message)
        .into_iter()
        .map(|attachment| ImapAttachment {
            part_id: attachment.part_id,
            filename: attachment.filename,
            media_type: attachment.media_type,
            declared_bytes: attachment.declared_bytes,
            disposition: match attachment.disposition {
                AttachmentDispositionV1::Attachment => ImapAttachmentDisposition::Attachment,
                AttachmentDispositionV1::Inline => ImapAttachmentDisposition::Inline,
            },
        })
        .collect()
}

fn uids_window(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

fn uid_set(uids: &[u32]) -> String {
    uids.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
