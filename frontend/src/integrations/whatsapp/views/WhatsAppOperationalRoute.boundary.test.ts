import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('WhatsApp operational active route boundary', () => {
	it('keeps browser operations integration-owned and host execution isolated', () => {
		const route = read('../views/WhatsAppOperationalRoute.vue')
		const controller = read('../queries/useWhatsAppOperationalPage.ts')
		const gateway = read('../api/whatsappOperationalGateway.ts')
		const presentation = read('../presentation/WhatsAppOperationalPage.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')

		for (const source of [route, controller, gateway, presentation]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(mail|telegram|zulip)/)
			expect(source).not.toMatch(/invoke\(|@tauri-apps/)
		}
		expect(gateway).toContain('getWhatsAppCommandConnectClient')
		expect(gateway).toContain('getWhatsAppQueryConnectClient')
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('WhatsAppOperationalRoute')
		expect(appLayout).toContain("'whatsapp.command.v1'")
		expect(compiledAdapters).toContain("'whatsapp-integration'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
