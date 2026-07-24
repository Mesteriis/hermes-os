//! Runs the encrypted Blob store behind the authenticated inherited channel.

use hermes_blob_runtime::{
    storage::BlobContentLifecycleStore,
    vault::{BlobVaultKeyLeaseAdapterV1, BlobVaultRouteContextV1},
};
use hermes_runtime_protocol::{
    v1::{
        BlobRuntimeConfigurationV1, BlobRuntimeControlRequestV1, BlobRuntimeControlResponseV1,
        BlobRuntimeStateV1, BlobRuntimeStatusV1, GetBlobRuntimeStatusRequestV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeReadyRequestV1,
        blob_runtime_control_request_v1::Operation,
        blob_runtime_control_response_v1::Result as ResponseResult,
        managed_runtime_control_request_v1::Operation as ManagedOperation,
    },
    validation::blob::{
        validate_blob_runtime_configuration, validate_blob_runtime_control_request,
        validate_blob_runtime_status,
    },
};
use hermes_vault_protocol::LeaseAudienceV1;
use prost::Message;

use super::{
    data::{BlobDataService, BlobDataSessionVerifierV1, PrivateBlobDataListener},
    framing::{read_frame, write_frame},
    handshake::{BlobRuntimeIdentity, authenticate},
    vault_route::InheritedBlobVaultRouteV1,
};

pub(crate) fn serve_inherited(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: BlobRuntimeConfigurationV1,
) -> Result<(), String> {
    validate_blob_runtime_configuration(&configuration)
        .map_err(|_| "Blob runtime configuration is invalid".to_owned())?;
    let (mut channel, identity) = authenticate(descriptor_bytes, settings_schema_bytes)?;
    let store = BlobContentLifecycleStore::open(
        std::path::Path::new(&configuration.data_dir),
        configuration.maximum_blob_bytes,
    )
    .map_err(|_| "Blob encrypted store is unavailable".to_owned())?;
    let listener =
        PrivateBlobDataListener::bind(std::path::Path::new(&configuration.data_socket_path))
            .map_err(|_| "Blob data socket is unavailable".to_owned())?;
    let data = data_service(&channel, &identity, &configuration, store)?;
    announce_ready(&mut channel, &identity)?;
    serve_status_and_data(channel, identity, configuration, listener, data)
}

fn announce_ready(
    channel: &mut std::os::unix::net::UnixStream,
    identity: &BlobRuntimeIdentity,
) -> Result<(), String> {
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Ready(ManagedRuntimeReadyRequestV1 {
            registration_id: identity.registration_id().to_owned(),
            runtime_generation: identity.runtime_generation(),
            grant_epoch: identity.grant_epoch(),
        })),
    };
    write_frame(channel, &request.encode_to_vec())
}

fn data_service(
    channel: &std::os::unix::net::UnixStream,
    identity: &BlobRuntimeIdentity,
    configuration: &BlobRuntimeConfigurationV1,
    store: BlobContentLifecycleStore,
) -> Result<BlobDataService<InheritedBlobVaultRouteV1>, String> {
    let public_key = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| "Blob Vault context is invalid".to_owned())?;
    let audience = LeaseAudienceV1::new(
        identity.registration_id().to_owned(),
        "blob".to_owned(),
        identity.runtime_generation(),
        identity.grant_epoch(),
    )
    .map_err(|_| "Blob Vault context is invalid".to_owned())?;
    let context = BlobVaultRouteContextV1::new(
        configuration.vault_instance_id.clone(),
        configuration.vault_runtime_generation,
        public_key,
        audience,
    )
    .map_err(|_| "Blob Vault context is invalid".to_owned())?;
    let route = InheritedBlobVaultRouteV1::new(
        channel
            .try_clone()
            .map_err(|_| "Blob inherited control channel is unavailable".to_owned())?,
    )
    .map_err(|_| "Blob inherited control channel is unavailable".to_owned())?;
    let verifier = BlobDataSessionVerifierV1::new(
        configuration.kernel_instance_id.clone(),
        identity.runtime_generation(),
        &configuration.kernel_authorization_public_key_sec1,
    )
    .map_err(|_| "Blob data session authority is invalid".to_owned())?;
    Ok(BlobDataService::new(
        store,
        verifier,
        BlobVaultKeyLeaseAdapterV1::new(route, context),
    ))
}

fn serve_status_and_data(
    mut channel: std::os::unix::net::UnixStream,
    identity: BlobRuntimeIdentity,
    configuration: BlobRuntimeConfigurationV1,
    listener: PrivateBlobDataListener,
    mut data: BlobDataService<InheritedBlobVaultRouteV1>,
) -> Result<(), String> {
    channel
        .set_write_timeout(Some(std::time::Duration::from_secs(2)))
        .map_err(|_| "Blob inherited control channel is unavailable".to_owned())?;
    loop {
        if let Some(mut client) = listener
            .accept()
            .map_err(|_| "Blob data socket is unavailable".to_owned())?
        {
            channel
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .map_err(|_| "Blob inherited control channel is unavailable".to_owned())?;
            let _ = data.serve_one(&mut client);
            continue;
        }
        channel
            .set_read_timeout(Some(std::time::Duration::from_millis(25)))
            .map_err(|_| "Blob inherited control channel is unavailable".to_owned())?;
        let response = read_frame(&mut channel)
            .and_then(|bytes| {
                BlobRuntimeControlRequestV1::decode(bytes.as_slice())
                    .map_err(|_| "Blob inherited control frame is invalid".to_owned())
            })
            .map(|request| response_for(request, &identity, &configuration))
            .unwrap_or_else(|error| {
                if error == "Blob inherited control channel is unavailable" {
                    return error_response("idle");
                }
                error_response("invalid_request")
            });
        if response.error_code == "idle" {
            continue;
        }
        write_frame(&mut channel, &response.encode_to_vec())?;
    }
}

fn response_for(
    request: BlobRuntimeControlRequestV1,
    identity: &BlobRuntimeIdentity,
    configuration: &BlobRuntimeConfigurationV1,
) -> BlobRuntimeControlResponseV1 {
    if validate_blob_runtime_control_request(&request).is_err() {
        return error_response("operation_not_available");
    }
    match request.operation {
        Some(Operation::GetStatus(GetBlobRuntimeStatusRequestV1 {})) => {
            status_response(identity, configuration)
        }
        None => error_response("operation_not_available"),
    }
}

fn status_response(
    identity: &BlobRuntimeIdentity,
    configuration: &BlobRuntimeConfigurationV1,
) -> BlobRuntimeControlResponseV1 {
    let status = BlobRuntimeStatusV1 {
        state: BlobRuntimeStateV1::Ready as i32,
        runtime_generation: identity.runtime_generation(),
        vault_runtime_generation: configuration.vault_runtime_generation,
        maximum_blob_bytes: configuration.maximum_blob_bytes,
        blocker_code: String::new(),
    };
    validate_blob_runtime_status(&status).expect("constant Blob runtime status is valid");
    BlobRuntimeControlResponseV1 {
        result: Some(ResponseResult::Status(status)),
        error_code: String::new(),
    }
}

fn error_response(error_code: &str) -> BlobRuntimeControlResponseV1 {
    BlobRuntimeControlResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}
