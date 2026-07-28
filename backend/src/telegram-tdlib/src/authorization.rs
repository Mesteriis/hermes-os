//! Long-lived TDLib authorization driver. Business lifecycle remains in telegram-core.

use std::{fmt, time::Duration};

use serde_json::Value;

use crate::{
    TdJsonClient, TdlibAuthorizationParameters, TdlibAuthorizationUpdate, TdlibError,
    check_authentication_password, check_database_encryption_key_request, close_session_request,
    parse_authorization_update, parse_qr_authorization_link, request_qr_code_authentication,
    set_tdlib_parameters_request,
};

#[derive(Clone, Eq, PartialEq)]
pub enum TdlibAuthorizationEvent {
    State(TdlibAuthorizationUpdate),
    QrLink(String),
}

impl fmt::Debug for TdlibAuthorizationEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(state) => formatter.debug_tuple("State").field(state).finish(),
            Self::QrLink(_) => formatter
                .debug_tuple("QrLink")
                .field(&"[redacted]")
                .finish(),
        }
    }
}

pub struct TdlibAuthorizationDriver {
    client: TdJsonClient,
    parameters: TdlibAuthorizationParameters,
    parameters_sent: bool,
    encryption_key_checked: bool,
    qr_requested: bool,
}

impl TdlibAuthorizationDriver {
    pub fn new(client: TdJsonClient, parameters: TdlibAuthorizationParameters) -> Self {
        Self {
            client,
            parameters,
            parameters_sent: false,
            encryption_key_checked: false,
            qr_requested: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), TdlibError> {
        self.client
            .send_json(&set_tdlib_parameters_request(&self.parameters)?)?;
        self.parameters_sent = true;
        Ok(())
    }

    pub fn poll(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<TdlibAuthorizationEvent>, TdlibError> {
        let timeout_seconds = timeout.as_secs_f64();
        let Some(payload) = self.client.receive_json(timeout_seconds)? else {
            return Ok(None);
        };
        self.handle_payload(payload).map(Some)
    }

    pub fn handle_payload(
        &mut self,
        payload: Value,
    ) -> Result<TdlibAuthorizationEvent, TdlibError> {
        let update = parse_authorization_update(&payload)?;
        match &update {
            TdlibAuthorizationUpdate::WaitingParameters if !self.parameters_sent => {
                self.initialize()?;
            }
            TdlibAuthorizationUpdate::WaitingEncryptionKey if !self.encryption_key_checked => {
                self.client
                    .send_json(&check_database_encryption_key_request(
                        self.parameters
                            .session_encryption_key
                            .as_deref()
                            .map(|value| value.as_slice()),
                    ))?;
                self.encryption_key_checked = true;
            }
            TdlibAuthorizationUpdate::Other(state)
                if matches!(
                    state.as_str(),
                    "authorizationStateWaitPhoneNumber" | "authorizationStateWaitCode"
                ) && !self.qr_requested =>
            {
                self.client.send_json(&request_qr_code_authentication())?;
                self.qr_requested = true;
            }
            TdlibAuthorizationUpdate::WaitingQrScan => {
                return Ok(TdlibAuthorizationEvent::QrLink(
                    parse_qr_authorization_link(&payload)?,
                ));
            }
            _ => {}
        }
        Ok(TdlibAuthorizationEvent::State(update))
    }

    pub fn submit_password(&self, password: &str) -> Result<(), TdlibError> {
        self.client
            .send_json(&check_authentication_password(password)?)
    }

    pub fn close(&self) -> Result<(), TdlibError> {
        self.client.send_json(&close_session_request())
    }

    pub fn into_client(self) -> TdJsonClient {
        self.client
    }

    pub fn into_transport(
        self,
        account_id: impl Into<String>,
    ) -> Result<crate::TdJsonTransport, TdlibError> {
        crate::TdJsonTransport::new(self.client, account_id)
    }
}
