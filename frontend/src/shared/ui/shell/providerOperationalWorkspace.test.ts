import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('provider operational workspace shell', () => {
	it('keeps Mail and Telegram on the shared card-and-gutter layout', () => {
		const styles = readFileSync(
			new URL('./providerOperationalWorkspace.css', import.meta.url),
			'utf8',
		)
		const mail = readFileSync(
			new URL('../../../integrations/mail/presentation/MailOperationalPage.vue', import.meta.url),
			'utf8',
		)
		const telegram = readFileSync(
			new URL('../../../integrations/telegram/presentation/TelegramOperationalPage.vue', import.meta.url),
			'utf8',
		)

		expect(styles).toContain('.mail-operational-page, .telegram-operational-page')
		expect(styles).toContain('gap: var(--h-space-3)')
		expect(styles).toContain('padding: var(--h-space-3)')
		expect(styles).toContain('border-radius: var(--h-radius-lg)')
		expect(styles).toContain('.mail-workspace-list')
		expect(styles).toContain('.telegram-workspace-chat-list')
		expect(mail).toContain("providerOperationalWorkspace.css")
		expect(telegram).toContain("providerOperationalWorkspace.css")
	})
})
