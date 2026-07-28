import { nextTick, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const {
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
	telegramQrDataUrl,
} = vi.hoisted(() => ({
	getTelegramAuthorizationStatus: vi.fn(),
	submitTelegramAuthorizationPassword: vi.fn(),
	telegramQrDataUrl: vi.fn(),
}))

vi.mock('../api/telegramAuthorizationGateway', () => ({
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
}))
vi.mock('./telegramQrArtifact', () => ({ telegramQrDataUrl }))

import { useTelegramQrPairing } from './useTelegramQrPairing'

describe('Telegram QR pairing', () => {
	beforeEach(() => {
		getTelegramAuthorizationStatus.mockReset()
		submitTelegramAuthorizationPassword.mockReset()
		telegramQrDataUrl.mockReset()
	})

	it('waits for effective Settings and then requests the real TDLib QR automatically', async () => {
		const module = ref({
			capabilityIds: ['telegram.authorization.v1'],
			settings: { effectiveRevision: 0n },
		})
		const startRequest = ref(0)
		getTelegramAuthorizationStatus.mockResolvedValue({
			state: 'waiting_qr_scan',
			qrLink: 'tg://login?token=provider-token',
		})
		telegramQrDataUrl.mockResolvedValue('data:image/png;base64,provider-qr')
		const pairing = useTelegramQrPairing(
			() => module.value as never,
			() => startRequest.value,
		)

		startRequest.value = 1
		await nextTick()
		expect(getTelegramAuthorizationStatus).not.toHaveBeenCalled()
		expect(pairing.message.value).toContain('Waiting for managed Settings')

		module.value.settings.effectiveRevision = 1n
		await nextTick()
		await vi.waitFor(() => expect(getTelegramAuthorizationStatus).toHaveBeenCalledOnce())
		await vi.waitFor(() => expect(pairing.qrDataUrl.value).toBe(
			'data:image/png;base64,provider-qr',
		))
		expect(telegramQrDataUrl).toHaveBeenCalledWith('tg://login?token=provider-token')
	})
})
