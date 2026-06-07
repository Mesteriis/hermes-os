use crate::domains::mail::core::ProviderAccount;
use crate::platform::settings::ApplicationSetting;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SettingsResponse {
    pub items: Vec<ApplicationSetting>,
}
#[derive(Serialize)]
pub struct AccountsResponse {
    pub items: Vec<ProviderAccount>,
}
#[derive(Deserialize)]
pub struct UpdateRequest {
    pub value: String,
}
