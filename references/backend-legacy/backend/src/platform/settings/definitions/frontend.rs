mod appearance;
mod bootstrap;
mod layout;
mod state;

use super::super::models::DeclaredApplicationSetting;

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    let mut settings = Vec::new();
    settings.extend(bootstrap::declared_settings());
    settings.extend(layout::declared_settings());
    settings.extend(appearance::declared_settings());
    settings.extend(state::declared_settings());
    settings
}
