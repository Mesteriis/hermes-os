use crate::{
    SettingsApplyState, SettingsConfigurationTarget, SettingsDesiredSnapshot,
    SettingsInitialSnapshot, SettingsSchemaBinding, SettingsSchemaTargetSuccessor,
};

pub trait SettingsRegistryStore {
    type Error;

    fn admit_settings_schema(
        &self,
        binding: &SettingsSchemaBinding,
        schema_bytes: &[u8],
    ) -> Result<(), Self::Error>;
    fn upgrade_settings_schema_with_successor(
        &self,
        expected: &SettingsSchemaBinding,
        successor: &SettingsSchemaBinding,
        schema_bytes: &[u8],
        target_successors: &[SettingsSchemaTargetSuccessor],
    ) -> Result<(), Self::Error>;
    fn settings_schema_artifact(
        &self,
        registration_id: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
    fn settings_schema_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<SettingsSchemaBinding>, Self::Error>;
    fn settings_configuration_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
    ) -> Result<Option<SettingsConfigurationTarget>, Self::Error>;
    fn settings_configuration_targets(
        &self,
        registration_id: &str,
    ) -> Result<Vec<SettingsConfigurationTarget>, Self::Error>;
    fn commit_desired_settings_snapshot(
        &self,
        update: &SettingsDesiredSnapshot,
    ) -> Result<u64, Self::Error>;
    fn materialize_initial_settings_snapshot(
        &self,
        update: &SettingsInitialSnapshot,
    ) -> Result<u64, Self::Error>;
    fn desired_settings_snapshot(
        &self,
        registration_id: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, Self::Error>;
    fn desired_settings_snapshot_for_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, Self::Error>;
    fn transition_settings_apply_state(
        &self,
        registration_id: &str,
        revision: u64,
        next: SettingsApplyState,
        sanitized_reason_code: Option<&str>,
    ) -> Result<(), Self::Error>;
    fn transition_settings_apply_state_for_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
        revision: u64,
        next: SettingsApplyState,
        sanitized_reason_code: Option<&str>,
    ) -> Result<(), Self::Error>;
    fn confirm_effective_settings_revision(
        &self,
        registration_id: &str,
        revision: u64,
    ) -> Result<(), Self::Error>;
    fn confirm_effective_settings_revision_for_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
        revision: u64,
    ) -> Result<(), Self::Error>;
}
