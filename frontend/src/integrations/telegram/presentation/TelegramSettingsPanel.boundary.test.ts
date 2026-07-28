import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Telegram account Settings composition', () => {
	it('coordinates user-account setup into provider QR without a bot credential surface', () => {
		const settings = read('./TelegramSettingsPanel.vue')
		const setup = read('./TelegramAccountSetupPanel.vue')
		const pairing = read('./TelegramQrPairingPanel.vue')
		const coordinator = read('../linking/useTelegramQrPairing.ts')

		expect(settings).toContain('@completed="refreshAccounts"')
		expect(settings).not.toContain('TelegramQrPairingPanel')
		expect(setup).toContain('<Steps')
		expect(setup).toContain('TelegramQrPairingPanel')
		expect(setup).toContain(':start-request="qrStartRequest"')
		expect(setup).toContain('Bot tokens are intentionally not part of this contract')
		expect(pairing).toContain('Telegram user QR login')
		expect(coordinator).toContain('getTelegramAuthorizationStatus')
		expect(coordinator).toContain('telegramQrDataUrl(status.qrLink)')
		for (const source of [settings, setup, pairing, coordinator]) {
			expect(source).not.toMatch(/botToken|bot_token|BotFather/)
			expect(source).not.toMatch(/domains\/communications/)
		}
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
