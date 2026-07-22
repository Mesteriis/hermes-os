import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('IntegrationsSettings boundary', () => {
  it('renders provider setup only through Accounts while keeping orchestration in surfaces', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const panel = readFileSync(
      new URL('./IntegrationsSettingsPanel.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useIntegrationsSettingsSurface.ts', import.meta.url),
      'utf8'
    )
    const presentation = readFileSync(
      new URL('../queries/integrationAccountPresentation.ts', import.meta.url),
      'utf8'
    )
    const predicates = readFileSync(
      new URL('../queries/integrationAccountPredicates.ts', import.meta.url),
      'utf8'
    )
    const contactsActions = readFileSync(
      new URL('../queries/integrationContactsActions.ts', import.meta.url),
      'utf8'
    )
    const accountActions = readFileSync(
      new URL('../queries/integrationAccountActions.ts', import.meta.url),
      'utf8'
    )
    const mailActions = readFileSync(
      new URL('../queries/integrationMailActions.ts', import.meta.url),
      'utf8'
    )
    const calendarActions = readFileSync(
      new URL('../queries/integrationCalendarActions.ts', import.meta.url),
      'utf8'
    )
    const accountToggleActions = readFileSync(
      new URL('../queries/integrationAccountToggleActions.ts', import.meta.url),
      'utf8'
    )
    const store = readFileSync(new URL('../stores/settings.ts', import.meta.url), 'utf8')
    const api = readFileSync(new URL('../api/settings.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./IntegrationsSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(page).toContain('import IntegrationsSettingsPanel')
    expect(page).toContain('<IntegrationsSettingsPanel')
    expect(page).toContain("selectedSection === 'accounts'")
    expect(page).not.toContain("selectedSection === 'integrations'")
    expect(page).not.toContain('updateSelectedAccount')
    expect(page).not.toContain('updateSelectedAccountLabel')
    expect(page).not.toContain('saveSelectedAccountLabel')
    expect(page).not.toContain('updateSelectedService')
    expect(page).not.toContain('runSelectedServiceNow')
    expect(page).not.toContain('runSelectedServiceModeAction')
    expect(page).not.toContain('IntegrationConnectionWizard')
    expect(page).not.toContain('selectedAccountSummary.canRecoverExpiredCredential')
    expect(page).not.toContain('Восстановить аккаунт')
    expect(store).not.toContain("| 'integrations'")

    expect(panel).toContain('handleAddAccount')
    expect(panel).toContain('handleSelectAccount')
    expect(panel).toContain('handleToggleSelectedAccount')
    expect(panel).toContain('handleUpdateSelectedAccountLabel')
    expect(panel).toContain('handleSaveSelectedAccountLabel')
    expect(panel).toContain('handleToggleSelectedService')
    expect(panel).toContain('handleRunSelectedServiceNow')
    expect(panel).toContain('handleRunSelectedServiceModeAction')
    expect(panel).toContain('handleOpenCredentialRecovery')
    expect(panel).toContain('handleOpenSelectedServiceSetup')
    expect(panel).toContain('handleExportAccount')
    expect(panel).toContain('handleLogoutAccount')
    expect(panel).toContain('handleDeleteAccount')
    expect(panel).toContain('IntegrationConnectionWizard')
    expect(panel).toContain('handleCloseConnectWizard')
    expect(panel).toContain('isConnectWizardOpen')
    expect(panel).toContain('connectWizardProviderId')
    expect(panel).toContain('connectWizardSelectedAccount')
    expect(panel).toContain('settings-service-row')
    expect(panel).toContain('selectedAccountSummary.canRecoverExpiredCredential')
    expect(panel).toContain('Восстановить аккаунт')

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
    expect(surface).toContain('accountCredentialRequiresReauthorization(')
    expect(predicates).toContain("account.credential_state?.status === 'expired'")
    expect(surface).toContain('openCredentialRecovery')
    expect(surface).toContain('toIntegrationServiceRows')
    expect(presentation).toContain('accountContactsSyncEnabled(')
    expect(predicates).toContain("accountConfigBoolean(account, 'address_book_sync_enabled') ?? true")
    expect(surface).toContain('handleToggleContactsService')
    expect(surface).toContain('handleRunSelectedServiceNow')
    expect(surface).toContain('handleEnableSelectedContactsBidirectional')
    expect(surface).toContain('contactsActionDependencies')
    expect(surface).toContain('accountActionDependencies')
    expect(accountActions).toContain('saveAccountLabel')
    expect(accountActions).toContain('exportMailAccount')
    expect(accountActions).toContain('logoutMailAccount')
    expect(accountActions).toContain('deleteMailAccount')
    expect(accountActions).toContain('clearSelectedAccount')
    expect(mailActions).toContain('MailSyncSettingsUpdate')
    expect(mailActions).toContain('normalizeMailSyncSettingsValues')
    expect(calendarActions).toContain("sync_status: enabled ? 'active' : 'paused'")
    expect(accountToggleActions).toContain('toggleSelectedAccount')
    expect(accountToggleActions).toContain('defaultProviderIdFromAccount')
    expect(accountToggleActions).toContain('logoutMailAccount')
    expect(contactsActions).toContain('address_book_sync_enabled')
    expect(contactsActions).toContain('address_book_sync_direction')
    expect(contactsActions).toContain('address_book_remote_write_enabled')
    expect(contactsActions).toContain('ensureContactsAvailable')
    expect(contactsActions).toContain('runAddressBookSyncNow')
    expect(presentation).toContain('Contacts sync reads provider contacts into Personas')
    expect(presentation).toContain('Two-way sync with provider contacts is enabled')
    expect(api).toContain('address_book_sync_direction')
    expect(api).toContain('address_book_remote_write_enabled')
    expect(api).toContain('runAddressBookSyncNow')
    expect(api).toContain('/address-book-sync-now')
    expect(panel).toContain('service.canRunNow')
    expect(panel).toContain('service.canRunModeAction')
    expect(surface).not.toContain('Contacts service toggle is hidden until the Contacts API contract exists.')
    expect(surface).not.toContain('Contacts account API is not present in the current backend contracts.')
    expect(surface).not.toContain('Contacts sync is managed through account setup and backend configuration.')
    expect(presentation).toContain("if (isTelegramProvider(providerKind)) return t('QR companion')")
    expect(surface).not.toContain('connectionModes = computed')
    expect(surface).not.toContain('operatorRoute = computed')
  })
})
