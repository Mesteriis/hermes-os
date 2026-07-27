import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Mail operational active route boundary', () => {
	it('uses separate Mail read, sync, delivery and status contracts without domain coupling', () => {
		const route = read('../views/MailOperationalRoute.vue')
		const controller = read('../queries/useMailOperationalPage.ts')
		const readController = read('../queries/useMailOperationalRead.ts')
		const gateway = read('../api/mailOperationalGateway.ts')
		const readGateway = read('../api/mailOperationalReadGateway.ts')
		const readClient = read('../api/mailOperationalQueryClient.ts')
		const presentation = read('../presentation/MailOperationalPage.vue')
		const readPresentation = read('../presentation/MailOperationalReadPanel.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')

		for (const source of [
			route,
			controller,
			readController,
			gateway,
			readGateway,
			readClient,
			presentation,
			readPresentation,
		]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(telegram|whatsapp|zulip)/)
		}
		expect(gateway).toContain('getMailSyncConnectClient')
		expect(gateway).toContain('getMailDeliveryCommandConnectClient')
		expect(gateway).toContain('getMailDeliveryQueryConnectClient')
		expect(readClient).toContain('MailOperationalQueryService')
		expect(readGateway).toContain('MailOperationalQueryV1Schema')
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(readPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('MailOperationalRoute')
		expect(appLayout).toContain("'mail.delivery.v1'")
		expect(appLayout).toContain("'mail.operational.query.v1'")
		expect(appLayout).toContain("'mail.sync.v1'")
		expect(compiledAdapters).toContain("'mail-integration'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
