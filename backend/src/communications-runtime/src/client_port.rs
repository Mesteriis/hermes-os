//! Exact contract dispatcher for Communications client-facing module requests.

use std::os::unix::net::UnixStream;
use std::sync::Arc;

use hermes_communications_persistence::CommunicationsDurablePersistence;
use hermes_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use hermes_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;

use crate::admission::{
    communications_content_read_contract_reference_v1,
    communications_content_ticket_contract_reference_v1,
    communications_query_contract_reference_v1,
};
use crate::content_blob_client_port::{
    CommunicationsContentBlobClientPortErrorV1, handle_module_content_blob_request_v1,
};
use crate::content_ticket_client_port::{
    CommunicationsContentTicketClientPortErrorV1, handle_module_content_ticket_request_v1,
};
use crate::content_ticket_store::CommunicationsContentTicketStoreV1;
use crate::query_client_port::{
    CommunicationsQueryClientPortErrorV1, handle_module_query_request_v1,
};
use crate::search_access::CommunicationsSearchAccessV1;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

pub async fn dispatch_module_client_request_v1(
    persistence: &CommunicationsDurablePersistence,
    tickets: &Arc<CommunicationsContentTicketStoreV1>,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    nested_dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: &ModuleClientRequestV1,
) -> ModuleClientResponseV1 {
    let encoded = request.encode_to_vec();
    let result = if request.contract.as_ref() == Some(&communications_query_contract_reference_v1())
    {
        handle_module_query_request_v1(
            persistence,
            search_access,
            control_channel,
            nested_dispatcher,
            &encoded,
        )
        .await
        .map_err(map_query_error)
    } else if request.contract.as_ref()
        == Some(&communications_content_ticket_contract_reference_v1())
    {
        handle_module_content_ticket_request_v1(persistence, tickets, &encoded)
            .await
            .map_err(map_ticket_error)
    } else if request.contract.as_ref()
        == Some(&communications_content_read_contract_reference_v1())
    {
        handle_module_content_blob_request_v1(persistence, tickets, &encoded)
            .await
            .map_err(map_blob_error)
    } else {
        return module_error(request.request_id, "REJECTED");
    };
    match result {
        Ok(bytes) => ModuleClientResponseV1::decode(bytes.as_slice())
            .ok()
            .filter(|response| {
                response.protocol_major == MODULE_CLIENT_PROTOCOL_MAJOR
                    && response.request_id == request.request_id
            })
            .unwrap_or_else(|| module_error(request.request_id, "UNAVAILABLE")),
        Err(error_code) => module_error(request.request_id, error_code),
    }
}

fn module_error(request_id: u64, error_code: &str) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
}

const fn map_query_error(error: CommunicationsQueryClientPortErrorV1) -> &'static str {
    match error {
        CommunicationsQueryClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsQueryClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

const fn map_ticket_error(error: CommunicationsContentTicketClientPortErrorV1) -> &'static str {
    match error {
        CommunicationsContentTicketClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsContentTicketClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

const fn map_blob_error(error: CommunicationsContentBlobClientPortErrorV1) -> &'static str {
    match error {
        CommunicationsContentBlobClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsContentBlobClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}
