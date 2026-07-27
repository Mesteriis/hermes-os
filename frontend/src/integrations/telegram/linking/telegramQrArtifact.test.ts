import { describe, expect, it } from 'vitest'

import { telegramQrDataUrl } from './telegramQrArtifact'

describe('telegramQrDataUrl', () => {
	it('renders a provider-issued Telegram login link locally', async () => {
		const result = await telegramQrDataUrl('tg://login?token=provider-token')

		expect(result).toMatch(/^data:image\/png;base64,/)
	})

	it.each([
		'',
		'https://telegram.org/login?token=x',
		'tg://resolve?token=x',
		'tg://login',
	])('rejects a non-login artifact: %s', async (value) => {
		await expect(telegramQrDataUrl(value)).rejects.toThrow('telegram_qr_link_invalid')
	})
})
