use crate::{SettingsApplyState, SettingsDesiredSnapshot, SettingsSchemaBinding};

pub trait SettingsRegistryStore {
    type Error;

    fn admit_settings_schema(
        &self,
        binding: &SettingsSchemaBinding,
        schema_bytes: &[u8],
    ) -> Result<(), Self::Error>;
    fn settings_schema_artifact(
        &self,
        registration_id: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
    fn settings_schema_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<SettingsSchemaBinding>, Self::Error>;
    fn commit_desired_settings_snapshot(
        &self,
        update: &SettingsDesiredSnapshot,
    ) -> Result<u64, Self::Error>;
    fn desired_settings_snapshot(
        &self,
        registration_id: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, Self::Error>;
    fn transition_settings_apply_state(
        &self,
        registration_id: &str,
        revision: u64,
        next: SettingsApplyState,
        sanitized_reason_code: Option<&str>,
    ) -> Result<(), Self::Error>;
    fn confirm_effective_settings_revision(
        &self,
        registration_id: &str,
        revision: u64,
    ) -> Result<(), Self::Error>;
}
