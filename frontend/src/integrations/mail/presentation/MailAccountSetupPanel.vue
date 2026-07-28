<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import IntegrationAccountSetupCard from '../../../shared/ui/settings/IntegrationAccountSetupCard.vue'
import { useMailAccountSetup } from '../setup/useMailAccountSetup'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const setup = useMailAccountSetup(() => props.module)
</script>

<template>
	<IntegrationAccountSetupCard
		eyebrow="Provider account"
		title="Add a mail account"
		description="Non-secret connection settings go to Mail Settings. Passwords and OAuth tokens remain in Vault."
		tone="mail"
		icon="tabler:mail-plus"
		:account-state="setup.configured.value ? 'Configured' : 'No account'"
		:submit-label="setup.submitLabel.value"
		:busy="setup.busy.value"
		:disabled="!setup.canSubmit.value"
		:message="setup.message.value || (setup.kind.value === 'imap' && !setup.secureHostAvailable ? 'Secure password commit requires the desktop shell or root make dev.' : '')"
		:message-tone="setup.messageTone.value"
		:expanded="!setup.configured.value"
		@submit="setup.submit"
	>
		<label>
			<span>Connection type</span>
			<select v-model="setup.kind.value">
				<option value="imap">IMAP / SMTP</option>
				<option value="gmail">Gmail OAuth</option>
			</select>
		</label>
		<label>
			<span>Local account ID</span>
			<input v-model="setup.connectionId.value" required maxlength="128" placeholder="personal-mail">
		</label>
		<label class="wide">
			<span>Email / username</span>
			<input v-model="setup.email.value" required type="email" autocomplete="username" placeholder="you@example.com">
		</label>

		<template v-if="setup.kind.value === 'imap'">
			<label>
				<span>IMAP host</span>
				<input v-model="setup.imapHost.value" required placeholder="imap.example.com">
			</label>
			<label>
				<span>IMAP port</span>
				<input v-model="setup.imapPort.value" required inputmode="numeric" pattern="[0-9]+">
			</label>
			<label class="wide">
				<span>IMAP password</span>
				<input v-model="setup.imapPassword.value" required type="password" autocomplete="new-password">
			</label>
			<label class="wide">
				<span>Outbound delivery</span>
				<select v-model="setup.smtpEnabled.value">
					<option :value="false">Configure later</option>
					<option :value="true">Enable SMTP now</option>
				</select>
			</label>
			<template v-if="setup.smtpEnabled.value">
				<label>
					<span>SMTP host</span>
					<input v-model="setup.smtpHost.value" required placeholder="smtp.example.com">
				</label>
				<label>
					<span>SMTP port</span>
					<input v-model="setup.smtpPort.value" required inputmode="numeric" pattern="[0-9]+">
				</label>
				<label class="wide">
					<span>SMTP password (blank = IMAP password)</span>
					<input v-model="setup.smtpPassword.value" type="password" autocomplete="new-password">
				</label>
			</template>
		</template>

		<template v-else>
			<label class="wide">
				<span>Google OAuth client ID</span>
				<input v-model="setup.gmailClientId.value" required autocomplete="off">
			</label>
			<label class="wide">
				<span>OAuth redirect URI</span>
				<input v-model="setup.gmailRedirectUri.value" required type="url">
			</label>
			<a
				v-if="setup.gmailState.value"
				class="wide"
				:href="setup.gmailState.value.started.authorizationUrl"
				target="_blank"
				rel="noreferrer"
			>Open Google authorization</a>
			<label v-if="setup.gmailState.value">
				<span>Returned state</span>
				<input v-model="setup.returnedState.value" required autocomplete="off">
			</label>
			<label v-if="setup.gmailState.value">
				<span>Authorization code</span>
				<input v-model="setup.authorizationCode.value" required type="password" autocomplete="one-time-code">
			</label>
		</template>
	</IntegrationAccountSetupCard>
</template>
