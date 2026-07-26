use hermes_telegram_call_media_contract::{
    TelegramCallDiscardContextV1, TelegramCallMediaContractError, TelegramCallSignalingMediaPort,
};
use hermes_telegram_calls_core::{
    TelegramCallCommand, TelegramCallFailureCategory, TelegramCallOperation,
    TelegramProviderCallState,
};
use hermes_telegram_calls_persistence::{TelegramCallsPersistence, TelegramCallsPersistenceError};
use hermes_telegram_tdlib::{TdlibError, TdlibRequest, TdlibResponse, TdlibTransport};

use crate::calls_client_port::{TelegramCallsCommandRuntime, TelegramCallsCommandRuntimeError};
use crate::{TelegramRuntime, TelegramRuntimeAdmission};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallExecutionError {
    Persistence,
    Admission,
}

impl<T: TdlibTransport> TelegramRuntime<T> {
    pub fn install_call_media_port(&mut self, port: Box<dyn TelegramCallSignalingMediaPort>) {
        self.call_media = Some(port);
    }

    pub fn has_call_signaling_media(&self) -> bool {
        self.call_media
            .as_ref()
            .is_some_and(|port| port.supported_protocol().is_ok())
    }

    pub fn call_admission(&self) -> Option<&TelegramRuntimeAdmission> {
        self.admission.as_ref()
    }

    pub fn resolve_own_provider_user_id(
        &mut self,
        correlation_id: &str,
    ) -> Result<String, TdlibError> {
        match self.transport.request(TdlibRequest::GetOwnUser {
            correlation_id: correlation_id.to_owned(),
        })? {
            TdlibResponse::OwnUser { provider_user_id } => Ok(provider_user_id),
            _ => Err(TdlibError::Protocol(
                "TDLib getMe returned an unexpected response".to_owned(),
            )),
        }
    }

    pub async fn execute_due_call_operations(
        &mut self,
        persistence: &TelegramCallsPersistence,
        account_id: &str,
        now_unix_seconds: u64,
        limit: u32,
    ) -> Result<Vec<TelegramCallOperation>, TelegramCallExecutionError> {
        let admission = self
            .admission
            .clone()
            .ok_or(TelegramCallExecutionError::Admission)?;
        persistence
            .fail_stale_accepted_call_operations(
                account_id,
                admission.runtime_generation,
                admission.grant_epoch,
                now_unix_seconds,
            )
            .await
            .map_err(|_| TelegramCallExecutionError::Persistence)?;
        let claimed = persistence
            .claim_accepted_call_operations(
                account_id,
                admission.runtime_generation,
                admission.grant_epoch,
                now_unix_seconds,
                limit,
            )
            .await
            .map_err(|_| TelegramCallExecutionError::Persistence)?;
        let mut results = Vec::with_capacity(claimed.len());

        for operation in claimed {
            let Some(command) = operation.command() else {
                results.push(
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            TelegramCallFailureCategory::Protocol,
                            now_unix_seconds,
                        )
                        .await
                        .map_err(|_| TelegramCallExecutionError::Persistence)?,
                );
                continue;
            };
            if let TelegramCallCommand::SetLocalMute {
                call_session_id,
                muted,
                ..
            } = &command
            {
                let applied = self
                    .call_media
                    .as_mut()
                    .ok_or(TelegramCallMediaContractError::Unavailable)
                    .and_then(|port| port.set_local_mute(call_session_id, *muted));
                let result = if applied.is_ok() {
                    persistence
                        .complete_local_mute_operation(
                            account_id,
                            &operation.operation_id,
                            now_unix_seconds,
                        )
                        .await
                } else {
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            TelegramCallFailureCategory::NotAvailable,
                            now_unix_seconds,
                        )
                        .await
                }
                .map_err(|_| TelegramCallExecutionError::Persistence)?;
                results.push(result);
                continue;
            }

