<script setup lang="ts">
import { Icon } from '@/shared/ui'
import type { TelegramAccountAccessModel } from './telegramAccountAccessModel'
import type { TelegramDiscoveryModel } from './telegramDiscoveryModel'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'

defineProps<{
	accountAccess: TelegramAccountAccessModel
	discovery: TelegramDiscoveryModel
	model: TelegramOperationalPageModel
}>()

const emit = defineEmits<{
	addAccount: []
	load: []
	openSearch: []
	toggleInspector: []
	updateAccountId: [value: string]
	updateSearchQuery: [value: string]
}>()
</script>

<template>
	<header class="telegram-workspace-toolbar">
		<div class="telegram-workspace-toolbar__title">
			<h1>Communications <span>/</span> Telegram</h1>
			<p>{{ accountAccess.authorizationState || 'Telegram runtime' }}</p>
		</div>

		<form class="telegram-workspace-search" @submit.prevent="emit('openSearch')">
			<Icon icon="tabler:search" size="1rem" />
			<input
				:value="discovery.query"
				placeholder="Search Telegram messages or filter chats…"
				autocomplete="off"
				@input="emit('updateSearchQuery', ($event.target as HTMLInputElement).value)"
			>
		</form>

		<label class="telegram-workspace-account">
			<span>Account</span>
			<select
				:value="model.accountId"
				:disabled="accountAccess.accounts.length === 0"
				@change="emit('updateAccountId', ($event.target as HTMLSelectElement).value)"
			>
				<option v-if="accountAccess.accounts.length === 0" value="">No account</option>
				<option v-for="account in accountAccess.accounts" :key="account.id" :value="account.id">
					{{ account.title }}
				</option>
			</select>
		</label>

		<div class="telegram-workspace-toolbar__actions">
			<button
				type="button"
				:disabled="model.status === 'loading' || accountAccess.authorizationState !== 'ready'"
				@click="emit('load')"
			>
				<Icon icon="tabler:refresh" size="1rem" /> Sync chats
			</button>
			<button type="button" @click="emit('openSearch')">
				<Icon icon="tabler:filter" size="1rem" /> Filters
			</button>
			<button type="button" @click="emit('addAccount')">
				<Icon icon="tabler:user-plus" size="1rem" /> Add Account
			</button>
			<button type="button" @click="emit('toggleInspector')">
				<Icon icon="tabler:layout-sidebar-right" size="1rem" /> Details
			</button>
			<button type="button" class="telegram-workspace-toolbar__new">
				New <Icon icon="tabler:plus" size="1rem" />
			</button>
		</div>
	</header>
</template>
