import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('canonical Communications active route boundary', () => {
	it('uses only the owner presentation and generated owner query adapter', () => {
		const route = read('../views/CanonicalCommunicationsRoute.vue')
		const controller = read('../queries/useCanonicalCommunicationsPage.ts')
		const detailController = read('../queries/useCanonicalCommunicationDetail.ts')
		const contentController = read('../queries/useCanonicalCommunicationContent.ts')
		const readAdapter = read('../queries/canonicalCommunicationsRead.ts')
		const detailAdapter = read('../queries/canonicalCommunicationsDetail.ts')
		const contentAdapter = read('../queries/canonicalCommunicationsContent.ts')
		const searchAdapter = read('../queries/canonicalCommunicationsSearch.ts')
		const presentation = read('../presentation/CanonicalCommunicationsPage.vue')
		const detailPresentation = read('../presentation/CanonicalCommunicationDetail.vue')
		const contentPresentation = read('../presentation/CanonicalCommunicationContent.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')

		for (const source of [
			route,
			controller,
			detailController,
			contentController,
			readAdapter,
			detailAdapter,
			contentAdapter,
			searchAdapter,
			presentation,
			detailPresentation,
			contentPresentation,
		]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/integrations\/(mail|telegram|whatsapp|zulip)/)
			expect(source).not.toMatch(/components\/(mail|messengers)/)
		}
		expect(readAdapter).toContain('getCommunicationsQueryConnectClient')
		expect(searchAdapter).toContain('getCommunicationsQueryConnectClient')
		expect(presentation).not.toMatch(/queries\/|connect\/|fetch\(/)
		expect(detailPresentation).not.toMatch(/queries\/|connect\/|fetch\(/)
		expect(contentPresentation).not.toMatch(/queries\/|connect\/|fetch\(|v-html/)
		expect(detailAdapter).toContain('getCanonicalMessage')
		expect(detailAdapter).not.toContain('getCommunicationsQueryConnectClient')
		expect(contentAdapter).toContain('getCommunicationsContentTicketConnectClient')
		expect(contentAdapter).toContain('BrowserGatewayFetch')
		expect(contentAdapter).not.toMatch(/provider|integrations\/(mail|telegram|whatsapp|zulip)/)
		expect(detailController).not.toContain('canonicalCommunicationsContent')
		expect(route).toContain('useCanonicalCommunicationContent')
		expect(appLayout).toContain('CanonicalCommunicationsRoute')
		expect(compiledAdapters).toContain("'communications-owner'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
