import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('IntegrationsSettings boundary', () => {
  it('preserves integration orchestration in surfaces after removing the Vue render layer', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useIntegrationsSettingsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./IntegrationsSettings.vue', import.meta.url))).toBe(false)
    expect(page).not.toContain('import IntegrationsSettings')
    expect(page).not.toContain('<IntegrationsSettings')
    expect(page).toContain('Component removed after logic extraction. Rebuild will land in the next UI pass.')
    expect(page).toContain('Integration logic is preserved')

    expect(surface).toContain('useProviderAccountsQuery')
    expect(surface).toContain('useExportMailAccountSettingsMutation')
    expect(surface).toContain('useLogoutMailAccountMutation')
    expect(surface).toContain('useDeleteMailAccountMutation')
    expect(surface).toContain('groups = computed')
    expect(surface).toContain('selectedAccountSummary = computed')
    expect(surface).toContain("label: 'Browser callbacks'")
    expect(surface).toContain("label: 'QR companion'")
    expect(surface).not.toContain('operatorRoute = computed')
  })
})
