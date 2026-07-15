mod ai;
mod communications;
mod frontend;
mod privacy;
mod server;
#[cfg(test)]
mod tests;
mod ui;

use super::models::DeclaredApplicationSetting;

pub(crate) fn declared_setting_keys() -> Vec<String> {
    declared_application_settings()
        .into_iter()
        .map(|setting| setting.setting_key.to_owned())
        .collect()
}

pub(crate) fn declared_setting(setting_key: &str) -> Option<DeclaredApplicationSetting> {
    declared_application_settings()
        .into_iter()
        .find(|setting| setting.setting_key == setting_key)
}

pub(crate) fn declared_application_settings() -> Vec<DeclaredApplicationSetting> {
    let mut settings = Vec::new();
    settings.extend(server::declared_settings());
    settings.extend(communications::declared_settings());
    settings.extend(frontend::declared_settings());
    settings.extend(privacy::declared_settings());
    settings.extend(ai::declared_settings());
    settings.extend(ui::declared_settings());
    settings
}
