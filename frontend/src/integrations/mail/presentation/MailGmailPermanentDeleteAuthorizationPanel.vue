<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import '../../../shared/ui/settings/integrationAccountSetupCard.css'
import { useMailGmailPermanentDeleteAuthorization } from '../setup/useMailGmailPermanentDeleteAuthorization'

const authorization = useMailGmailPermanentDeleteAuthorization()
</script>

<template>
	<section class="integration-account-setup" data-provider-tone="mail">
		<header class="integration-account-setup__header">
			<span class="integration-account-setup__icon"><Icon icon="tabler:shield-lock" /></span>
			<div>
				<small>Gmail authority</small>
				<h3>Permanent deletion</h3>
				<p>Optional broad Gmail permission, requested separately and only after owner action.</p>
			</div>
			<strong>Explicit opt-in</strong>
		</header>
		<details>
			<summary>
				<span><Icon icon="tabler:key" /> Authorize permanent deletion</span>
				<Icon icon="tabler:chevron-down" />
			</summary>
			<form class="integration-account-setup__form" @submit.prevent="authorization.submit">
				<div class="integration-account-setup__fields">
					<a
						v-if="authorization.started.value"
						class="wide"
						:href="authorization.started.value.authorizationUrl"
						target="_blank"
						rel="noreferrer"
					>Open Google authorization</a>
					<label v-if="authorization.started.value">
						<span>Returned state</span>
						<input v-model="authorization.returnedState.value" required autocomplete="off">
					</label>
					<label v-if="authorization.started.value">
						<span>Authorization code</span>
						<input v-model="authorization.authorizationCode.value" required type="password" autocomplete="one-time-code">
					</label>
				</div>
				<footer>
					<p
						v-if="authorization.message.value"
						:class="authorization.failed.value
							? 'integration-account-setup__message--error'
							: 'integration-account-setup__message--neutral'"
						aria-live="polite"
					>
						{{ authorization.message.value }}
					</p>
					<button v-if="authorization.operationId.value" type="button" :disabled="authorization.busy.value" @click="authorization.refreshStatus">
						Refresh status
					</button>
					<button type="submit" :disabled="authorization.busy.value">
						<Icon :icon="authorization.busy.value ? 'tabler:loader-2' : 'tabler:shield-check'" />
						{{ authorization.busy.value ? 'Applying…' : authorization.submitLabel.value }}
					</button>
				</footer>
			</form>
		</details>
	</section>
</template>
