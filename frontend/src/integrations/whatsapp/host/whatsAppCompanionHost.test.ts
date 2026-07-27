import { describe, expect, it, vi } from 'vitest'

import { NativeWhatsAppCompanionHostV1 } from './whatsAppCompanionHost'

describe('NativeWhatsAppCompanionHostV1', () => {
	it('opens only the exact account-scoped owner-visible companion', async () => {
		const invoke = vi.fn().mockResolvedValue({
			account_id: 'personal',
			owner_visible: true,
			opened_window: true,
			reused_existing_window: false,
		})
		const host = new NativeWhatsAppCompanionHostV1(invoke, () => true)

		await expect(host.open(' personal ')).resolves.toEqual({
			accountId: 'personal',
			ownerVisible: true,
			openedWindow: true,
			reusedExistingWindow: false,
		})
		expect(invoke).toHaveBeenCalledWith('open_whatsapp_web_companion', {
			request: { account_id: 'personal' },
		})
	})

	it('fails closed outside the desktop host', async () => {
		const host = new NativeWhatsAppCompanionHostV1(vi.fn(), () => false)

		await expect(host.open('personal')).rejects.toThrow('desktop_host_required')
	})
})
