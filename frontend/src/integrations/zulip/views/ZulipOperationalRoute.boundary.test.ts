import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Zulip operational active route boundary', () => {
	it('keeps provider commands in the Zulip integration', () => {
		const route = read('../views/ZulipOperationalRoute.vue')
		const controller = read('../queries/useZulipOperationalPage.ts')
		const gateway = read('../api/zulipOperationalGateway.ts')
		const presentation = read('../presentation/ZulipOperationalPage.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')

		for (const source of [route, controller, gateway, presentation]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(mail|telegram|whatsapp)/)
			expect(source).not.toMatch(/invoke\(|@tauri-apps/)
		}
		expect(gateway).toContain('getZulipCommandConnectClient')
		expect(gateway).toContain('getZulipQueryConnectClient')
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('ZulipOperationalRoute')
		expect(appLayout).toContain("'zulip.command.v1'")
		expect(compiledAdapters).toContain("'zulip-integration'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
