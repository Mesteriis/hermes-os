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
		const mailStyles = readFileSync(
			new URL('../../../integrations/mail/presentation/mailOperationalPage.css', import.meta.url),
			'utf8',
		)
		const telegramStyles = readFileSync(
			new URL('../../../integrations/telegram/presentation/telegramOperationalPage.css', import.meta.url),
			'utf8',
		)
		const telegramInspectorStyles = readFileSync(
			new URL('../../../integrations/telegram/presentation/telegramMessageInspector.css', import.meta.url),
			'utf8',
		)
		const mailToolbar = readFileSync(
			new URL('../../../integrations/mail/presentation/MailWorkspaceToolbar.vue', import.meta.url),
			'utf8',
		)
		const telegramToolbar = readFileSync(
			new URL('../../../integrations/telegram/presentation/TelegramWorkspaceToolbar.vue', import.meta.url),
			'utf8',
		)

		expect(styles).toContain('.mail-operational-page, .telegram-operational-page')
		expect(styles).toContain('gap: var(--h-space-5)')
		expect(styles).toContain('padding: var(--h-space-5)')
		expect(styles).toContain('gap: var(--h-space-4)')
		expect(styles).toContain('background: var(--h-color-surface-raised)')
		expect(styles).toContain('background: var(--h-color-surface)')
		expect(styles).toContain('border: 1px solid var(--h-color-border-strong)')
		expect(styles).toContain('grid-template-columns: minmax(0, 1fr) auto')
		expect(styles).toContain('grid-column: 1 / -1')
		expect(styles).toContain('@media (max-width: 480px)')
		expect(styles).toContain('border-radius: var(--h-radius-lg)')
		expect(styles).toContain('.mail-folder-strip')
		expect(styles).toContain('.mail-list-view')
		expect(styles).toContain('.telegram-workspace-chat-list')
		expect(styles).toContain('.mail-list-item, .telegram-workspace-chat')
		expect(styles).toContain('grid-auto-rows: max-content')
		expect(styles).toContain('box-shadow: var(--h-shadow-xs)')
		expect(mailToolbar).not.toContain('mail-workspace-account')
		expect(telegramToolbar).not.toContain('telegram-workspace-account')
		expect(mailStyles).toMatch(/\.mail-workspace-toolbar__compose\s*\{\s*font-size:\s*0/)
		expect(mailStyles).toContain('radial-gradient(')
		expect(mailStyles).toContain('#bd4a21 10%')
		expect(telegramStyles).toContain('radial-gradient(')
		expect(telegramStyles).toContain('#229ed9 10%')
		expect(telegramStyles).toMatch(
			/\.telegram-workspace-inspector__body\s*\{[^}]*display:\s*grid[^}]*gap:\s*var\(--h-space-3\)/s,
		)
		expect(telegramStyles).toMatch(
			/\.telegram-workspace-inspector__body \.telegram-message-inspector,[^{]*\{[^}]*margin:\s*0[^}]*background:\s*var\(--h-color-surface-raised\)/s,
		)
		expect(telegramInspectorStyles).toContain(
			'grid-template-columns: repeat(auto-fit, minmax(min(100%, 9rem), 1fr))',
		)
		expect(telegramInspectorStyles).not.toContain(
			'grid-template-columns: repeat(3, minmax(0, 1fr))',
		)
		expect(mail).toContain("providerOperationalWorkspace.css")
		expect(telegram).toContain("providerOperationalWorkspace.css")
		expect(mail).toContain('useResponsiveWorkspaceInspector()')
		expect(telegram).toContain('useResponsiveWorkspaceInspector()')
	})
})
