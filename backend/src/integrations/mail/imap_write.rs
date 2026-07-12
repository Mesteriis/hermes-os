use crate::platform::secrets::ResolvedSecret;
use futures::TryStreamExt;
use std::fmt::Debug;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

pub struct ImapWriteClient;

#[derive(Clone, Copy, Debug)]
pub struct ImapWriteConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub tls: bool,
    pub username: &'a str,
    pub password: &'a ResolvedSecret,
    pub mailbox: &'a str,
    pub expected_uid_validity: Option<u32>,
}

impl Default for ImapWriteClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ImapWriteClient {
    pub fn new() -> Self {
        Self
    }
    fn uid_set(uids: &[u32]) -> Result<String, ImapWriteError> {
        if uids.is_empty() || uids.contains(&0) {
            return Err(ImapWriteError::InvalidUidSet);
        }
        Ok(uids
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(","))
    }

    pub async fn add_flags(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
        flags: &[&str],
    ) -> Result<(), ImapWriteError> {
        self.store_flags(config, uids, "+FLAGS.SILENT", flags).await
    }

    pub async fn remove_flags(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
        flags: &[&str],
    ) -> Result<(), ImapWriteError> {
        self.store_flags(config, uids, "-FLAGS.SILENT", flags).await
    }

    async fn store_flags(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
        operation: &'static str,
        flags: &[&str],
    ) -> Result<(), ImapWriteError> {
        let uid_set = Self::uid_set(uids)?;
        let flags = render_flags(flags)?;
        with_imap_session(config, |mut session| async move {
            session
                .uid_store(uid_set, format!("{operation} ({flags})"))
                .await?
                .try_collect::<Vec<_>>()
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn copy_messages(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
        destination_mailbox: &str,
    ) -> Result<(), ImapWriteError> {
        let uid_set = Self::uid_set(uids)?;
        validate_mailbox(destination_mailbox)?;
        with_imap_session(config, |mut session| async move {
            session.uid_copy(uid_set, destination_mailbox).await
        })
        .await
    }

    pub async fn move_messages(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
        destination_mailbox: &str,
    ) -> Result<(), ImapWriteError> {
        let uid_set = Self::uid_set(uids)?;
        validate_mailbox(destination_mailbox)?;
        with_imap_session(config, |mut session| async move {
            session.uid_mv(uid_set, destination_mailbox).await
        })
        .await
    }

    pub async fn mark_seen(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
    ) -> Result<(), ImapWriteError> {
        self.add_flags(config, uids, &["\\Seen"]).await
    }

    pub async fn mark_unseen(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
    ) -> Result<(), ImapWriteError> {
        self.remove_flags(config, uids, &["\\Seen"]).await
    }
}

fn render_flags(flags: &[&str]) -> Result<String, ImapWriteError> {
    if flags.is_empty() {
        return Err(ImapWriteError::InvalidFlag);
    }
    for flag in flags {
        let valid_system_flag = matches!(*flag, "\\Seen" | "\\Answered" | "\\Flagged" | "\\Draft");
        let valid_keyword = !flag.is_empty()
            && flag
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
        if !valid_system_flag && !valid_keyword {
            return Err(ImapWriteError::InvalidFlag);
        }
    }
    Ok(flags.join(" "))
}

fn validate_mailbox(mailbox: &str) -> Result<(), ImapWriteError> {
    if mailbox.trim().is_empty() || mailbox.contains(['\r', '\n', '\0']) {
        return Err(ImapWriteError::InvalidMailbox);
    }
    Ok(())
}

type ImapSession = async_imap::Session<Box<dyn AsyncReadWrite>>;
trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin + Send + Debug {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send + Debug> AsyncReadWrite for T {}

async fn with_imap_session<F, Fut>(config: &ImapWriteConfig<'_>, f: F) -> Result<(), ImapWriteError>
where
    F: FnOnce(ImapSession) -> Fut,
    Fut: std::future::Future<Output = Result<(), async_imap::error::Error>>,
{
    let address = (config.host, config.port);
    let tcp_stream = tokio::net::TcpStream::connect(address).await?;
    let session: ImapSession = if config.tls {
        let tls_stream = async_native_tls::connect(config.host, tcp_stream).await?;
        let client = async_imap::Client::new(Box::new(tls_stream) as Box<dyn AsyncReadWrite>);
        let mut s = client
            .login(config.username, config.password.expose_for_runtime())
            .await
            .map_err(|(e, _)| ImapWriteError::Imap(e))?;
        select_mailbox(&mut s, config).await?;
        s
    } else {
        let client = async_imap::Client::new(Box::new(tcp_stream) as Box<dyn AsyncReadWrite>);
        let mut s = client
            .login(config.username, config.password.expose_for_runtime())
            .await
            .map_err(|(e, _)| ImapWriteError::Imap(e))?;
        select_mailbox(&mut s, config).await?;
        s
    };
    f(session).await?;
    Ok(())
}

async fn select_mailbox(
    session: &mut ImapSession,
    config: &ImapWriteConfig<'_>,
) -> Result<(), ImapWriteError> {
    let mailbox = session.select(config.mailbox).await?;
    validate_uid_validity(config.expected_uid_validity, mailbox.uid_validity)
}

fn validate_uid_validity(
    expected_uid_validity: Option<u32>,
    selected_uid_validity: Option<u32>,
) -> Result<(), ImapWriteError> {
    if expected_uid_validity.is_some() && expected_uid_validity != selected_uid_validity {
        return Err(ImapWriteError::UidValidityMismatch);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ImapWriteError {
    #[error("IMAP UID set must contain only non-zero UIDs")]
    InvalidUidSet,
    #[error("IMAP flag is empty or contains unsafe characters")]
    InvalidFlag,
    #[error("IMAP mailbox is empty or contains unsafe characters")]
    InvalidMailbox,
    #[error("IMAP mailbox UIDVALIDITY no longer matches the message locator")]
    UidValidityMismatch,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tls(#[from] async_native_tls::Error),
    #[error(transparent)]
    Imap(#[from] async_imap::error::Error),
}

#[cfg(test)]
mod tests {
    use super::{
        ImapWriteClient, ImapWriteError, render_flags, validate_mailbox, validate_uid_validity,
    };

    #[test]
    fn validates_uid_sets_flags_and_mailboxes_before_writing() {
        assert_eq!(ImapWriteClient::uid_set(&[7, 9]).expect("UID set"), "7,9");
        assert!(ImapWriteClient::uid_set(&[]).is_err());
        assert_eq!(
            render_flags(&["\\Seen", "Follow-up"]).expect("flags"),
            "\\Seen Follow-up"
        );
        assert!(render_flags(&["unsafe flag"]).is_err());
        assert!(render_flags(&["\\Deleted"]).is_err());
        assert!(validate_mailbox("Archive/2026").is_ok());
        assert!(validate_mailbox("Archive\r\nEXPUNGE").is_err());
    }

    #[test]
    fn refuses_a_stale_uid_validity_before_a_mutation() {
        assert!(validate_uid_validity(Some(42), Some(42)).is_ok());
        assert!(validate_uid_validity(None, Some(42)).is_ok());
        assert!(matches!(
            validate_uid_validity(Some(42), Some(43)),
            Err(ImapWriteError::UidValidityMismatch)
        ));
        assert!(matches!(
            validate_uid_validity(Some(42), None),
            Err(ImapWriteError::UidValidityMismatch)
        ));
    }
}
