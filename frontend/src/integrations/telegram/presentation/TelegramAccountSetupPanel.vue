<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import IntegrationAccountSetupCard from '../../../shared/ui/settings/IntegrationAccountSetupCard.vue'
import { useTelegramAccountSetup } from '../setup/useTelegramAccountSetup'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const setup = useTelegramAccountSetup(() => props.module)
const emit = defineEmits<{ provisioned: [] }>()

async function submit(): Promise<void> {
	if (await setup.submit()) emit('provisioned')
}
</script>

<template>
	<IntegrationAccountSetupCard
		eyebrow="Provider account"
		title="Connect Telegram user"
		description="User account only. The integration owns Telegram API credentials and TDLib session state; bot tokens are not accepted."
		tone="telegram"
		icon="tabler:brand-telegram"
		:account-state="setup.configured.value ? 'Configured' : 'No account'"
		submit-label="Save and show QR"
		:busy="setup.busy.value"
		:disabled="!setup.canSubmit.value"
		:message="setup.message.value || (!setup.secureHostAvailable ? 'Secure credential commit requires the desktop shell or root make dev.' : '')"
		:message-tone="setup.messageTone.value"
		:expanded="!setup.configured.value"
		@submit="submit"
	>
		<label>
			<span>Local account ID</span>
			<input v-model="setup.accountId.value" required maxlength="128" placeholder="personal-telegram">
		</label>
		<label>
			<span>Display name</span>
			<input v-model="setup.displayName.value" required maxlength="128" placeholder="Personal Telegram">
		</label>
		<label>
			<span>Telegram API ID</span>
			<input v-model="setup.apiId.value" required inputmode="numeric" pattern="[0-9]+" placeholder="123456">
		</label>
		<label>
			<span>Telegram API hash</span>
			<input v-model="setup.apiHash.value" required type="password" autocomplete="new-password">
		</label>
	</IntegrationAccountSetupCard>
</template>
