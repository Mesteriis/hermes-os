//! Telegram-owned client subset available before provider authorization.

use std::os::unix::net::UnixStream;

use hermes_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use hermes_telegram_api::{
    TelegramAccount, TelegramAccountState, TelegramClientRequest, TelegramClientResponse,
    TelegramRuntimeState, validate_setup,
};
use hermes_telegram_persistence::TelegramDurablePersistence;

use crate::{
    TelegramRuntimeComposition,
    bootstrap::{
        TelegramProviderReconfigurationContextV1, credential_revisions,
        resolve_provider_setup_parameters,
    },
    client_port::{decode_module_request, encode_module_response},
    client_transport::TelegramClientTransportError,
};

pub(crate) struct TelegramConfigurationClientContextV1<'a, D> {
    pub(crate) runtime_available: bool,
    pub(crate) composition: &'a mut TelegramRuntimeComposition,
    pub(crate) authorization_status: Option<&'a hermes_telegram_api::TelegramAuthorizationStatus>,
    pub(crate) durable: &'a TelegramDurablePersistence,
    pub(crate) control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub(crate) dispatcher: &'a mut D,
    pub(crate) reconfiguration_context: &'a mut TelegramProviderReconfigurationContextV1,
}

pub(crate) async fn try_handle<D>(
    request: &[u8],
    context: TelegramConfigurationClientContextV1<'_, D>,
) -> Result<Option<Vec<u8>>, TelegramClientTransportError>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    let TelegramConfigurationClientContextV1 {
        runtime_available,
        composition,
        authorization_status,
        durable,
        control_channel,
        dispatcher,
        reconfiguration_context,
    } = context;
    let (request_id, contract, request) =
        decode_module_request(request).map_err(TelegramClientTransportError::Port)?;
    let response = match request {
        TelegramClientRequest::AuthorizationStatus => {
            TelegramClientResponse::AuthorizationStatus(authorization_status.cloned().unwrap_or(
                hermes_telegram_api::TelegramAuthorizationStatus {
                    state: "starting".to_owned(),
                    qr_link: None,
                    password_hint: None,
                },
            ))
        }
        TelegramClientRequest::SubmitAuthorizationPassword { password } => {
            composition.submit_password(&password).map_err(|error| {
                TelegramClientTransportError::Port(
                    crate::client_port::TelegramClientPortError::Provider(error),
                )
            })?;
            TelegramClientResponse::AuthorizationPasswordAccepted
        }
        TelegramClientRequest::ProvisionAccount { setup } if !runtime_available => {
            if setup.account_id != composition.configured_account_id()
                || validate_setup(&setup).is_err()
                || reconfiguration_context
                    .configuration_instance_id()
                    .trim()
                    .is_empty()
            {
                return Err(TelegramClientTransportError::Port(
                    crate::client_port::TelegramClientPortError::Protocol(
                        "Telegram configuration account is invalid".to_owned(),
                    ),
                ));
            }
            let (api_hash_revision, session_encryption_key_revision) =
                credential_revisions(&setup.credentials).map_err(|_| {
                    TelegramClientTransportError::Port(
                        crate::client_port::TelegramClientPortError::Protocol(
                            "Telegram credential binding is invalid".to_owned(),
                        ),
                    )
                })?;
            let parameters = resolve_provider_setup_parameters(
                control_channel,
                dispatcher,
                reconfiguration_context,
                api_hash_revision,
                session_encryption_key_revision,
            )
            .map_err(|_| TelegramClientTransportError::RuntimeUnavailable)?;
            let account = TelegramAccount {
                account_id: setup.account_id.clone(),
                display_name: setup.display_name.clone(),
                external_account_id: setup.external_account_id.clone(),
                state: TelegramAccountState::Provisioning,
                runtime_state: TelegramRuntimeState::Stopped,
                runtime_epoch: 0,
            };
            durable
                .upsert_account(&account, &setup.credentials)
                .await
                .map_err(|error| {
                    TelegramClientTransportError::Port(
                        crate::client_port::TelegramClientPortError::Persistence(error),
                    )
                })?;
            composition
                .begin_account_authorization(setup, parameters)
                .map_err(|error| {
                    TelegramClientTransportError::Port(
                        crate::client_port::TelegramClientPortError::Provider(error),
                    )
                })?;
            reconfiguration_context
                .bind_credential_revisions(api_hash_revision, session_encryption_key_revision);
            TelegramClientResponse::Account(account)
        }
        TelegramClientRequest::ListAccounts if !runtime_available => {
            TelegramClientResponse::Accounts(durable.accounts().await.map_err(|error| {
                TelegramClientTransportError::Port(
                    crate::client_port::TelegramClientPortError::Persistence(error),
                )
            })?)
        }
        TelegramClientRequest::GetAccount { account_id } if !runtime_available => {
            TelegramClientResponse::Account(
                durable
                    .account(&account_id)
                    .await
                    .map_err(|error| {
                        TelegramClientTransportError::Port(
                            crate::client_port::TelegramClientPortError::Persistence(error),
                        )
                    })?
                    .map(|(account, _)| account)
                    .ok_or_else(|| {
                        TelegramClientTransportError::Port(
                            crate::client_port::TelegramClientPortError::Protocol(
                                "Telegram account is unknown".to_owned(),
                            ),
                        )
                    })?,
            )
        }
        _ => return Ok(None),
    };
    encode_module_response(contract, request_id, &response)
        .map(Some)
        .map_err(TelegramClientTransportError::Port)
}
