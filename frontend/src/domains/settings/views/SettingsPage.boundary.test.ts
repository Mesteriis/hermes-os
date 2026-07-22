import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SettingsPage boundary', () => {
  it('is a section router and delegates section UIs to dedicated panels', () => {
    const page = readFileSync(new URL('./SettingsPage.vue', import.meta.url), 'utf8')

    expect(page).toContain('import AISettingsPanel')
    expect(page).toContain('import ApplicationSettingsPanel')
    expect(page).toContain('import BackgroundJobsSettingsPanel')
    expect(page).toContain('import LanguageSettingsPanel')
    expect(page).toContain('import IntegrationsSettingsPanel')
    expect(page).toContain('import CommunicationsSettingsPanel')
    expect(page).toContain('import MaintenanceSettingsPanel')
    expect(page).toContain('import SignalHubSettingsPanel')
    expect(page).toContain('import SettingsOverviewStrip')
    expect(page).toContain('import TraceLogsSettingsPanel')
    expect(page).toContain('import SettingsNavigationTree')

    expect(page).toContain("selectedSection === 'accounts'")
    expect(page).toContain("selectedSection === 'communications'")
    expect(page).toContain("selectedSection === 'application'")
    expect(page).toContain("selectedSection === 'background-jobs'")
    expect(page).toContain("selectedSection === 'logs-traces'")
    expect(page).toContain("selectedSection === 'maintenance'")
    expect(page).toContain("selectedSection === 'language'")
    expect(page).toContain("selectedSection === 'signal-hub'")
    expect(page).toContain("selectedSection === 'ai'")

    expect(page).not.toContain('runSelectedServiceNow')
    expect(page).not.toContain('runSelectedServiceModeAction')
    expect(page).not.toContain('selectedAccountSummary')
    expect(page).not.toContain('IntegrationConnectionWizard')
    expect(page).not.toContain('closeConnectWizard')
    expect(page).not.toContain('handleSaveSelectedAccountLabel')
    expect(page).not.toContain('openConnectWizard')
    expect(page).not.toContain('updateSelectedAccount')

    expect(page).toContain('<IntegrationsSettingsPanel')
    expect(page).toContain('<CommunicationsSettingsPanel')
    expect(page).toContain('<ApplicationSettingsPanel')
    expect(page).toContain('<BackgroundJobsSettingsPanel')
    expect(page).toContain('<TraceLogsSettingsPanel')
    expect(page).toContain('<MaintenanceSettingsPanel')
    expect(page).toContain('<LanguageSettingsPanel')
    expect(page).toContain('<SignalHubSettingsPanel')
    expect(page).toContain('<AISettingsPanel')
  })
})
