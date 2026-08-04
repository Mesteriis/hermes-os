import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Call Transcription workflow boundary', () => {
	it('is app-composed, generated-contract-only and absent from Communications domain implementation', () => {
		const app = read('../../app/layout/AppLayoutRoot.vue')
		const domainRoute = read('../../domains/communications/views/CanonicalCommunicationsRoute.vue')
		const api = read('./api/callTranscription.ts')
		const controller = read('./queries/useCallTranscription.ts')
		const presentation = read('./presentation/CallTranscriptionPanel.vue')
		const generator = read('../../../scripts/generate-proto.mjs')

		expect(app).toContain('CallTranscriptionWorkflow')
		expect(app).toContain("'call_transcription.v1'")
		expect(domainRoute).not.toMatch(/call-transcription|CallTranscription/)
		expect(api).toContain('getCallTranscriptionCommandClient')
		expect(api).toContain('getCallTranscriptionQueryClient')
		expect(api).toContain('getCallTranscriptTicketClient')
		expect(api).toContain('BrowserGatewayFetch')
		expect(api).toContain('getBrowserGatewayRealtimeHub')
		expect(api).not.toMatch(/integrations\/(mail|telegram|whatsapp|zulip)|provider|audioBytes|transcriptBytes/)
		expect(controller).toContain('openCallTranscriptionRealtime')
		expect(controller).toContain('readCallTranscript')
		expect(controller).not.toMatch(/setInterval|setTimeout|poll|crypto\.getRandomValues/i)
		expect(presentation).not.toMatch(/api\/|connect\/|fetch\(|v-html/)
		expect(generator).toContain('call-transcription-api')
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
