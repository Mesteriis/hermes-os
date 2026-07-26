import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('canonical Communications active route boundary', () => {
	it('uses only the owner presentation and generated owner query adapter', () => {
		const route = read('../views/CanonicalCommunicationsRoute.vue')
		const controller = read('../queries/useCanonicalCommunicationsPage.ts')
		const readAdapter = read('../queries/canonicalCommunicationsRead.ts')
		const searchAdapter = read('../queries/canonicalCommunicationsSearch.ts')
		const presentation = read('../presentation/CanonicalCommunicationsPage.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')

		for (const source of [route, controller, readAdapter, searchAdapter, presentation]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/integrations\/(mail|telegram|whatsapp|zulip)/)
			expect(source).not.toMatch(/components\/(mail|messengers)/)
		}
		expect(readAdapter).toContain('getCommunicationsQueryConnectClient')
		expect(searchAdapter).toContain('getCommunicationsQueryConnectClient')
		expect(presentation).not.toMatch(/queries\/|connect\/|fetch\(/)
		expect(appLayout).toContain('CanonicalCommunicationsRoute')
		expect(compiledAdapters).toContain("'communications-owner'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
