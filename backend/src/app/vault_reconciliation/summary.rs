#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct HostVaultReconciliationSummary {
    pub(super) restored_accounts: usize,
    pub(super) restored_calendar_accounts: usize,
}
