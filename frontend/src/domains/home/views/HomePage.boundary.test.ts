import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('HomePage boundary', () => {
	it('keeps Home planned without a Communications compatibility projection', () => {
		const appSurfaceSource = readFileSync(new URL('../../../app/queries/useHomeViewSurface.ts', import.meta.url), 'utf8')
		const surfaceSource = readFileSync(new URL('../queries/useHomeSurface.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./HomePage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeMetrics.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeWhatsNew.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomePriorities.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeUpcoming.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomePeopleTalked.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeSystemStatus.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeActiveProjects.vue', import.meta.url))).toBe(false)

    expect(existsSync(new URL('../api/home.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../queries/useHomeQuery.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../queries/useHomePageSurface.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../types/api.ts', import.meta.url))).toBe(false)

		expect(appSurfaceSource).toContain('Home projection is not admitted yet.')
		expect(appSurfaceSource).toContain('No compatibility data path is retained')
    expect(surfaceSource).toContain("status: 'planned'")
    expect(surfaceSource).not.toContain('communications')
    expect(surfaceSource).not.toContain('useHomePageSurface')
  })
})
