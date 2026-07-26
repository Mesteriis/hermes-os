import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Telegram operational active route boundary', () => {
	it('stays integration-owned and uses only the generated Telegram contract', () => {
		const route = read('../views/TelegramOperationalRoute.vue')
		const controller = read('../queries/useTelegramOperationalPage.ts')
		const gateway = read('../api/telegramOperationalGateway.ts')
		const presentation = read('../presentation/TelegramOperationalPage.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')
		const capabilityComposition = read('../../../app/client-surfaces/clientModuleCapabilities.ts')

		for (const source of [route, controller, gateway, presentation]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(mail|whatsapp|zulip)/)
		}
		expect(gateway).toContain('getTelegramOperationalConnectClient')
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('TelegramOperationalRoute')
		expect(appLayout).toContain("'telegram.command.v1'")
		expect(capabilityComposition).toContain('module.sectionsEnabled')
		expect(compiledAdapters).toContain("'telegram-integration'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
