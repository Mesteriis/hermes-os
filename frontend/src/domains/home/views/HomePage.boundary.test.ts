import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('HomePage boundary', () => {
	it('preserves home dashboard derivations after removing the legacy HomePage Vue layer', () => {
		const appSurfaceSource = readFileSync(new URL('../../../app/queries/useHomeViewSurface.ts', import.meta.url), 'utf8')
		const surfaceSource = readFileSync(new URL('../queries/useHomePageSurface.ts', import.meta.url), 'utf8')
		const querySource = readFileSync(new URL('../queries/useHomeQuery.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./HomePage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeMetrics.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeWhatsNew.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomePriorities.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeUpcoming.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomePeopleTalked.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeSystemStatus.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/HomeActiveProjects.vue', import.meta.url))).toBe(false)

		expect(appSurfaceSource).toContain('Home UI removed after logic extraction. Rebuild pending new design language.')
		expect(appSurfaceSource).toContain('Home logic is preserved')

    expect(surfaceSource).toContain('channelIcons')
    expect(surfaceSource).toContain('useCommunicationMessagesQuery(50)')
    expect(surfaceSource).toContain('useMailboxHealthQuery()')
    expect(surfaceSource).toContain('homeStats')
    expect(surfaceSource).toContain('whatsNew')
    expect(surfaceSource).toContain('personasTalked')
    expect(surfaceSource).toContain("seen = new Set<string>()")
    expect(querySource).toContain('useCommunicationMessagesQuery')
    expect(querySource).toContain('useMailboxHealthQuery')
  })
})
