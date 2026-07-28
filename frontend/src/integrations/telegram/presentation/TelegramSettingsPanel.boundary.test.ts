import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Telegram account Settings composition', () => {
	it('coordinates user-account setup into provider QR without a bot credential surface', () => {
		const settings = read('./TelegramSettingsPanel.vue')
		const setup = read('./TelegramAccountSetupPanel.vue')
		const pairing = read('./TelegramQrPairingPanel.vue')
		const coordinator = read('../linking/useTelegramQrPairing.ts')

		expect(settings).toContain('@provisioned="startQrAuthorization"')
		expect(settings).toContain(':start-request="qrStartRequest"')
		expect(setup).toContain('Save and show QR')
		expect(setup).toContain('bot tokens are not accepted')
		expect(pairing).toContain('Telegram user QR login')
		expect(coordinator).toContain('getTelegramAuthorizationStatus')
		for (const source of [settings, setup, pairing, coordinator]) {
			expect(source).not.toMatch(/botToken|bot_token|BotFather/)
			expect(source).not.toMatch(/domains\/communications/)
		}
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
