use async_native_tls::TlsConnector;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use crate::platform::secrets::ResolvedSecret;

pub use crate::platform::communications::{
    EmailSendError, OutgoingEmail, SendResult, SmtpConfig, SmtpTransport,
};

#[derive(Clone, Default)]
pub struct LiveSmtpTransport;

impl SmtpTransport for LiveSmtpTransport {
    fn send<'a>(
        &'a self,
        config: &'a SmtpConfig,
        password: &'a ResolvedSecret,
        email: &'a OutgoingEmail,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>,
    > {
        Box::pin(async move { SmtpClient::new().send(config, password, email).await })
    }
}

pub(crate) fn smtp_outbox_transport() -> impl SmtpTransport {
    LiveSmtpTransport
}

pub struct SmtpClient;

impl Default for SmtpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SmtpClient {
    pub fn new() -> Self {
        Self
    }
    pub async fn send(
        &self,
        config: &SmtpConfig,
        password: &ResolvedSecret,
        email: &OutgoingEmail,
    ) -> Result<SendResult, EmailSendError> {
        let address = (config.host.as_str(), config.port);
        let tcp_stream = tokio::net::TcpStream::connect(address).await?;
        if config.starttls {
            starttls_smtp(tcp_stream, config, password, email).await
        } else if config.tls {
            let tls = TlsConnector::new();
            let tls_stream = tls.connect(&config.host, tcp_stream).await?;
            send_smtp(tls_stream, config, password, email).await
        } else {
            send_smtp(tcp_stream, config, password, email).await
        }
    }
}

async fn send_smtp<T: AsyncRead + AsyncWrite + Unpin>(
    stream: T,
    config: &SmtpConfig,
    password: &ResolvedSecret,
    email: &OutgoingEmail,
) -> Result<SendResult, EmailSendError> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    read_line(&mut reader, &mut buf).await?;
    send_smtp_after_greeting(reader, config, password, email).await
}

async fn starttls_smtp(
    stream: tokio::net::TcpStream,
    config: &SmtpConfig,
    password: &ResolvedSecret,
    email: &OutgoingEmail,
) -> Result<SendResult, EmailSendError> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    let greeting = read_line(&mut reader, &mut buf).await?;
    if !greeting.starts_with("220") {
        return Err(EmailSendError::Protocol(greeting));
    }
    write_cmd(&mut reader, "EHLO hermes-hub\r\n").await?;
    read_ehlo_response(&mut reader, &mut buf).await?;
    write_cmd(&mut reader, "STARTTLS\r\n").await?;
    let response = read_line(&mut reader, &mut buf).await?;
    if !response.starts_with("220") {
        return Err(EmailSendError::Protocol(response));
    }
    let tcp_stream = reader.into_inner();
    let tls = TlsConnector::new();
    let tls_stream = tls.connect(&config.host, tcp_stream).await?;
    send_smtp_after_greeting(BufReader::new(tls_stream), config, password, email).await
}

async fn send_smtp_after_greeting<T: AsyncRead + AsyncWrite + Unpin>(
    mut reader: BufReader<T>,
    config: &SmtpConfig,
    password: &ResolvedSecret,
    email: &OutgoingEmail,
) -> Result<SendResult, EmailSendError> {
    let mut buf = String::new();
    write_cmd(&mut reader, "EHLO hermes-hub\r\n").await?;
    read_ehlo_response(&mut reader, &mut buf).await?;
    write_cmd(&mut reader, "AUTH LOGIN\r\n").await?;
    read_line(&mut reader, &mut buf).await?;
    write_cmd(
        &mut reader,
        &format!("{}\r\n", base64(config.username.as_bytes())),
    )
    .await?;
    read_line(&mut reader, &mut buf).await?;
    write_cmd(
        &mut reader,
        &format!("{}\r\n", base64(password.expose_for_runtime().as_bytes())),
    )
    .await?;
    read_line(&mut reader, &mut buf).await?;
    write_cmd(&mut reader, &format!("MAIL FROM:<{}>\r\n", email.from)).await?;
    read_line(&mut reader, &mut buf).await?;
    let mut accepted = Vec::new();
    for r in email
        .to
        .iter()
        .chain(email.cc.iter())
        .chain(email.bcc.iter())
    {
        write_cmd(&mut reader, &format!("RCPT TO:<{r}>\r\n")).await?;
        if read_line(&mut reader, &mut buf).await?.starts_with("250") {
            accepted.push(r.clone());
        }
    }
    write_cmd(&mut reader, "DATA\r\n").await?;
    read_line(&mut reader, &mut buf).await?;
    let msg = build_rfc2822_message(email);
    write_cmd(&mut reader, &format!("{msg}\r\n.\r\n")).await?;
    let resp = read_line(&mut reader, &mut buf).await?;
    let _ = write_cmd(&mut reader, "QUIT\r\n").await;
    Ok(SendResult {
        message_id: resp.split_whitespace().nth(1).unwrap_or("unknown").into(),
        accepted_recipients: accepted,
    })
}

