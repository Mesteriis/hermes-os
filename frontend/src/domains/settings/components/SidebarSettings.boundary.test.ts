import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('SidebarSettings boundary', () => {
  it('does not expose sidebar controls through SettingsPage', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const pageSurface = readFileSync(
      new URL('../queries/useSettingsPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./SidebarSettings.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarGroupEditor.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarItemEditor.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarNavigationList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./sidebar/SidebarSettingsSummary.vue', import.meta.url))).toBe(false)

    expect(page).not.toContain('import SidebarSettings')
    expect(page).not.toContain('<SidebarSettings')
    expect(page).not.toContain("store.selectedSection === 'sidebar'")
    expect(page).not.toContain('sidebarSettings.')
    expect(page).not.toContain('store.newSidebarGroupLabel')
    expect(page).not.toContain('store.updateNewSidebarGroupLabel')

    expect(pageSurface).not.toContain('useSidebarSettingsSurface')
    expect(pageSurface).not.toContain("id: 'sidebar'")
    expect(pageSurface).not.toContain("label: 'Interface'")
  })
})
