import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('SidebarSettings boundary', () => {
  it('preserves sidebar orchestration after removing the Vue render layer', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useSidebarSettingsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./SidebarSettings.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarGroupEditor.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarItemEditor.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarNavigationList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarSettingsSummary.vue', import.meta.url))).toBe(false)

    expect(page).not.toContain('import SidebarSettings')
    expect(page).not.toContain('<SidebarSettings')
    expect(page).toContain('Sidebar UI removed after logic extraction. Rebuild pending new design language.')
    expect(page).toContain('Sidebar logic is preserved')

    expect(surface).toContain('useSidebarStore')
    expect(surface).toContain('useSettingsStore')
    expect(surface).toContain('useSaveFrontendSidebarMutation')
    expect(surface).toContain('sidebarRuleSummaries')
    expect(surface).toContain('handleSaveSidebar')
    expect(surface).toContain('sidebar.addSidebarGroup')
    expect(surface).not.toContain('useSidebarSettingsController')
    expect(surface).not.toContain('../api/')
    expect(surface).not.toContain('saveApplicationSetting')
    expect(surface).not.toContain('FRONTEND_SIDEBAR_SETTING_KEY')
    expect(surface).not.toContain('queryClient')
    expect(surface).not.toContain('fetch(')
  })
})
