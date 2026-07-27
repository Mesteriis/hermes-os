<script setup lang="ts">
import { onMounted } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import IntegrationAccountLifecycleCard from '../../../shared/ui/settings/IntegrationAccountLifecycleCard.vue'
import { useTelegramAccountManagement } from '../account-management/useTelegramAccountManagement'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const management = useTelegramAccountManagement(() => props.module)

onMounted(() => void management.refresh())

function retire(): void {
	const accountId = management.account.value?.accountId
	if (accountId && window.confirm(`Retire Telegram account ${accountId}? TDLib provider state will be fenced.`)) {
		void management.retire()
	}
}
</script>

<template>
	<IntegrationAccountLifecycleCard
		eyebrow="Account lifecycle"
		title="Manage Telegram account"
		description="Runtime state and replay stay in Telegram; QR authorization is handled by the dedicated pairing panel below."
		tone="telegram"
		icon="tabler:brand-telegram"
		:account-state="management.stateLabel.value"
		:busy="management.busy.value"
		:message="management.message.value"
		:message-tone="management.messageTone.value"
	>
		<template #summary>
			<div><small>Account ID</small><strong>{{ management.accountId.value || 'Not configured' }}</strong></div>
			<div><small>Display name</small><strong>{{ management.account.value?.displayName || '—' }}</strong></div>
			<div><small>Provider state</small><strong>{{ management.account.value?.state || '—' }}</strong></div>
			<div><small>Runtime epoch</small><strong>{{ management.account.value?.runtimeEpoch ?? '—' }}</strong></div>
		</template>

		<template #actions>
			<button type="button" :disabled="management.busy.value || !management.canManage.value" @click="management.refresh">
				<Icon icon="tabler:refresh" /> Refresh
			</button>
			<button
				v-if="management.account.value"
				type="button"
				:disabled="management.busy.value"
				@click="management.replay"
			>
				<Icon icon="tabler:history" /> Replay
			</button>
			<button
				v-if="management.account.value"
				class="primary"
				type="button"
				:disabled="management.busy.value || !management.canReconfigure.value"
				@click="management.restart"
			>
				<Icon icon="tabler:reload" /> Restart runtime
			</button>
			<button
				v-if="management.account.value"
				class="danger"
				type="button"
				:disabled="management.busy.value"
				@click="retire"
			>
				<Icon icon="tabler:archive" /> Retire
			</button>
		</template>
	</IntegrationAccountLifecycleCard>
</template>