            let request = match self.call_request(persistence, &command).await {
                Ok(request) => request,
                Err(failure_category) => {
                    results.push(
                        persistence
                            .fail_call_operation(
                                account_id,
                                &operation.operation_id,
                                failure_category,
                                now_unix_seconds,
                            )
                            .await
                            .map_err(|_| TelegramCallExecutionError::Persistence)?,
                    );
                    continue;
                }
            };
            let provider_result = self.transport.request(request);
            let saved = match provider_result {
                Ok(TdlibResponse::CallCreated {
                    operation_id,
                    tdlib_call_id,
                }) if operation_id == operation.operation_id => {
                    persistence
                        .mark_call_operation_awaiting_provider(
                            account_id,
                            &operation.operation_id,
                            Some(tdlib_call_id),
                            now_unix_seconds,
                        )
                        .await
                }
                Ok(TdlibResponse::Accepted { operation_id })
                    if operation_id == operation.operation_id =>
                {
                    persistence
                        .mark_call_operation_awaiting_provider(
                            account_id,
                            &operation.operation_id,
                            None,
                            now_unix_seconds,
                        )
                        .await
                }
                Ok(_) => {
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            TelegramCallFailureCategory::Protocol,
                            now_unix_seconds,
                        )
                        .await
                }
                Err(error) => {
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            provider_failure_category(&error),
                            now_unix_seconds,
                        )
                        .await
                }
            }
            .map_err(|_| TelegramCallExecutionError::Persistence)?;
            results.push(saved);
        }
        Ok(results)
    }

    async fn call_request(
        &self,
        persistence: &TelegramCallsPersistence,
        command: &TelegramCallCommand,
    ) -> Result<TdlibRequest, TelegramCallFailureCategory> {
        match command {
            TelegramCallCommand::InitiateAudio {
                operation_id,
                provider_user_id,
                ..
            } => Ok(TdlibRequest::CreateCall {
                operation_id: operation_id.clone(),
                provider_user_id: provider_user_id.clone(),
                protocol: self.call_protocol()?,
            }),
            TelegramCallCommand::AcceptAudio {
                operation_id,
                account_id,
                call_session_id,
            } => {
                let call = required_call(persistence, account_id, call_session_id).await?;
                Ok(TdlibRequest::AcceptCall {
                    operation_id: operation_id.clone(),
                    tdlib_call_id: call.tdlib_call_id,
                    protocol: self.call_protocol()?,
                })
            }
            TelegramCallCommand::Decline {
                operation_id,
                account_id,
                call_session_id,
            } => {
                let call = required_call(persistence, account_id, call_session_id).await?;
                Ok(discard_request(
                    operation_id,
                    call.tdlib_call_id,
                    TelegramCallDiscardContextV1 {
                        duration_seconds: 0,
                        connection_id: 0,
                    },
                ))
            }
            TelegramCallCommand::End {
                operation_id,
                account_id,
                call_session_id,
            } => {
                let call = required_call(persistence, account_id, call_session_id).await?;
                let context = if call.state == TelegramProviderCallState::MediaReady {
                    self.call_media
                        .as_ref()
                        .ok_or(TelegramCallFailureCategory::NotAvailable)?
                        .discard_context(call_session_id)
                        .map_err(|_| TelegramCallFailureCategory::NotAvailable)?
                } else {
                    TelegramCallDiscardContextV1 {
                        duration_seconds: 0,
                        connection_id: 0,
                    }
                };
                Ok(discard_request(operation_id, call.tdlib_call_id, context))
            }
            TelegramCallCommand::SetLocalMute { .. } => Err(TelegramCallFailureCategory::Protocol),
        }
    }

    fn call_protocol(
        &self,
    ) -> Result<
        hermes_telegram_call_media_contract::TelegramCallProtocolV1,
        TelegramCallFailureCategory,
    > {
        self.call_media
            .as_ref()
            .ok_or(TelegramCallFailureCategory::NotAvailable)?
            .supported_protocol()
            .map_err(|_| TelegramCallFailureCategory::NotAvailable)
    }
}

impl<T: TdlibTransport> TelegramCallsCommandRuntime for TelegramRuntime<T> {
    fn calls_media_available(&self) -> bool {
        self.has_call_signaling_media()
    }

    fn calls_fence(&self) -> Option<(u64, u64)> {
        self.call_admission()
            .map(|admission| (admission.runtime_generation, admission.grant_epoch))
    }

    fn owns_calls_account(&self, account_id: &str) -> bool {
        self.account(account_id).is_some()
    }

    fn resolve_call_owner_provider_identity(
        &mut self,
        correlation_id: &str,
    ) -> Result<String, TelegramCallsCommandRuntimeError> {
        self.resolve_own_provider_user_id(correlation_id)
            .map_err(|_| TelegramCallsCommandRuntimeError)
    }
}

async fn required_call(
    persistence: &TelegramCallsPersistence,
    account_id: &str,
    call_session_id: &str,
) -> Result<hermes_telegram_calls_core::TelegramCallSession, TelegramCallFailureCategory> {
    persistence
        .call(account_id, call_session_id)
        .await
        .map_err(persistence_failure_category)?
        .ok_or(TelegramCallFailureCategory::NotAvailable)
}

fn persistence_failure_category(
    _error: TelegramCallsPersistenceError,
) -> TelegramCallFailureCategory {
    TelegramCallFailureCategory::NotAvailable
}

fn discard_request(
    operation_id: &str,
    tdlib_call_id: i32,
    context: TelegramCallDiscardContextV1,
) -> TdlibRequest {
    TdlibRequest::DiscardCall {
        operation_id: operation_id.to_owned(),
        tdlib_call_id,
        is_disconnected: false,
        duration_seconds: context.duration_seconds,
        connection_id: context.connection_id,
    }
}

fn provider_failure_category(error: &TdlibError) -> TelegramCallFailureCategory {
    match error {
        TdlibError::Transport(_) => TelegramCallFailureCategory::Network,
        TdlibError::Protocol(_) => TelegramCallFailureCategory::Protocol,
        TdlibError::AuthenticationRequired => TelegramCallFailureCategory::Permission,
        TdlibError::RuntimeUnavailable => TelegramCallFailureCategory::NotAvailable,
    }
}
