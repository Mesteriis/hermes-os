import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Attachment Preview workflow boundary', () => {
	it('is app-composed, generated-contract-only and consumes the shared SSE stream without polling', () => {
		const app = read('../../app/layout/AppLayoutRoot.vue')
		const route = read('../../domains/communications/views/CanonicalCommunicationsRoute.vue')
		const detail = read('../../domains/communications/presentation/CanonicalCommunicationDetail.vue')
		const api = read('./api/attachmentPreview.ts')
		const controller = read('./queries/useAttachmentPreview.ts')
		const presentation = read('./presentation/AttachmentPreviewPanel.vue')
		const generator = read('../../../scripts/generate-proto.mjs')
		const navigation = read('../../app/queries/useClientNavigationSurface.ts')

		expect(app).toContain('AttachmentPreviewWorkflow')
		expect(app).toContain("'attachment_preview.client.v1'")
		expect(route).toContain('canonicalAttachmentSelected')
		expect(route).not.toMatch(/workflows\/attachment-preview|attachmentPreviewClient/)
		expect(detail).toContain("emit('selectAttachment', row.key)")
		expect(api).toContain('getAttachmentPreviewCommandClient')
		expect(api).toContain('getAttachmentPreviewQueryClient')
		expect(api).toContain('getAttachmentPreviewTicketClient')
		expect(api).toContain('getBrowserGatewayRealtimeHub')
		expect(api).toContain('BrowserGatewayFetch')
		expect(api).not.toMatch(/domains\/communications|integrations\/(mail|telegram|whatsapp|zulip)/)
		expect(controller).toContain('subscribeAttachmentPreviewStatus')
		expect(controller).not.toMatch(/setInterval\(|setTimeout\(|poll/i)
		expect(navigation).toContain('getBrowserGatewayRealtimeHub().subscribe')
		expect(presentation).not.toMatch(/api\/|connect\/|fetch\(|v-html/)
		expect(generator).toContain('attachment-preview-api')
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
