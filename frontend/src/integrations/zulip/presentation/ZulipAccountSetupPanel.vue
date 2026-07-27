<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import IntegrationAccountSetupCard from '../../../shared/ui/settings/IntegrationAccountSetupCard.vue'
import { useZulipAccountSetup } from '../setup/useZulipAccountSetup'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const setup = useZulipAccountSetup(() => props.module)
</script>

<template>
	<IntegrationAccountSetupCard
		eyebrow="Provider account"
		title="Connect a Zulip bot"
		description="Realm and bot identity stay in Zulip Settings; the API key is sealed directly to Vault."
		tone="zulip"
		icon="tabler:brand-zulip"
		:account-state="setup.configured.value ? 'Configured' : 'No account'"
		submit-label="Connect Zulip"
		:busy="setup.busy.value"
		:disabled="!setup.canSubmit.value"
		:message="setup.message.value || (!setup.secureHostAvailable ? 'Secure credential commit is available in the desktop shell.' : '')"
		:message-tone="setup.messageTone.value"
		:expanded="!setup.configured.value"
		@submit="setup.submit"
	>
		<label>
			<span>Local account ID</span>
			<input v-model="setup.accountId.value" required maxlength="128" placeholder="work-zulip">
		</label>
		<label>
			<span>Bot email</span>
			<input v-model="setup.botEmail.value" required type="email" autocomplete="username" placeholder="bot@example.com">
		</label>
		<label class="wide">
			<span>Realm URL</span>
			<input v-model="setup.realmUrl.value" required type="url" placeholder="https://example.zulipchat.com">
		</label>
		<label class="wide">
			<span>API key</span>
			<input v-model="setup.apiKey.value" required type="password" autocomplete="new-password">
		</label>
	</IntegrationAccountSetupCard>
</template>
