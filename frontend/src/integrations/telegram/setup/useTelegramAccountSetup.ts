import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import { hasNativeOwnerVaultProvisioningHostV1 } from '../../../platform/vault'
import { TelegramAccountSetupWorkflowV1 } from './telegramAccountSetupWorkflow'

export function useTelegramAccountSetup(
	module: () => ClientModuleBootstrapV1 | null,
	workflow = new TelegramAccountSetupWorkflowV1(),
) {
	const accountId = ref('')
	const displayName = ref('')
	const apiId = ref('')
	const apiHash = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const secureHostAvailable = hasNativeOwnerVaultProvisioningHostV1()
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canSubmit = computed(() => Boolean(
		module()?.settings
		&& accountId.value.trim()
		&& displayName.value.trim()
		&& apiId.value.trim()
		&& apiHash.value,
	))

	async function submit(): Promise<void> {
		const current = module()
		if (!current?.settings || !canSubmit.value) return
		if (!secureHostAvailable) {
			message.value = 'Open the desktop shell to seal Telegram API hash and session key. Browser Settings never receives them.'
			messageTone.value = 'neutral'
			return
		}
		busy.value = true
		message.value = ''
		try {
			await workflow.setup({
				registrationId: current.registrationId,
				expectedDesiredRevision: current.settings.desiredRevision,
				accountId: accountId.value,
				displayName: displayName.value,
				apiId: BigInt(apiId.value),
				apiHash: new TextEncoder().encode(apiHash.value),
			})
			apiHash.value = ''
			message.value = 'Telegram account provisioned. Continue provider authorization on the Communications page.'
			messageTone.value = 'success'
		} catch {
			apiHash.value = ''
			message.value = 'Telegram setup failed before provider authorization. No secret was written to Settings.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	return {
		accountId,
		displayName,
		apiId,
		apiHash,
		busy,
		message,
		messageTone,
		configured,
		canSubmit,
		secureHostAvailable,
		submit,
	}
}
