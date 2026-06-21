import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

function source(path: string): string {
	return readFileSync(new URL(path, import.meta.url), 'utf8')
}

function forbiddenIntegrationKey(provider: string, segment: string): string {
	return `['integrations', '${provider}', '${segment}'`
}

describe('provider query key ownership boundary', () => {
	it('keeps Telegram business read-model keys under communications', () => {
		const lifecycle = source('./useTelegramLifecycleQuery.ts')
		const references = source('./useTelegramReferenceQuery.ts')
		const attachments = source('./useTelegramAttachmentPreviewQuery.ts')
		const rawEvidence = source('./useTelegramRawEvidenceQuery.ts')

		expect(lifecycle).toContain("['communications', 'messages', toValue(messageId), 'versions']")
		expect(lifecycle).toContain("['communications', 'messages', toValue(messageId), 'tombstones']")
		expect(references).toContain("['communications', 'messages', toValue(messageId), 'reply-chain']")
		expect(references).toContain("['communications', 'messages', toValue(messageId), 'forward-chain']")
		expect(attachments).toContain("['communications', 'messages', toValue(attachmentId) ?? 'none', 'attachment-preview']")
		expect(rawEvidence).toContain("['communications', 'messages', toValue(messageId), 'raw-evidence']")

		for (const fileSource of [lifecycle, references, attachments, rawEvidence]) {
			expect(fileSource).not.toContain(forbiddenIntegrationKey('telegram', 'messages'))
			expect(fileSource).not.toContain(forbiddenIntegrationKey('telegram', 'chats'))
		}
	})

	it('keeps WhatsApp session and message read-model keys under communications', () => {
		const whatsapp = source('../../whatsapp/queries/useWhatsappQuery.ts')

		expect(whatsapp).toContain("queryKey: ['communications', 'whatsapp', 'sessions', accountId ?? 'all', limit]")
		expect(whatsapp).toContain("queryKey: ['communications', 'whatsapp', 'messages', accountId ?? 'all', providerChatId ?? 'all', limit]")
		expect(whatsapp).toContain("queryKey: ['integrations', 'whatsapp', 'capabilities']")
		expect(whatsapp).not.toContain(`queryKey: ${forbiddenIntegrationKey('whatsapp', 'sessions')}`)
		expect(whatsapp).not.toContain(`queryKey: ${forbiddenIntegrationKey('whatsapp', 'messages')}`)
	})
})
