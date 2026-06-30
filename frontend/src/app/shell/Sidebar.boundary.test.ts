import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Sidebar boundary', () => {
  it('delegates sidebar routing helpers to an app-level surface', () => {
    const source = readFileSync(new URL('./Sidebar.vue', import.meta.url), 'utf8')

    expect(source).toContain("import { useSidebarSurface } from '../queries/useSidebarSurface'")
    expect(source).toContain('const {')
    expect(source).toContain('isCommunicationItemActive')
    expect(source).not.toContain('useNavigationStore')
    expect(source).not.toContain('useSidebarStore')
    expect(source).not.toContain("nav.currentView === 'communications'")
    expect(source).not.toContain('nav.activeCommunicationSection === sectionId')
  })
})
