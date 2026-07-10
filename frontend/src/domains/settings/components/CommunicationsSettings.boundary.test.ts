import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('Communications settings boundary', () => {
  it('keeps mail reliability controls in Settings and uses persisted provider contracts', () => {
    const page = readFileSync(new URL('../views/SettingsPage.vue', import.meta.url), 'utf8')
    const settingsStore = readFileSync(new URL('../stores/settings.ts', import.meta.url), 'utf8')
    const pageSurface = readFileSync(new URL('../queries/useSettingsPageSurface.ts', import.meta.url), 'utf8')
    const surface = readFileSync(
      new URL('../queries/useCommunicationsSettingsSurface.ts', import.meta.url),
      'utf8'
    )
    const panel = readFileSync(new URL('./CommunicationsSettingsPanel.vue', import.meta.url), 'utf8')

    expect(existsSync(new URL('./CommunicationsSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(settingsStore).toContain("'communications'")
    expect(pageSurface).toContain("id: 'communications'")
    expect(page).toContain('import CommunicationsSettingsPanel')
    expect(page).toContain("store.selectedSection === 'communications'")
    expect(page).toContain(':surface="communicationsSettings"')
    expect(surface).toContain('communications.mail.consecutive_failures_before_degraded')
    expect(surface).toContain('useMailSyncSettingsQuery')
    expect(surface).toContain('useUpdateMailSyncSettingsMutation')
    expect(surface).toContain('useSyncStatusesQuery')
    expect(panel).toContain("t('Mail')")
    expect(panel).toContain("t('Failures before degradation')")
  })
})
