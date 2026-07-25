import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('TimelinePage boundary', () => {
	it('keeps Timeline planned without a Communications compatibility projection', () => {
		const appSurfaceSource = readFileSync(new URL('../../../app/queries/useTimelineViewSurface.ts', import.meta.url), 'utf8')
		const surfaceSource = readFileSync(new URL('../queries/useTimelineSurface.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./TimelinePage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/TimelineStream.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/TimelineFilters.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../api/timeline.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../queries/useTimelineQuery.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../queries/useTimelinePageSurface.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../stores/timeline.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../types/timeline.ts', import.meta.url))).toBe(false)

		expect(appSurfaceSource).toContain('Timeline projection is not admitted yet.')
		expect(appSurfaceSource).toContain('No compatibility data path is retained')
    expect(surfaceSource).toContain("status: 'planned'")
    expect(surfaceSource).not.toContain('communications')
    expect(surfaceSource).not.toContain('useTimelinePageSurface')
  })
})
