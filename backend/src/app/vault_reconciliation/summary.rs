#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct HostVaultReconciliationSummary {
    pub(super) restored_accounts: usize,
    pub(super) restored_calendar_accounts: usize,
    pub(super) restored_ai_providers: usize,
    pub(super) skipped_duplicate_provider_secrets: usize,
    pub(super) purged_duplicate_provider_secrets: usize,
}
