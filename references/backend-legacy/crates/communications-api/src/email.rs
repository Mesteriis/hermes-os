use std::future::Future;
use std::io;
use std::pin::Pin;

#[derive(Clone, Debug)]
pub struct OutgoingEmail {
    pub from: String,
    pub message_id: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub attachments: Vec<OutgoingEmailAttachment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutgoingEmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub disposition: String,
    pub content_id: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SendResult {
    pub message_id: String,
    pub accepted_recipients: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub starttls: bool,
    pub username: String,
}

impl SmtpConfig {
    pub fn new(host: impl Into<String>, port: u16, tls: bool, username: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            tls,
            starttls: false,
            username: username.into(),
        }
    }

    pub fn starttls(mut self, starttls: bool) -> Self {
        self.starttls = starttls;
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmailSendError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("SMTP protocol error: {0}")]
    Protocol(String),
    #[error("provider send error: {0}")]
    Provider(String),
}

pub struct GmailOutboxSendRequest<'a> {
    pub account_id: &'a str,
    pub oauth_secret_ref: &'a str,
    pub api_base_url: &'a str,
    pub email: &'a OutgoingEmail,
}

pub trait GmailOutboxTransport: Clone + Send + Sync {
    fn send<'a>(
        &'a self,
        request: GmailOutboxSendRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>>;
}
