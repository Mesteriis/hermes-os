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

    expect(existsSync(new URL('./IntegrationsSettings.vue', import.meta.url))).toBe(false)
    expect(page).not.toContain('import IntegrationsSettings')
    expect(page).not.toContain('<IntegrationsSettings')
    expect(page).toContain("store.selectedSection === 'accounts'")
    expect(page).not.toContain("store.selectedSection === 'integrations'")
    expect(page).toContain('settings-service-row')
    expect(page).toContain('IntegrationConnectionWizard')
    expect(store).not.toContain("| 'integrations'")

    expect(surface).toContain('useProviderAccountsQuery')
    expect(surface).toContain('useCalendarAccountsQuery')
    expect(surface).toContain('useMailSyncSettingsQuery')
    expect(surface).toContain('useExportMailAccountSettingsMutation')
    expect(surface).toContain('useLogoutMailAccountMutation')
    expect(surface).toContain('useDeleteMailAccountMutation')
    expect(surface).toContain('useUpdateMailSyncSettingsMutation')
    expect(surface).toContain('useUpdateCalendarAccountMutation')
    expect(surface).toContain('groups = computed')
    expect(surface).toContain('selectedAccountSummary = computed')
    expect(surface).toContain('serviceRowsForAccount')
    expect(surface).toContain("if (isTelegramProvider(providerKind)) return t('QR companion')")
    expect(surface).not.toContain('connectionModes = computed')
    expect(surface).not.toContain('operatorRoute = computed')
  })
})
