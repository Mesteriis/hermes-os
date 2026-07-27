//! Managed Gateway conformance for the Mail-owned operational read model.

use std::time::Duration;

use hermes_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSyncInboxRequestV1,
    client_contract::MailClientContractV1,
    operational::{MailOperationalQueryResponseV1, MailOperationalQueryV1},
};
use hermes_mail_runtime::client_port::{
    MailClientPortErrorV1, decode_module_response, encode_module_request,
};

use super::*;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

pub(super) fn assert_mail_operational_read(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let folders = query_operational(
        store,
        supervisor,
        mail,
        71,
        MailOperationalQueryV1::ListFolders {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 20,
        },
    );
    let MailOperationalQueryResponseV1::Folders(folders) = folders else {
        panic!("Mail folders query returned the wrong response")
    };
    assert_eq!(folders.items.len(), 1);
    let inbox = &folders.items[0];
    assert_eq!(inbox.connection_id, MAIL_ACCOUNT_ID);
    assert_eq!(inbox.folder_id, "INBOX");
    assert_eq!(inbox.total_messages, 1);
    assert_eq!(inbox.unread_messages, 1);

    let messages = query_operational(
        store,
        supervisor,
        mail,
        72,
        MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some(inbox.folder_id.clone()),
            provider_thread_id: None,
            cursor: None,
            limit: 1,
        },
    );
    let MailOperationalQueryResponseV1::Messages(messages) = messages else {
        panic!("Mail messages query returned the wrong response")
    };
    assert_eq!(messages.items.len(), 1);
    let summary = &messages.items[0];
    assert_eq!(summary.connection_id, MAIL_ACCOUNT_ID);
    assert_eq!(summary.folder_ids, ["INBOX"]);
    assert_eq!(
        summary.subject.as_deref(),
        Some("managed attachment evidence")
    );
    assert_eq!(summary.sender.as_deref(), Some("source@example.test"));
    assert_eq!(summary.recipients, ["owner@example.test"]);
    assert!(summary.has_plain_text);
    assert!(summary.has_attachments);
    assert!(summary.observation_anchor_id.iter().any(|byte| *byte != 0));
    let cursor = messages
        .next_cursor
        .clone()
        .expect("bounded Mail message page cursor");
    let provider_message_id = summary.provider_message_id.clone();

    let detail = query_operational(
        store,
        supervisor,
        mail,
        73,
        MailOperationalQueryV1::GetMessage {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            provider_message_id,
        },
    );
    let MailOperationalQueryResponseV1::Message(detail) = detail else {
        panic!("Mail message detail query returned the wrong response")
    };
    assert_eq!(detail.summary, *summary);

    assert_operational_response_is_private(store, supervisor, mail);
    assert_cross_account_query_is_rejected(store, supervisor, mail);
    assert_cursor_scope_is_enforced(store, supervisor, mail, &cursor);
    assert_stale_operational_generation_is_rejected(store, supervisor, mail);

    std::thread::sleep(Duration::from_secs(1));
    sync_mail(
        store,
        supervisor,
        mail,
        77,
        "managed-mail-operational-cursor-stale",
    );
    assert_stale_cursor_is_rejected(store, supervisor, mail, &cursor);
}

fn query_operational(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    query: MailOperationalQueryV1,
) -> MailOperationalQueryResponseV1 {
    let request = encode_module_request(request_id, &MailClientRequestV1::OperationalQuery(query))
        .expect("encode Mail operational query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route Mail operational query");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes)
            .expect("decode Mail operational query");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::OperationalQuery(response) = response else {
        panic!("Mail operational route returned the wrong response")
    };
    response
}

fn assert_operational_response_is_private(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        74,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: None,
            provider_thread_id: None,
            cursor: None,
            limit: 20,
        }),
    )
    .expect("encode Mail privacy query");
    let response = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route Mail privacy query");
    for forbidden in [
        b"managed-mail-imap-password".as_slice(),
        b"Content-Type: multipart/mixed".as_slice(),
        b"Y2xlYW4tcm9vbS1hdHRhY2htZW50".as_slice(),
        b"hermes-fixture--".as_slice(),
    ] {
        assert!(
            !response
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "Mail operational response exposed provider or credential bytes"
        );
    }
}

fn assert_cross_account_query_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        75,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListFolders {
            connection_id: "other-mail-account".to_owned(),
            cursor: None,
            limit: 20,
        }),
    )
    .expect("encode cross-account Mail query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route rejected cross-account Mail query");
    assert_eq!(
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert_runtime_active(supervisor, mail);
}

fn assert_cursor_scope_is_enforced(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    cursor: &str,
) {
    let request = encode_module_request(
        76,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: None,
            provider_thread_id: None,
            cursor: Some(cursor.to_owned()),
            limit: 1,
        }),
    )
    .expect("encode wrong-scope Mail cursor query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route rejected wrong-scope Mail cursor query");
    assert_eq!(
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert_runtime_active(supervisor, mail);
}

fn assert_stale_cursor_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    cursor: &str,
) {
    let request = encode_module_request(
        78,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some("INBOX".to_owned()),
            provider_thread_id: None,
            cursor: Some(cursor.to_owned()),
            limit: 1,
        }),
    )
    .expect("encode stale Mail cursor query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route rejected stale Mail cursor query");
    assert_eq!(
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert_runtime_active(supervisor, mail);
}

fn assert_stale_operational_generation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        79,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListFolders {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 20,
        }),
    )
    .expect("encode stale-generation Mail query");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation + 1,
        mail.grant_epoch,
        MailClientContractV1::OperationalQuery.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("stale Mail operational generation"),
        "managed runtime fence is stale"
    );
}

fn sync_mail(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: operation_id.to_owned(),
        }),
    )
    .expect("encode Mail sync for cursor invalidation");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::Sync,
        &request,
    )
    .expect("route Mail sync for cursor invalidation");
    let (_, response) = decode_module_response(MailClientContractV1::Sync, &bytes)
        .expect("decode Mail sync for cursor invalidation");
    assert_eq!(
        response,
        MailClientResponseV1::SyncInboxCompleted {
            operation_id: operation_id.to_owned(),
            observed_messages: 1,
        }
    );
}

fn route(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request: &[u8],
) -> Result<Vec<u8>, String> {
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        contract.capability_id(),
        request,
    );
    route_managed_client_request(store, &supervisor.relay_port(), &route)
}

fn assert_runtime_active(supervisor: &ManagedRuntimeSupervisor, mail: &StartedMailRuntime) {
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("observe Mail after rejected operational query"),
        "rejected Mail operational query must not terminate the managed runtime"
    );
}
