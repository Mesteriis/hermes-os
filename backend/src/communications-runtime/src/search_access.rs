//! Runtime-only private Blob and owner-key access for derived Communications search.

use std::os::unix::net::UnixStream;

use hermes_blob_client::{BlobDataClient, request_managed_blob_session_v2};
use hermes_communications_api::CommunicationBodyBlobReferenceV1;
use hermes_communications_domain::COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1;
use hermes_managed_vault_client::owner_derived_key::{
    ManagedOwnerDerivedKeyContextV1, ensure_managed_owner_derived_key_v2,
};
use hermes_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobDataOperationV1, ManagedStorageRuntimeConfigurationV1},
};
use zeroize::Zeroizing;

use crate::{
    admission::{
        COMMUNICATIONS_BLOB_CAPABILITY_ID, COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID,
        COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION,
        COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS, COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID,
    },
    event_runtime::CommunicationsRuntimeAdmissionV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchAccessErrorV1 {
    Admission,
    Denied,
    Unavailable,
}

#[derive(Clone)]
pub struct CommunicationsSearchAccessV1 {
    key_context: ManagedOwnerDerivedKeyContextV1,
}

impl CommunicationsSearchAccessV1 {
    pub fn open(
        admission: &CommunicationsRuntimeAdmissionV1,
        storage: &ManagedStorageRuntimeConfigurationV1,
    ) -> Result<Self, CommunicationsSearchAccessErrorV1> {
        let vault_public_key_x25519 = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| CommunicationsSearchAccessErrorV1::Admission)?;
        if storage.vault_instance_id.is_empty()
            || storage.vault_runtime_generation == 0
            || admission.logical_owner_id != storage.logical_owner_id
            || admission.registration_id.is_empty()
            || admission.runtime_instance_id.is_empty()
            || admission.runtime_generation == 0
            || admission.grant_epoch == 0
        {
            return Err(CommunicationsSearchAccessErrorV1::Admission);
        }
        Ok(Self {
            key_context: ManagedOwnerDerivedKeyContextV1 {
                vault_instance_id: storage.vault_instance_id.clone(),
                vault_runtime_generation: storage.vault_runtime_generation,
                vault_public_key_x25519,
                logical_owner_id: admission.logical_owner_id.clone(),
                registration_id: admission.registration_id.clone(),
                runtime_instance_id: admission.runtime_instance_id.clone(),
                runtime_generation: admission.runtime_generation,
                grant_epoch: admission.grant_epoch,
            },
        })
    }

    pub fn ensure_index_key(
        &mut self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    ) -> Result<Zeroizing<Vec<u8>>, CommunicationsSearchAccessErrorV1> {
        control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CommunicationsSearchAccessErrorV1::Unavailable)?;
        let result = ensure_managed_owner_derived_key_v2(
            control_channel,
            dispatcher,
            &self.key_context,
            COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID,
            COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID,
            COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION,
            COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS,
        )
        .map_err(|_| CommunicationsSearchAccessErrorV1::Denied);
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CommunicationsSearchAccessErrorV1::Unavailable)?;
        result
    }

    pub fn read_admitted_body(
        &mut self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        blob: &CommunicationBodyBlobReferenceV1,
    ) -> Result<Vec<u8>, CommunicationsSearchAccessErrorV1> {
        let read_end = bounded_read_end(blob.declared_bytes)?;
        control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CommunicationsSearchAccessErrorV1::Unavailable)?;
        let result = (|| {
            let session = request_managed_blob_session_v2(
                control_channel,
                dispatcher,
                COMMUNICATIONS_BLOB_CAPABILITY_ID,
                BlobDataOperationV1::BlobDataOperationReadRangeV1,
                &blob.reference_id,
                blob.declared_bytes,
                1,
                None,
            )
            .map_err(|_| CommunicationsSearchAccessErrorV1::Denied)?;
            BlobDataClient::new(session.data_socket_path)
                .and_then(|client| {
                    client.read_range(session.grant, session.channel_binding, 0, read_end)
                })
                .map_err(|_| CommunicationsSearchAccessErrorV1::Unavailable)
        })();
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CommunicationsSearchAccessErrorV1::Unavailable)?;
        result
    }
}

fn bounded_read_end(declared_bytes: u64) -> Result<u64, CommunicationsSearchAccessErrorV1> {
    (1..=u64::try_from(COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1).expect("bounded constant"))
        .contains(&declared_bytes)
        .then_some(declared_bytes)
        .ok_or(CommunicationsSearchAccessErrorV1::Denied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_read_never_expands_past_the_search_document_limit() {
        assert_eq!(
            bounded_read_end(0),
            Err(CommunicationsSearchAccessErrorV1::Denied)
        );
        assert_eq!(bounded_read_end(1), Ok(1));
        assert_eq!(
            bounded_read_end(
                u64::try_from(COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1).expect("limit") + 1
            ),
            Err(CommunicationsSearchAccessErrorV1::Denied),
        );
    }
}
