import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('navigation communication source routing boundary', () => {
	it('routes provider communication sections through the communications route query', () => {
		const source = readFileSync(new URL('./navigation.ts', import.meta.url), 'utf8')
		const shellSource = readFileSync(
			new URL('../../app/shell/AppShell.vue', import.meta.url),
			'utf8'
		)
		const shellSurfaceSource = readFileSync(
			new URL('../../app/queries/useAppShellSurface.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain("router.push({ name: 'communications', query: { section: sectionId } })")
		expect(source).toContain('communicationSectionFromQuery(sectionQuery)')
		expect(shellSource).toContain('const { nav, theme } = useAppShellSurface()')
		expect(shellSurfaceSource).toContain('route.query.section')
		expect(shellSurfaceSource).toContain('nav.syncFromRoute(')
		expect(source).not.toContain('router.push(`/${routeViewId}`)')
		expect(source).not.toContain("type RouteViewId = AppViewId | 'telegram' | 'whatsapp'")
	})
})
