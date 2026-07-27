<script setup lang="ts">
import { onMounted } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import IntegrationAccountLifecycleCard from '../../../shared/ui/settings/IntegrationAccountLifecycleCard.vue'
import { useMailAccountManagement } from '../account-management/useMailAccountManagement'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const management = useMailAccountManagement(() => props.module)

onMounted(() => void management.refresh())

function retire(): void {
	const accountId = management.status.value?.connectionId
	if (accountId && window.confirm(`Retire Mail account ${accountId}? Provider access will be fenced.`)) {
		void management.retire()
	}
}

function deleteAccount(): void {
	const accountId = management.status.value?.connectionId
	if (accountId && window.confirm(`Delete Mail account ${accountId}? This lifecycle action cannot be undone from Settings.`)) {
		void management.deleteAccount()
	}
}
</script>

<template>
	<IntegrationAccountLifecycleCard
		eyebrow="Account lifecycle"
		title="Manage mail account"
		description="Readiness, credential rotation and retirement remain owned by the Mail integration."
		tone="mail"
		icon="tabler:mail-cog"
		:account-state="management.stateLabel.value"
		:busy="management.busy.value"
		:message="management.message.value || (!management.secureHostAvailable ? 'Password rotation is available in the desktop shell.' : '')"
		:message-tone="management.messageTone.value"
	>
		<template #summary>
			<div><small>Account ID</small><strong>{{ management.connectionId.value || 'Not configured' }}</strong></div>
			<div><small>Runtime</small><strong>{{ management.status.value?.runtimeGeneration ?? '—' }}</strong></div>
			<div><small>Settings revision</small><strong>{{ management.status.value?.settingsRevision ?? '—' }}</strong></div>
			<div><small>Lifecycle revision</small><strong>{{ management.status.value?.lifecycleRevision ?? '—' }}</strong></div>
		</template>

		<label v-if="management.canRotateImap.value">
			<span>New IMAP password</span>
			<input v-model="management.imapPassword.value" type="password" autocomplete="new-password">
		</label>
		<label v-if="management.canRotateSmtp.value">
			<span>New SMTP password</span>
			<input v-model="management.smtpPassword.value" type="password" autocomplete="new-password">
		</label>

		<template #actions>
			<button type="button" :disabled="management.busy.value || !management.canQuery.value" @click="management.refresh">
				<Icon icon="tabler:refresh" /> Refresh
			</button>
			<button
				v-if="management.canRefreshLifecycle.value"
				type="button"
				:disabled="management.busy.value"
				@click="management.refreshLifecycle"
			>
				<Icon icon="tabler:activity" /> Operation status
			</button>
			<button
				v-if="management.canRetry.value"
				type="button"
				:disabled="management.busy.value"
				@click="management.retry"
			>
				<Icon icon="tabler:repeat" /> Retry operation
			</button>
			<button
				v-if="management.canRotateImap.value"
				class="primary"
				type="button"
				:disabled="management.busy.value || !management.imapPassword.value"
				@click="management.rotatePassword('imap')"
			>
				<Icon icon="tabler:key" /> Rotate IMAP password
			</button>
			<button
				v-if="management.canRotateSmtp.value"
				class="primary"
				type="button"
				:disabled="management.busy.value || !management.smtpPassword.value"
				@click="management.rotatePassword('smtp')"
			>
				<Icon icon="tabler:key" /> Rotate SMTP password
			</button>
			<button
				v-if="management.canRetire.value"
				class="danger"
				type="button"
				:disabled="management.busy.value"
				@click="retire"
			>
				<Icon icon="tabler:archive" /> Retire
			</button>
			<button
				v-if="management.canDelete.value"
				class="danger"
				type="button"
				:disabled="management.busy.value"
				@click="deleteAccount"
			>
				<Icon icon="tabler:trash" /> Delete
			</button>
		</template>
	</IntegrationAccountLifecycleCard>
</template>
