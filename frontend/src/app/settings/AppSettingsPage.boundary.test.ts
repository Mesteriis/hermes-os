import { existsSync, readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('app settings clean-room composition', () => {
	it('composes platform and exact integration-owned panels without a settings domain', () => {
		const page = readFileSync(new URL('./AppSettingsPage.vue', import.meta.url), 'utf8')
		const modules = readFileSync(new URL('./clientSettingsModules.ts', import.meta.url), 'utf8')
		const layout = readFileSync(new URL('../layout/AppLayoutRoot.vue', import.meta.url), 'utf8')
		const main = readFileSync(new URL('../../main.ts', import.meta.url), 'utf8')
		const globalSurfaces = readFileSync(new URL('../../styles/surfaces.css', import.meta.url), 'utf8')

		expect(page).toContain('SystemControlPage')
		expect(page).toContain('MailSettingsPanel')
		expect(page).toContain('TelegramSettingsPanel')
		expect(page).toContain('WhatsAppSettingsPanel')
		expect(page).toContain('ZulipSettingsPanel')
		expect(page).not.toContain('domains/settings')
		expect(page).not.toContain('domains/communications')
		expect(page).not.toMatch(/class="settings-/)
		expect(page).not.toMatch(/\/api\/v1\/|ApiClient|fetch\(|useMutation/)
		expect(modules).toContain('hermes-mail-runtime')
		expect(modules).toContain('hermes-telegram-runtime')
		expect(modules).toContain('hermes-whatsapp-runtime')
		expect(modules).toContain('hermes-zulip-runtime')
		expect(layout).toContain('AppSettingsPage')
		expect(layout).not.toContain('SystemControlPage')
		expect(main).not.toContain("styles/settings-")
		expect(globalSurfaces).not.toMatch(/settings-|communications-|telegram-/)
		expect(existsSync(new URL('../../styles/settings-background-jobs.css', import.meta.url))).toBe(false)
		expect(existsSync(new URL('../../styles/settings-maintenance.css', import.meta.url))).toBe(false)
		expect(existsSync(new URL('../../styles/settings-signal-hub.css', import.meta.url))).toBe(false)
		expect(existsSync(new URL('../../styles/settings-trace-logs.css', import.meta.url))).toBe(false)
	})
})
