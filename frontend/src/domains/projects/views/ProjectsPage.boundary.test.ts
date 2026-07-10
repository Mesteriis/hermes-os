import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('ProjectsPage boundary', () => {
	it('preserves projects orchestration after removing the legacy ProjectsPage Vue layer', () => {
		const appSurfaceSource = readFileSync(new URL('../../../app/queries/useProjectsViewSurface.ts', import.meta.url), 'utf8')
		const surfaceSource = readFileSync(new URL('../queries/useProjectsPageSurface.ts', import.meta.url), 'utf8')
		const storeSource = readFileSync(new URL('../stores/projects.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./ProjectsPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/ProjectsHero.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/ProjectsDashboard.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/ProjectsRail.vue', import.meta.url))).toBe(false)

		expect(appSurfaceSource).toContain('Projects UI removed after logic extraction. Rebuild pending new design language.')
		expect(appSurfaceSource).toContain('Projects logic is preserved')

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

  it('keeps project detail Persona-native while retaining deprecated people aliases', () => {
    const typeSource = readFileSync(new URL('../types/project.ts', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useProjectsPageSurface.ts', import.meta.url), 'utf8')

    expect(typeSource).toContain('persona_count: number')
    expect(typeSource).toContain('key_personas: ProjectPersonaSummary[]')
    expect(typeSource).toContain('export interface ProjectPersonaSummary')
    expect(typeSource).toContain('@deprecated Use persona_count.')
    expect(typeSource).toContain('@deprecated Use key_personas.')
    expect(surfaceSource).toContain('persona_count: 0')
  })
})
