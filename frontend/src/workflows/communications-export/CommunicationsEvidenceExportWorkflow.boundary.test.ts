import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Communications evidence export workflow boundary', () => {
	it('is app-composed, generated-contract-only and absent from the domain implementation graph', () => {
		const app = read('../../app/layout/AppLayoutRoot.vue')
		const route = read('../../domains/communications/views/CanonicalCommunicationsRoute.vue')
		const api = read('./api/communicationsEvidenceExport.ts')
		const controller = read('./queries/useCommunicationsEvidenceExport.ts')
		const presentation = read('./presentation/CommunicationsEvidenceExportPanel.vue')
		const generator = read('../../../scripts/generate-proto.mjs')

		expect(app).toContain('CommunicationsEvidenceExportWorkflow')
		expect(app).toContain("'communications.export.v1'")
		expect(route).toContain('canonicalMessageSelected')
		expect(route).not.toMatch(/workflows\/communications-export|communicationsEvidenceExport/)
		expect(api).toContain('getCommunicationsExportCommandClient')
		expect(api).toContain('getCommunicationsExportQueryClient')
		expect(api).toContain('getCommunicationsExportTicketClient')
		expect(api).toContain('BrowserGatewayFetch')
		expect(api).toContain('getBrowserGatewayRealtimeHub')
		expect(api).not.toMatch(/integrations\/(mail|telegram|whatsapp|zulip)|provider|blobRef/)
		expect(controller).toContain('crypto.getRandomValues')
		expect(controller).toContain('downloadBytesFile')
		expect(controller).toContain('openCommunicationsEvidenceExportRealtime')
		expect(controller).not.toMatch(/setInterval|setTimeout|poll/i)
		expect(presentation).not.toMatch(/api\/|connect\/|fetch\(|v-html/)
		expect(generator).toContain('communications-export-api')
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
