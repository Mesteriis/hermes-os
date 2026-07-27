import { invoke } from '@tauri-apps/api/core'

export type WhatsAppCompanionManifestV1 = {
	accountId: string
	ownerVisible: boolean
	openedWindow: boolean
	reusedExistingWindow: boolean
}

export interface WhatsAppCompanionHostV1 {
	available(): boolean
	open(accountId: string): Promise<WhatsAppCompanionManifestV1>
}

type HostInvoke = (
	command: string,
	args?: Record<string, unknown>,
) => Promise<unknown>
type HostAvailability = () => boolean

type NativeWhatsAppCompanionManifest = {
	account_id: string
	owner_visible: boolean
	opened_window: boolean
	reused_existing_window: boolean
}

export class NativeWhatsAppCompanionHostV1 implements WhatsAppCompanionHostV1 {
	constructor(
		private readonly invokeImpl: HostInvoke = invoke as HostInvoke,
		private readonly availability: HostAvailability = nativeHostAvailable,
	) {}

	available(): boolean {
		return this.availability()
	}

	async open(accountId: string): Promise<WhatsAppCompanionManifestV1> {
		const exactAccountId = accountId.trim()
		if (!exactAccountId) throw new RangeError('whatsapp_account_id_invalid')
		if (!this.available()) throw new Error('desktop_host_required')
		const response = await this.invokeImpl('open_whatsapp_web_companion', {
			request: { account_id: exactAccountId },
		}) as NativeWhatsAppCompanionManifest
		return {
			accountId: response.account_id,
			ownerVisible: response.owner_visible,
			openedWindow: response.opened_window,
			reusedExistingWindow: response.reused_existing_window,
		}
	}
}

function nativeHostAvailable(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}
