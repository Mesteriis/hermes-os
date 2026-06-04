use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use serde_json::json;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use hermes_hub_backend::email_provider_network::{
    GmailApiClient, GmailFetchOptions, ImapFetchOptions, ImapNetworkClient,
};
use hermes_hub_backend::email_sync::{
    EmailSyncBatch, FetchedEmailMessage, record_email_sync_batch,
};
use hermes_hub_backend::secrets::ResolvedSecret;
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn gmail_api_client_fetches_raw_messages_with_bearer_token() {
    let server = MockGmailServer::start();
    let token = ResolvedSecret::new("gmail-access-token").expect("token");
    let client = GmailApiClient::new(server.base_url()).user_id("me");

    let batch = client
        .fetch_raw_messages(&token, &GmailFetchOptions::new(2).query("is:unread"))
        .await
        .expect("fetch gmail messages");

    assert_eq!(batch.provider_kind, EmailProviderKind::Gmail);
    assert_eq!(batch.stream_id, "gmail:history");
    assert_eq!(
        batch.checkpoint,
        Some(json!({
            "provider": "gmail",
            "history_id": "12345",
            "next_page_token": "next-page"
        }))
    );
    assert_eq!(batch.messages.len(), 1);

    let message = &batch.messages[0];
    assert_eq!(message.provider_record_id, "gmail-msg-1");
    assert_eq!(
        message.occurred_at,
        Utc.timestamp_millis_opt(1_770_000_000_000).single()
    );
    assert!(message.source_fingerprint.starts_with("sha256:"));
    assert_eq!(message.payload["provider"], "gmail");
    assert_eq!(message.payload["thread_id"], "thread-1");
    assert_eq!(
        message.payload["raw_base64url"],
        "U3ViamVjdDogR21haWwNCg0KSGVsbG8"
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .request_line
            .starts_with("GET /gmail/v1/users/me/messages?")
    );
    assert!(requests[0].request_line.contains("maxResults=2"));
    assert!(requests[0].request_line.contains("q=is%3Aunread"));
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer gmail-access-token")
    );
    assert_eq!(
        requests[1].request_line,
        "GET /gmail/v1/users/me/messages/gmail-msg-1?format=raw HTTP/1.1"
    );
    assert_eq!(
        requests[1].authorization.as_deref(),
        Some("Bearer gmail-access-token")
    );
}

#[tokio::test]
async fn imap_network_client_fetches_raw_messages_by_uid_without_mutating_mailbox() {
    let server = MockImapServer::start();
    let password = ResolvedSecret::new("imap-password").expect("password");
    let client = ImapNetworkClient::new();
    let options = ImapFetchOptions::new(
        "127.0.0.1",
        server.addr().port(),
        false,
        "Archive",
        "imap-user@example.net",
    )
    .last_seen_uid(42)
    .max_messages(10);

    let batch = client
        .fetch_raw_messages(&password, &options)
        .await
        .expect("fetch IMAP messages");

    assert_eq!(batch.provider_kind, EmailProviderKind::Imap);
    assert_eq!(batch.stream_id, "imap:Archive");
    assert_eq!(
        batch.checkpoint,
        Some(json!({
            "provider": "imap",
            "mailbox": "Archive",
            "uid_validity": 999,
            "last_seen_uid": 43
        }))
    );
    assert_eq!(batch.messages.len(), 1);
    assert_eq!(batch.messages[0].provider_record_id, "43");
    assert_eq!(batch.messages[0].payload["provider"], "imap");
    assert_eq!(batch.messages[0].payload["mailbox"], "Archive");
    assert_eq!(
        batch.messages[0].payload["raw_rfc822_base64"],
        "U3ViamVjdDogSU1BUA0KDQpIZWxsbw=="
    );

    let commands = server.commands();
    assert!(
        commands
            .iter()
            .any(|command| command.contains("LOGIN") && command.contains("imap-user@example.net"))
    );
    assert!(
        commands
            .iter()
            .any(|command| command.contains("EXAMINE") && command.contains("Archive"))
    );
    assert!(
        commands
            .iter()
            .any(|command| command.contains("UID SEARCH 43:*"))
    );
    assert!(commands.iter().any(|command| {
        command.contains("UID FETCH 43 (UID BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
    }));
    for prohibited_command in ["SELECT", "STORE", "EXPUNGE", "COPY", "MOVE", "DELETE"] {
        assert!(
            !commands
                .iter()
                .any(|command| command.to_ascii_uppercase().contains(prohibited_command)),
            "IMAP client must not send mutating command `{prohibited_command}`: {commands:?}"
        );
    }
}

#[tokio::test]
async fn email_sync_records_provider_network_batch_against_postgres() {
    let Some((database, store, suffix)) = live_sync_context("provider network batch").await else {
        return;
    };

    let account_id = format!("acct_network_batch_{suffix}");
    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Network Gmail",
            format!("network-batch-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let batch = EmailSyncBatch {
        provider_kind: EmailProviderKind::Gmail,
        stream_id: "gmail:history".to_owned(),
        checkpoint: Some(json!({"provider": "gmail", "history_id": "12345"})),
        messages: vec![FetchedEmailMessage {
            provider_record_id: format!("gmail-network-message-{suffix}"),
            source_fingerprint: format!("sha256:gmail-network-message-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "gmail",
                "id": format!("gmail-network-message-{suffix}"),
                "raw_base64url": "U3ViamVjdA"
            }),
        }],
    };

    let report = record_email_sync_batch(
        &store,
        &account_id,
        &format!("provider-network-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("record provider network batch");

    assert_eq!(report.inserted_or_existing_records, 1);
    assert!(report.checkpoint_saved);

    let pool = database.pool().expect("configured pool");
    let raw_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM communication_raw_records
        WHERE account_id = $1
          AND provider_record_id = $2
          AND payload ->> 'provider' = 'gmail'
        "#,
    )
    .bind(&account_id)
    .bind(&batch.messages[0].provider_record_id)
    .fetch_one(pool)
    .await
    .expect("count raw record");
    assert_eq!(raw_count, 1);

    let checkpoint = store
        .checkpoint(&account_id, &batch.stream_id)
        .await
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(checkpoint.checkpoint["history_id"], "12345");
}

struct MockGmailServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<RecordedHttpRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockGmailServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock Gmail server");
        let addr = listener.local_addr().expect("mock Gmail addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept Gmail request");
                let request = read_http_request(&mut stream);
                let request_line = request.request_line.clone();
                requests_for_thread
                    .lock()
                    .expect("requests lock")
                    .push(request);

                let body = if request_line.starts_with("GET /gmail/v1/users/me/messages?") {
                    json!({
                        "messages": [{"id": "gmail-msg-1", "threadId": "thread-1"}],
                        "nextPageToken": "next-page",
                        "resultSizeEstimate": 1
                    })
                    .to_string()
                } else if request_line
                    == "GET /gmail/v1/users/me/messages/gmail-msg-1?format=raw HTTP/1.1"
                {
                    json!({
                        "id": "gmail-msg-1",
                        "threadId": "thread-1",
                        "labelIds": ["INBOX"],
                        "historyId": "12345",
                        "internalDate": "1770000000000",
                        "raw": "U3ViamVjdDogR21haWwNCg0KSGVsbG8"
                    })
                    .to_string()
                } else {
                    json!({"error": "unexpected request", "request": request_line}).to_string()
                };

                write_http_response(&mut stream, &body);
            }
        });

        Self {
            addr,
            requests,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> Vec<RecordedHttpRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl Drop for MockGmailServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("mock Gmail server join");
        }
    }
}

