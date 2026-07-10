import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('IntegrationsSettings boundary', () => {
  it('renders provider setup only through Accounts while keeping orchestration in surfaces', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useIntegrationsSettingsSurface.ts', import.meta.url),
      'utf8'
    )
    const store = readFileSync(new URL('../stores/settings.ts', import.meta.url), 'utf8')
    const api = readFileSync(new URL('../api/settings.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./IntegrationsSettings.vue', import.meta.url))).toBe(false)
    expect(page).not.toContain('import IntegrationsSettings')
    expect(page).not.toContain('<IntegrationsSettings')
    expect(page).toContain("store.selectedSection === 'accounts'")
    expect(page).not.toContain("store.selectedSection === 'integrations'")
    expect(page).toContain('settings-service-row')
    expect(page).toContain('IntegrationConnectionWizard')
    expect(page).toContain('selectedAccountSummary.canRecoverExpiredCredential')
    expect(page).toContain('Восстановить аккаунт')
    expect(store).not.toContain("| 'integrations'")

    expect(surface).toContain('useProviderAccountsQuery')
    expect(surface).toContain('useCalendarAccountsQuery')
    expect(surface).toContain('useMailSyncSettingsQuery')
    expect(surface).toContain('useExportMailAccountSettingsMutation')
    expect(surface).toContain('useLogoutMailAccountMutation')
    expect(surface).toContain('useDeleteMailAccountMutation')
    expect(surface).toContain('useUpdateMailSyncSettingsMutation')
    expect(surface).toContain('useUpdateCalendarAccountMutation')
    expect(surface).toContain('useRunAddressBookSyncNowMutation')
    expect(surface).toContain('groups = computed')
    expect(surface).toContain('selectedAccountSummary = computed')
    expect(surface).toContain('accountCredentialRequiresReauthorization')
    expect(surface).toContain("account.credential_state?.status === 'expired'")
    expect(surface).toContain('openCredentialRecovery')
    expect(surface).toContain('serviceRowsForAccount')
    expect(surface).toContain('accountContactsSyncEnabled')
    expect(surface).toContain("accountConfigBoolean(account, 'address_book_sync_enabled') ?? true")
    expect(surface).toContain('handleToggleContactsService')
    expect(surface).toContain('handleRunSelectedServiceNow')
    expect(surface).toContain('handleEnableSelectedContactsBidirectional')
    expect(surface).toContain('address_book_sync_enabled')
    expect(surface).toContain('address_book_sync_direction')
    expect(surface).toContain('address_book_remote_write_enabled')
    expect(surface).toContain('Contacts sync reads provider contacts into Personas')
    expect(surface).toContain('Two-way sync with provider contacts is enabled')
    expect(api).toContain('address_book_sync_direction')
    expect(api).toContain('address_book_remote_write_enabled')
    expect(api).toContain('runAddressBookSyncNow')
    expect(api).toContain('/address-book-sync-now')
    expect(page).toContain('runSelectedServiceNow')
    expect(page).toContain('runSelectedServiceModeAction')
    expect(page).toContain('service.canRunNow')
    expect(page).toContain('service.canRunModeAction')
    expect(surface).not.toContain('Contacts service toggle is hidden until the Contacts API contract exists.')
    expect(surface).not.toContain('Contacts account API is not present in the current backend contracts.')
    expect(surface).not.toContain('Contacts sync is managed through account setup and backend configuration.')
    expect(surface).toContain("if (isTelegramProvider(providerKind)) return t('QR companion')")
    expect(surface).not.toContain('connectionModes = computed')
    expect(surface).not.toContain('operatorRoute = computed')
  })
})
