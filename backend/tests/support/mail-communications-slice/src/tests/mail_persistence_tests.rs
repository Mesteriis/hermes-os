use hermes_mail_persistence::{MailPersistence, PersistedMailConnection, PersistedMailOperation};
use hermes_mail_api::{DEFAULT_WINDOW, MAX_WINDOWS};

#[test]
fn stores_and_reads_connection_and_operation() {
    let mut persistence = MailPersistence::new();
    let connection = PersistedMailConnection {
        id: "conn-1".to_owned(),
        host: "mail.example.com".to_owned(),
        port: 993,
        username: "alice".to_owned(),
    };
    persistence.put_connection(connection.clone());
    assert_eq!(
        persistence.get_connection("conn-1").expect("conn"),
        &connection
    );

    let operation = PersistedMailOperation {
        operation_id: "op-1".to_owned(),
        window_size: DEFAULT_WINDOW,
    };
    persistence.put_operation(operation);
    assert_eq!(persistence.operation_count(), 1);
    assert_eq!(persistence.policy().max_sync_windows, MAX_WINDOWS);
}
