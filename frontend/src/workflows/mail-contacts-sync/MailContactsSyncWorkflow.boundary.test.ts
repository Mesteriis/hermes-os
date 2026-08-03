import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Mail Contacts Sync frontend boundary', () => {
	it('keeps app composition outside Mail and uses generated Start Get plus shared SSE', () => {
		const app = readFileSync(new URL('../../app/settings/MailSettingsComposition.vue', import.meta.url), 'utf8')
		const mail = readFileSync(new URL('../../integrations/mail/presentation/MailSettingsPanel.vue', import.meta.url), 'utf8')
		const api = readFileSync(new URL('./api/mailContactsSync.ts', import.meta.url), 'utf8')
		const controller = readFileSync(new URL('./queries/useMailContactsSyncSettings.ts', import.meta.url), 'utf8')
		const panel = readFileSync(new URL('./presentation/MailContactsSyncSettingsPanel.vue', import.meta.url), 'utf8')

		expect(app).toContain('MailSettingsPanel')
		expect(app).toContain('MailContactsSyncSettingsPanel')
		expect(mail).not.toMatch(/mail-contacts-sync|MailContactsSync/)
		expect(api).toContain('getBrowserGatewayRealtimeHub')
		expect(api).toContain('getMailContactsSyncCommandClient')
		expect(api).toContain('getMailContactsSyncQueryClient')
		expect(controller).toMatch(/await realtime\.ready[\s\S]*startMailContactsSync[\s\S]*getMailContactsSync/)
		expect(panel).not.toMatch(/\.\.\/api|ManagedWorkflowSetupV1/)
		expect(controller).not.toMatch(/setInterval|setTimeout|poll/i)
	})
})
