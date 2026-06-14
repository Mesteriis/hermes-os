mod models;
mod provider;
mod runtime;

use super::super::models::DeclaredApplicationSetting;

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    let mut settings = Vec::new();
    settings.extend(provider::declared_settings());
    settings.extend(models::declared_settings());
    settings.extend(runtime::declared_settings());
    settings
}