#[derive(Clone, Debug)]
struct RecordedHttpRequest {
    request_line: String,
    authorization: Option<String>,
}

fn read_http_request(stream: &mut TcpStream) -> RecordedHttpRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .expect("read request line");
    let request_line = request_line.trim_end().to_owned();
    let mut authorization = None;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read request header");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("authorization") {
                authorization = Some(value.trim().to_owned());
            }
        }
    }

    RecordedHttpRequest {
        request_line,
        authorization,
    }
}

fn write_http_response(stream: &mut TcpStream, body: &str) {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    )
    .expect("write response");
}

struct MockImapServer {
    addr: SocketAddr,
    commands: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockImapServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock IMAP server");
        let addr = listener.local_addr().expect("mock IMAP addr");
        let commands = Arc::new(Mutex::new(Vec::new()));
        let commands_for_thread = Arc::clone(&commands);
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept IMAP request");
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("set IMAP timeout");
            stream
                .write_all(b"* OK hermes test imap ready\r\n")
                .expect("write IMAP greeting");

            let mut reader = BufReader::new(stream.try_clone().expect("clone IMAP stream"));
            loop {
                let mut line = String::new();
                let bytes = reader.read_line(&mut line).expect("read IMAP command");
                if bytes == 0 {
                    break;
                }
                let command = line.trim_end().to_owned();
                commands_for_thread
                    .lock()
                    .expect("commands lock")
                    .push(command.clone());
                let tag = command
                    .split_whitespace()
                    .next()
                    .expect("tagged IMAP command")
                    .to_owned();

                if command.contains("LOGIN") {
                    writeln!(stream, "{tag} OK LOGIN completed\r").expect("write LOGIN response");
                } else if command.contains("EXAMINE") {
                    write!(
                        stream,
                        "* FLAGS (\\Seen)\r\n* 1 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 999] UIDs valid\r\n* OK [UIDNEXT 44] Predicted next UID\r\n{tag} OK [READ-ONLY] EXAMINE completed\r\n"
                    )
                    .expect("write EXAMINE response");
                } else if command.contains("UID SEARCH") {
                    write!(stream, "* SEARCH 43\r\n{tag} OK SEARCH completed\r\n")
                        .expect("write SEARCH response");
                } else if command.contains("UID FETCH") {
                    write!(
                        stream,
                        "* 1 FETCH (UID 43 RFC822.SIZE 22 INTERNALDATE \"04-Jun-2026 12:00:00 +0000\" BODY[] {{22}}\r\nSubject: IMAP\r\n\r\nHello)\r\n{tag} OK FETCH completed\r\n"
                    )
                    .expect("write FETCH response");
                } else if command.contains("LOGOUT") {
                    write!(stream, "* BYE logging out\r\n{tag} OK LOGOUT completed\r\n")
                        .expect("write LOGOUT response");
                    break;
                } else {
                    write!(stream, "{tag} BAD unexpected command\r\n").expect("write BAD response");
                }
            }
        });

        Self {
            addr,
            commands,
            handle: Some(handle),
        }
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }

    fn commands(&self) -> Vec<String> {
        self.commands.lock().expect("commands lock").clone()
    }
}

impl Drop for MockImapServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("mock IMAP server join");
        }
    }
}

async fn live_sync_context(
    test_name: &str,
) -> Option<(Database, CommunicationIngestionStore, u128)> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());

    Some((database, store, unique_suffix()))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
