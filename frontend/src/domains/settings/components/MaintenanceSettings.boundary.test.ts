import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('MaintenanceSettings boundary', () => {
  it('keeps Maintenance as a Settings-owned backend surface with guarded actions', () => {
    const page = readFileSync(new URL('../views/SettingsPage.vue', import.meta.url), 'utf8')
    const store = readFileSync(new URL('../stores/settings.ts', import.meta.url), 'utf8')
    const pageSurface = readFileSync(new URL('../queries/useSettingsPageSurface.ts', import.meta.url), 'utf8')
    const surface = readFileSync(new URL('../queries/useMaintenanceSettingsSurface.ts', import.meta.url), 'utf8')
    const query = readFileSync(new URL('../queries/useMaintenanceQuery.ts', import.meta.url), 'utf8')
    const api = readFileSync(new URL('../api/maintenance.ts', import.meta.url), 'utf8')
    const panel = readFileSync(new URL('./MaintenanceSettingsPanel.vue', import.meta.url), 'utf8')
    const main = readFileSync(new URL('../../../main.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./MaintenanceSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(store).toContain("'maintenance'")
    expect(pageSurface).toContain('useMaintenanceSettingsSurface')
    expect(pageSurface).toContain("id: 'maintenance'")
    expect(pageSurface).toContain("label: 'Maintenance'")
    expect(page).toContain('import MaintenanceSettingsPanel')
    expect(page).toContain("store.selectedSection === 'maintenance'")
    expect(page).toContain(':surface="maintenanceSettings"')
    expect(main).toContain("import './styles/settings-maintenance.css'")

    expect(api).toContain('/api/v1/maintenance/overview')
    expect(api).toContain('/api/v1/maintenance/actions/')
    expect(query).toContain('useMutation')
    expect(query).toContain('invalidateQueries')
    expect(surface).toContain('confirmationDraft')
    expect(surface).toContain('canRunSelectedAction')
    expect(panel).toContain('settings-maintenance-confirm')
    expect(panel).toContain('surface.canRunSelectedAction')
    expect(panel).not.toContain('ApiClient')
    expect(panel).not.toMatch(/\bfetch\s*\(/)
  })
})
