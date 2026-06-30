import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('ProjectsPage boundary', () => {
  it('preserves projects orchestration after removing the legacy ProjectsPage Vue layer', () => {
    const appViewSource = readFileSync(new URL('../../../app/views/ProjectsView.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useProjectsPageSurface.ts', import.meta.url), 'utf8')
    const storeSource = readFileSync(new URL('../stores/projects.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./ProjectsPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/ProjectsHero.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/ProjectsDashboard.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/ProjectsRail.vue', import.meta.url))).toBe(false)

    expect(appViewSource).toContain('Projects UI removed after logic extraction. Rebuild pending new design language.')
    expect(appViewSource).toContain('Projects logic is preserved')

    expect(surfaceSource).toContain('useProjectsQuery')
    expect(surfaceSource).toContain('useProjectQuery')
    expect(surfaceSource).toContain('useProjectsStore')
    expect(surfaceSource).toContain('relatedProjectSummaries')
    expect(surfaceSource).toContain('selectedProjectRecord')
    expect(surfaceSource).toContain('selectedProjectStats')
    expect(surfaceSource).toContain('formatNumber')
    expect(storeSource).toContain('projectStatusLabel')
    expect(storeSource).toContain('projectTimelineIcon')
    expect(storeSource).toContain('projectDocumentIcon')
    expect(storeSource).toContain('formatProjectDate')
    expect(storeSource).toContain('formatProjectDateTime')
  })
})
