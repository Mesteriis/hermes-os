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
		const businessQueries = source('../../../shared/communications/telegramBusinessQueries.ts')

		expect(businessQueries).toContain("['communications', 'messages', toValue(messageId), 'versions']")
		expect(businessQueries).toContain("['communications', 'messages', toValue(messageId), 'tombstones']")
		expect(businessQueries).toContain("['communications', 'messages', toValue(messageId), 'reply-chain']")
		expect(businessQueries).toContain("['communications', 'messages', toValue(messageId), 'forward-chain']")
		expect(businessQueries).toContain("'attachment-preview'")
		expect(businessQueries).toContain("['communications', 'messages', toValue(messageId), 'raw-evidence']")

		expect(businessQueries).not.toContain(forbiddenIntegrationKey('telegram', 'messages'))
		expect(businessQueries).not.toContain(forbiddenIntegrationKey('telegram', 'chats'))
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
