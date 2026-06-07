use crate::secrets::ResolvedSecret;
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
    fn uid_set(uids: &[u32]) -> String {
        uids.iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    pub async fn mark_seen(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
    ) -> Result<(), ImapWriteError> {
        with_imap_session(config, |mut s| async move {
            s.uid_store(Self::uid_set(uids), "+FLAGS (\\Seen)")
                .await?
                .try_collect::<Vec<_>>()
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn delete_messages(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
    ) -> Result<(), ImapWriteError> {
        with_imap_session(config, |mut s| async move {
            s.uid_store(Self::uid_set(uids), "+FLAGS (\\Deleted)")
                .await?
                .try_collect::<Vec<_>>()
                .await?;
            s.expunge().await?.try_collect::<Vec<_>>().await?;
            Ok(())
        })
        .await
    }
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
        s.select(config.mailbox).await?;
        s
    } else {
        let client = async_imap::Client::new(Box::new(tcp_stream) as Box<dyn AsyncReadWrite>);
        let mut s = client
            .login(config.username, config.password.expose_for_runtime())
            .await
            .map_err(|(e, _)| ImapWriteError::Imap(e))?;
        s.select(config.mailbox).await?;
        s
    };
    f(session).await?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum ImapWriteError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tls(#[from] async_native_tls::Error),
    #[error(transparent)]
    Imap(#[from] async_imap::error::Error),
}
