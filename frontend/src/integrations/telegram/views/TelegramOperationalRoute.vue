<script setup lang="ts">
import TelegramOperationalPage from '../presentation/TelegramOperationalPage.vue'
import { useTelegramOperationalPage } from '../queries/useTelegramOperationalPage'
import { useTelegramAccountAccess } from '../queries/useTelegramAccountAccess'
import { useTelegramDiscovery } from '../queries/useTelegramDiscovery'

const props = defineProps<{
	canAuthorize: boolean
	canManageLifecycle: boolean
	canQuery: boolean
	canSend: boolean
}>()
const surface = useTelegramOperationalPage(() => props.canSend)
const accountAccess = useTelegramAccountAccess({
	canAuthorize: () => props.canAuthorize,
	canManageLifecycle: () => props.canManageLifecycle,
})
const discovery = useTelegramDiscovery({
	accountId: () => accountAccess.selectedAccountId.value,
	canQuery: () => props.canQuery,
	selectedChatId: () => surface.model.value.selectedChatId,
})

async function refreshAccounts(): Promise<void> {
	await accountAccess.refresh()
	updateAccountId(accountAccess.selectedAccountId.value)
}

async function selectAccount(accountId: string): Promise<void> {
	accountAccess.selectAccount(accountId)
	updateAccountId(accountId)
	await surface.loadChats()
}

async function selectChat(providerChatId: string): Promise<void> {
	await surface.selectChat(providerChatId)
}

function updateAccountId(accountId: string): void {
	accountAccess.selectAccount(accountId)
	surface.updateAccountId(accountId)
}
</script>

<template>
	<TelegramOperationalPage
		:account-access="accountAccess.model.value"
		:discovery="discovery.model.value"
		:model="surface.model.value"
		@provision-account="accountAccess.provision"
		@refresh-accounts="refreshAccounts"
		@refresh-chat-context="discovery.refreshChatContext"
		@replay-account="accountAccess.replay"
		@retire-account="accountAccess.retire"
		@select-account="selectAccount"
		@start-account="accountAccess.start"
		@stop-account="accountAccess.stop"
		@submit-authorization-password="accountAccess.submitPassword"
		@load="surface.loadChats"
		@search="discovery.search"
		@select-chat="selectChat"
		@send="surface.send"
		@update-account-id="updateAccountId"
		@update-authorization-password="accountAccess.updatePassword"
		@update-draft="surface.updateDraft"
		@update-search-query="discovery.updateQuery"
		@update-provision-account-id="accountAccess.updateProvisionAccountId"
		@update-provision-display-name="accountAccess.updateProvisionDisplayName"
		@update-provision-external-account-id="accountAccess.updateProvisionExternalAccountId"
	/>
</template>