async fn read_ehlo_response<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    buf: &mut String,
) -> Result<(), EmailSendError> {
    loop {
        let line = read_line(reader, buf).await?;
        if !line.starts_with("250-") {
            return Ok(());
        }
    }
}

async fn read_line<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    buf: &mut String,
) -> Result<String, EmailSendError> {
    buf.clear();
    reader.read_line(buf).await?;
    Ok(buf.trim().to_owned())
}

async fn write_cmd<W: AsyncWrite + Unpin>(
    writer: &mut W,
    data: &str,
) -> Result<(), EmailSendError> {
    writer.write_all(data.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub fn build_rfc2822_message(email: &OutgoingEmail) -> String {
    let now = chrono::Utc::now().to_rfc2822();
    let mut message = format!("From: {}\r\nTo: {}\r\n", email.from, email.to.join(", "));
    if !email.cc.is_empty() {
        message.push_str(&format!("Cc: {}\r\n", email.cc.join(", ")));
    }
    if let Some(ref r) = email.in_reply_to {
        message.push_str(&format!("In-Reply-To: {r}\r\n"));
    }
    if !email.references.is_empty() {
        message.push_str(&format!("References: {}\r\n", email.references.join(" ")));
    }
    message.push_str(&format!(
        "Date: {now}\r\nSubject: {}\r\nMIME-Version: 1.0\r\n",
        email.subject
    ));

    match email
        .body_html
        .as_deref()
        .map(str::trim)
        .filter(|body_html| !body_html.is_empty())
    {
        Some(body_html) => {
            let boundary = multipart_alternative_boundary(email);
            message.push_str(&format!(
                "Content-Type: multipart/alternative; boundary=\"{boundary}\"\r\n\r\n"
            ));
            message.push_str(&format!(
                "--{boundary}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{}\r\n",
                email.body_text
            ));
            message.push_str(&format!(
                "--{boundary}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{body_html}\r\n"
            ));
            message.push_str(&format!("--{boundary}--"));
        }
        None => {
            message.push_str(&format!(
                "Content-Type: text/plain; charset=utf-8\r\n\r\n{}",
                email.body_text
            ));
        }
    }

    message
}

fn multipart_alternative_boundary(email: &OutgoingEmail) -> String {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    digest.update(email.from.as_bytes());
    digest.update(b"\0");
    digest.update(email.to.join(",").as_bytes());
    digest.update(b"\0");
    digest.update(email.subject.as_bytes());
    digest.update(b"\0");
    digest.update(email.body_text.as_bytes());
    digest.update(b"\0");
    if let Some(body_html) = &email.body_html {
        digest.update(body_html.as_bytes());
    }

    let digest = digest.finalize();
    let mut suffix = String::with_capacity(24);
    for byte in digest.iter().take(12) {
        suffix.push(hex_char(byte >> 4));
        suffix.push(hex_char(byte & 0x0f));
    }
    format!("hermes-alt-{suffix}")
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must fit in 0..=15"),
    }
}

fn base64(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outgoing_email(body_html: Option<String>) -> OutgoingEmail {
        OutgoingEmail {
            from: "sender@example.com".to_owned(),
            to: vec!["recipient@example.com".to_owned()],
            cc: vec!["copy@example.com".to_owned()],
            bcc: Vec::new(),
            subject: "Rich body".to_owned(),
            body_text: "Plain body".to_owned(),
            body_html,
            in_reply_to: Some("<parent@example.com>".to_owned()),
            references: vec!["<root@example.com>".to_owned()],
        }
    }

    #[test]
    fn rfc2822_builder_sends_plain_only_messages_as_text_plain() {
        let message = build_rfc2822_message(&outgoing_email(None));

        assert!(message.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(!message.contains("multipart/alternative"));
        assert!(message.ends_with("Plain body"));
    }

    #[test]
    fn rfc2822_builder_preserves_html_body_as_multipart_alternative() {
        let message = build_rfc2822_message(&outgoing_email(Some(
            "<p><strong>Rich body</strong></p>".into(),
        )));

        assert!(message.contains("MIME-Version: 1.0\r\n"));
        assert!(message.contains("Content-Type: multipart/alternative; boundary=\"hermes-alt-"));
        assert!(message.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(message.contains("Content-Transfer-Encoding: 8bit\r\n\r\nPlain body\r\n"));
        assert!(message.contains("Content-Type: text/html; charset=utf-8\r\n"));
        assert!(message.contains(
            "Content-Transfer-Encoding: 8bit\r\n\r\n<p><strong>Rich body</strong></p>\r\n"
        ));
        assert!(message.contains("In-Reply-To: <parent@example.com>\r\n"));
        assert!(message.contains("References: <root@example.com>\r\n"));
    }
}
