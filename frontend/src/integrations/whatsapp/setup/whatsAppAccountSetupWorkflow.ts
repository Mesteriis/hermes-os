import {
	ManagedIntegrationSetupV1,
	type ManagedIntegrationSetupReceiptV1,
} from '../../../platform/settings'

const WHATSAPP_STORAGE_CAPABILITY_ID = 'whatsapp.storage.v1'

type WhatsAppSettingsPortV1 = Pick<ManagedIntegrationSetupV1, 'apply'>

export class WhatsAppAccountSetupWorkflowV1 {
	constructor(
		private readonly settings: WhatsAppSettingsPortV1 =
			new ManagedIntegrationSetupV1(),
	) {}

	async setup(input: {
		registrationId: string
		expectedDesiredRevision: bigint
		accountId: string
	}): Promise<ManagedIntegrationSetupReceiptV1> {
		const accountId = required(input.accountId)
		return this.settings.apply({
			registrationId: input.registrationId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			storageCapabilityId: WHATSAPP_STORAGE_CAPABILITY_ID,
			configurationInstanceId: accountId,
			requestHostBridge: true,
			values: [{
				settingId: 'whatsapp.account_id',
				value: { case: 'stringValue', value: accountId },
			}],
		})
	}
}

function required(value: string): string {
	const normalized = value.trim()
	if (!normalized || normalized.length > 128) {
		throw new Error('whatsapp_account_id_invalid')
	}
	return normalized
}
